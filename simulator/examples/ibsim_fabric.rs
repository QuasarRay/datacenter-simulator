// SPDX-License-Identifier: RPL-1.5
use anyhow::{Context, Result, ensure};
use datacenter_simulator::{Simulator, infiniband::IbSimulation, manifest::Manifest, model::Role};
use patchbay::infiniband::IbOptions;
use std::{path::PathBuf, time::Duration};
use tokio::process::Command;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(
        args.len() == 2,
        "usage: ibsim_fabric manifest.json NEW_STATE_DIR"
    );
    let manifest = Manifest::read(&args[0])?;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("vendor/patchbay/vendor/ibsim");
    let options = IbOptions {
        binary: root.join("ibsim/ibsim"),
        umad_library: root.join("umad2sim/libumad2sim.so"),
        state_dir: PathBuf::from(&args[1]),
    };
    patchbay::init_userns()?;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(run(manifest, options))
}
async fn run(manifest: Manifest, options: IbOptions) -> Result<()> {
    let report_dir = options.state_dir.clone();
    let mut api = Simulator::new();
    let id = api.import(manifest, true)?;
    let model = api.get(&id)?;
    let mut hosts: Vec<_> = model
        .nodes()
        .filter(|n| n.spec.role == Role::Host)
        .map(|n| n.spec.name.clone())
        .collect();
    hosts.sort();
    ensure!(
        hosts.len() >= 2,
        "smoke requires at least two connected HCAs"
    );
    let (source, target) = (&hosts[0], &hosts[1]);
    let target_id = model.node_named(target)?.id.clone();
    let cuts: Vec<_> = model
        .links()
        .filter(|l| {
            l.spec.up
                && l.interfaces
                    .iter()
                    .any(|id| model.interface(id).is_ok_and(|i| i.node == target_id))
        })
        .map(|l| l.id.clone())
        .collect();
    ensure!(!cuts.is_empty(), "target HCA must have a connected cable");
    let mut fabric = IbSimulation::build(model, options).await?;
    fabric.start_subnet_manager(source, "opensm").await?;
    for node in hosts.iter() {
        fabric.wait_active(node, Duration::from_secs(30)).await?;
    }
    let baseline = fabric
        .run(
            source,
            Command::new("ibnetdiscover"),
            Duration::from_secs(20),
        )
        .await?;
    ensure!(
        baseline.status.success(),
        "ibnetdiscover: {}",
        String::from_utf8_lossy(&baseline.stderr)
    );
    let discovered = String::from_utf8(baseline.stdout)?;
    for native in fabric.plan().nodes.values() {
        ensure!(
            discovered.contains(&format!("'{native}'"))
                || discovered.contains(&format!("\"{native}\"")),
            "ibnetdiscover omitted {native}"
        );
    }
    std::fs::write(report_dir.join("discovery-before.txt"), discovered)?;
    let target_native = fabric.plan().nodes[target].clone();
    let source_lid = fabric.port_status(source)?.lid;
    let target_lid = fabric.port_status(target)?.lid;
    let mut trace = Command::new("ibtracert");
    trace.args([source_lid.to_string(), target_lid.to_string()]);
    ensure!(
        fabric
            .run(source, trace, Duration::from_secs(20))
            .await?
            .status
            .success(),
        "OpenSM forwarding trace failed"
    );
    for link in &cuts {
        fabric.set_link_up(link, false).await?;
    }
    ensure!(
        fabric.port_status(target)?.state == 1,
        "native HCA did not go Down"
    );
    let partition = fabric
        .run(
            source,
            Command::new("ibnetdiscover"),
            Duration::from_secs(20),
        )
        .await?;
    ensure!(
        partition.status.success(),
        "partition discovery failed to execute"
    );
    let partition = String::from_utf8(partition.stdout)?;
    ensure!(
        !partition.contains(&target_native),
        "IB partition was bypassed"
    );
    std::fs::write(report_dir.join("discovery-partition.txt"), partition)?;
    for link in &cuts {
        fabric.set_link_up(link, true).await?;
    }
    fabric.wait_active(target, Duration::from_secs(30)).await?;
    let recovered = fabric
        .run(
            source,
            Command::new("ibnetdiscover"),
            Duration::from_secs(20),
        )
        .await?;
    ensure!(
        recovered.status.success()
            && String::from_utf8_lossy(&recovered.stdout).contains(&target_native),
        "native IB recovery failed"
    );
    std::fs::write(report_dir.join("discovery-recovered.txt"), recovered.stdout)?;
    let report = serde_json::json!({"ok":true,"scope":"infiniband-management","native_discovery":true,"opensm_routing":true,"partition":true,"recovery":true,"rdma_payload_tested":false,"nccl_tested":false,"mapping":fabric.plan()});
    fabric.shutdown().await.context("reap native IB services")?;
    std::fs::write(
        report_dir.join("result.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
