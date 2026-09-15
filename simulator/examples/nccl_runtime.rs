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

//! Hardware gate: no mocks, skipped devices, Python, or substitute collectives.
use anyhow::{Result, ensure};
use datacenter_simulator::{
    Simulator,
    collective::{Collective, NcclOptions, Reduction},
    linux::LinuxFabric,
    manifest::Manifest,
};
use std::{path::PathBuf, process::Command};
fn main() -> Result<()> {
    let worker = PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("pass the native datacenter-simulator binary"),
    );
    patchbay::init_userns()?;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(run(worker))
}
async fn run(worker: PathBuf) -> Result<()> {
    let mut api = Simulator::new();
    let id = api.import(
        Manifest::from_json(include_str!("gpu-spine-leaf.json"))?,
        true,
    )?;
    let sim = api.get(&id)?;
    let ranks = vec![
        sim.node_named("gpu1")?.id.clone(),
        sim.node_named("gpu2")?.id.clone(),
    ];
    let fabric = LinuxFabric::build(sim).await?;
    let options = NcclOptions {
        devices: vec![0, 1],
        timeout_secs: 60,
    };
    let before = serde_json::to_value(sim)?;
    let mut cases = Vec::new();
    for (reduction, expected) in [
        (Reduction::Sum, 5.0),
        (Reduction::Product, 6.0),
        (Reduction::Min, 2.0),
        (Reduction::Max, 3.0),
        (Reduction::Average, 2.5),
    ] {
        // A count not divisible by rank count exercises NCCL's own chunk handling.
        cases.push((
            Collective::AllReduce {
                inputs: vec![vec![2.0; 7], vec![3.0; 7]],
                reduction,
            },
            vec![vec![expected; 7]; 2],
        ));
    }
    cases.extend([
        (
            Collective::Broadcast {
                root: 1,
                input: vec![4.0, 8.0],
            },
            vec![vec![4.0, 8.0]; 2],
        ),
        (
            Collective::AllGather {
                inputs: vec![vec![1.0, 2.0], vec![3.0, 4.0]],
            },
            vec![vec![1.0, 2.0, 3.0, 4.0]; 2],
        ),
        (
            Collective::ReduceScatter {
                inputs: vec![vec![1.0, 2.0, 3.0, 4.0]; 2],
                reduction: Reduction::Sum,
            },
            vec![vec![2.0, 4.0], vec![6.0, 8.0]],
        ),
        (
            Collective::AllReduce {
                inputs: vec![vec![]; 2],
                reduction: Reduction::Sum,
            },
            vec![vec![]; 2],
        ),
    ]);
    for (request, expected) in cases {
        let result = fabric
            .collective(&ranks, &request, &options, &worker)
            .await?;
        ensure!(
            result.backend == "nccl" && result.transport == "Socket",
            "wrong execution backend"
        );
        ensure!(result.outputs == expected, "NCCL result mismatch");
        ensure!(
            result
                .network
                .iter()
                .all(|n| n.tx_bytes > 0 && n.rx_bytes > 0),
            "NCCL did not use the physical data interfaces"
        );
        println!("PASS: {}", serde_json::to_string(&result)?);
    }
    let single = Collective::AllReduce {
        inputs: vec![vec![9.0]],
        reduction: Reduction::Sum,
    };
    let result = fabric
        .collective(
            &ranks[..1],
            &single,
            &NcclOptions {
                devices: vec![0],
                ..options.clone()
            },
            &worker,
        )
        .await?;
    ensure!(result.outputs == vec![vec![9.0]], "single-rank NCCL failed");
    ensure!(
        serde_json::to_value(sim)? == before,
        "NCCL changed virtual model time/state"
    );

    let request = Collective::AllReduce {
        inputs: vec![vec![1.0; 1024]; 2],
        reduction: Reduction::Sum,
    };
    // A nonexistent GPU must fail; it must never trigger a CPU or software-verbs path.
    ensure!(
        fabric
            .collective(
                &ranks,
                &request,
                &NcclOptions {
                    devices: vec![0, i32::MAX],
                    timeout_secs: 15
                },
                &worker
            )
            .await
            .is_err(),
        "missing GPU was accepted"
    );
    // Cut only the real kernel link, keeping admission's graph connected. NCCL
    // must encounter the network partition and terminate under its supervisor deadline.
    let dev = fabric.device(&ranks[0])?;
    dev.run_sync(|| {
        ensure!(
            Command::new("ip")
                .args(["link", "set", "eth1", "down"])
                .status()?
                .success(),
            "cut link failed"
        );
        Ok(())
    })?;
    let error = fabric
        .collective(
            &ranks,
            &request,
            &NcclOptions {
                timeout_secs: 5,
                ..options.clone()
            },
            &worker,
        )
        .await;
    ensure!(error.is_err(), "NCCL bypassed the partitioned fabric");
    dev.run_sync(|| {
        ensure!(
            Command::new("ip")
                .args(["link", "set", "eth1", "up"])
                .status()?
                .success(),
            "restore link failed"
        );
        Ok(())
    })?;
    let recovered = fabric
        .collective(&ranks, &request, &options, &worker)
        .await?;
    ensure!(
        recovered.outputs == vec![vec![2.0; 1024]; 2],
        "NCCL did not recover after worker cleanup"
    );
    println!("PASS: native NCCL operations, missing-device error, network partition and recovery");
    Ok(())
}
