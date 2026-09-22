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

use datacenter_simulator::{Simulator, linux::LinuxFabric, manifest::Manifest};
fn main() -> anyhow::Result<()> {
    patchbay::init_userns()?;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(run())
}
async fn run() -> anyhow::Result<()> {
    let mut api = Simulator::new();
    let id = api.import(
        Manifest::from_json(include_str!("gpu-spine-leaf.json"))?,
        true,
    )?;
    let sim = api.get(&id)?;
    let a = sim.node_named("gpu1")?.id.clone();
    let b = sim.node_named("gpu2")?.id.clone();
    let first = sim.route(&a, &b, 0)?[0].clone();
    let mut linux = LinuxFabric::build(sim).await?;
    linux.ping(&a, &b).await?;
    anyhow::ensure!(
        linux
            .tcp_transfer(&a, &b, b"GPU-FABRIC-TEST!!".to_vec())
            .await?
            == b"GPU-FABRIC-TEST!!",
        "TCP payload mismatch"
    );
    let mut conditions = linux.model().links().find(|l| l.id == first).unwrap().spec;
    conditions.bandwidth_bps = 1_000_000_000;
    conditions.latency_ns = 100_000;
    linux.set_link_spec(&first, conditions).await?;
    anyhow::ensure!(
        linux.model().links().find(|l| l.id == first).unwrap().spec == conditions,
        "live link model did not commit"
    );
    anyhow::ensure!(
        linux.tcp_transfer(&a, &b, vec![42; 8192]).await? == vec![42; 8192],
        "TCP payload after live rate/delay update"
    );
    linux.set_link_up(&first, false).await?;
    anyhow::ensure!(
        linux.ping(&a, &b).await.is_err(),
        "partition unexpectedly has a route"
    );
    linux.set_link_up(&first, true).await?;
    linux.ping(&a, &b).await?;
    let _ = linux.raw_device(&a)?;
    anyhow::ensure!(
        linux.ping(&a, &b).await.is_err(),
        "raw access did not invalidate synchronization"
    );
    println!("PASS: real TCP payload, isolated partition, and link recovery");
    Ok(())
}
