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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RdmaDevice {
    pub name: String,
    pub port: u8,
    pub gid_index: u32,
}
pub fn loopback_on(selection: &RdmaDevice, payload: &[u8], timeout: Duration) -> Result<Vec<u8>> {
    if payload.is_empty() || payload.len() > 1024 * 1024 || timeout.is_zero() {
        bail!("payload must be 1 byte..1 MiB and timeout must be positive");
    }
    let devices = ibverbs::devices()?;
    let device = devices
        .iter()
        .find(|d| {
            d.name()
                .is_some_and(|n| n.to_bytes() == selection.name.as_bytes())
        })
        .context("configured RDMA device is absent")?;
    let ctx = device.open()?;
    let cq = ctx.create_cq(16).build()?;
    let pd = ctx.alloc_pd()?;
    if selection.port == 0 {
        bail!("RDMA port must be explicit and nonzero");
    }
    ctx.query_port(selection.port)?;
    ctx.query_gid(selection.port, selection.gid_index)?;
    let prepared = pd
        .create_qp::<ibverbs::Rc>(&cq, &cq, selection.port)?
        .set_gid_index(selection.gid_index)
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

/// Compatibility entry point now requires explicit runner configuration.
pub fn loopback(payload: &[u8], timeout: Duration) -> Result<Vec<u8>> {
    let selection = RdmaDevice {
        name: std::env::var("SIMULATOR_RDMA_DEVICE").context("set SIMULATOR_RDMA_DEVICE")?,
        port: std::env::var("SIMULATOR_RDMA_PORT")
            .context("set SIMULATOR_RDMA_PORT")?
            .parse()?,
        gid_index: std::env::var("SIMULATOR_RDMA_GID_INDEX")
            .context("set SIMULATOR_RDMA_GID_INDEX")?
            .parse()?,
    };
    loopback_on(&selection, payload, timeout)
}
