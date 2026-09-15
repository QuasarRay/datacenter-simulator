// SPDX-License-Identifier: RPL-1.5
// Rust simulator extension of QuasarRay/datacenter-simulator.
// Unless explicitly acquired and licensed from Licensor under another license,
// the contents of this file are subject to the Reciprocal Public License
// ("RPL") Version 1.5, or subsequent versions as allowed by the RPL, and You
// may not copy or use this file in either source code or executable form,
// except in compliance with the terms and conditions of the RPL.
// All software distributed under the RPL is provided strictly on an "AS IS"
// basis, WITHOUT WARRANTY OF ANY KIND, EITHER EXPRESS OR IMPLIED, AND LICENSOR
// HEREBY DISCLAIMS ALL SUCH WARRANTIES, INCLUDING WITHOUT LIMITATION, ANY
// WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, QUIET
// ENJOYMENT, OR NON-INFRINGEMENT. See ../license.md for the RPL's specific
// language governing rights and limitations.

//! Actual registered-memory send/receive via the selected rust-ibverbs fork.
use anyhow::{Context, Result, bail};
use std::time::{Duration, Instant};

/// Execute a bounded reliable-connected loopback on an RDMA or SoftRoCE device.
/// Absence of a device is an error; it never becomes a successful emulated result.
pub fn loopback(payload: &[u8], timeout: Duration) -> Result<Vec<u8>> {
    if payload.is_empty() || payload.len() > 1024 * 1024 || timeout.is_zero() {
        bail!("payload must be 1 byte..1 MiB and timeout must be positive");
    }
    let devices = ibverbs::devices()?;
    let device = devices
        .iter()
        .next()
        .context("no RDMA device; configure SoftRoCE or attach an RNIC")?;
    let ctx = device.open()?;
    let cq = ctx.create_cq(16).build()?;
    let pd = ctx.alloc_pd()?;
    let gid = ctx
        .routable_gid(1)?
        .context("no routable GID on RDMA port 1")?
        .gid_index;
    let prepared = pd
        .create_qp::<ibverbs::Rc>(&cq, &cq, 1)?
        .set_gid_index(gid)
        .build()?;
    let endpoint = prepared.endpoint()?;
    // MR is declared before QP so QP is destroyed first on errors/timeouts.
    let mut mr = pd.allocate(payload.len() * 2, ibverbs::AccessFlags::PERMISSIVE)?;
    let mut qp = prepared.handshake(endpoint)?;
    mr.bytes_mut()[payload.len()..].copy_from_slice(payload);
    // SAFETY: disjoint receive/send slices are registered for the QP's PD.
    // They remain alive and are not touched until their completions are observed.
    unsafe { qp.post_recv([ibverbs::RecvRequest::new(2, &[mr.slice(..payload.len())])]) }?;
    let mut batch = qp.start_send();
    batch.op().signaled().send(1, &[mr.slice(payload.len()..)]);
    // SAFETY: the registered send slice remains alive until completion/QP destruction.
    unsafe { batch.submit() }?;
    let deadline = Instant::now()
        .checked_add(timeout)
        .context("timeout overflow")?;
    let mut sent = false;
    let mut received = false;
    while !sent || !received {
        {
            let mut completions = cq.poll()?;
            while let Some(wc) = completions.next() {
                wc.ok()?;
                match wc.wr_id() {
                    1 if !sent => sent = true,
                    2 if !received => received = true,
                    _ => bail!("unexpected or duplicate work completion"),
                }
            }
        }
        if Instant::now() >= deadline {
            bail!("RDMA completion timeout");
        }
        std::thread::yield_now();
    }
    let result = mr.bytes_mut()[..payload.len()].to_vec();
    if result != payload {
        bail!("RDMA payload mismatch");
    }
    Ok(result)
}
