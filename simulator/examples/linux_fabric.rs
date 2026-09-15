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
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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
    let target = linux.address(&b)?;
    // Bind first, then signal readiness across namespaces without timing sleeps.
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    let server = linux.device(&b)?.spawn(move |_| async move {
        let listener = tokio::net::TcpListener::bind((target, 19000)).await?;
        ready_tx
            .send(())
            .map_err(|_| anyhow::anyhow!("client disappeared"))?;
        let (mut socket, _) = listener.accept().await?;
        let mut payload = [0u8; 17];
        socket.read_exact(&mut payload).await?;
        anyhow::ensure!(&payload == b"GPU-FABRIC-TEST!!", "payload mismatch");
        anyhow::Ok(())
    })?;
    ready_rx.await?;
    let client = linux.device(&a)?.spawn(move |_| async move {
        let mut socket = tokio::net::TcpStream::connect((target, 19000)).await?;
        socket.write_all(b"GPU-FABRIC-TEST!!").await?;
        anyhow::Ok(())
    })?;
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        client.await??;
        server.await??;
        anyhow::Ok(())
    })
    .await??;
    linux.set_link_up(&first, false).await?;
    anyhow::ensure!(
        linux.ping(&a, &b).await.is_err(),
        "partition unexpectedly has a route"
    );
    linux.set_link_up(&first, true).await?;
    linux.ping(&a, &b).await?;
    println!("PASS: real TCP payload, isolated partition, and link recovery");
    Ok(())
}
