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

use datacenter_simulator::{
    Simulator,
    collective::{Collective, NcclOptions, Reduction},
    manifest::Manifest,
    model::Role,
};
use std::io::{self, BufRead, Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    #[cfg(all(feature = "nccl", not(feature = "nccl-check")))]
    {
        if args.first().map(String::as_str) == Some("__nccl_rank") {
            return datacenter_simulator::nccl_worker::run(&args[1..]).map_err(Into::into);
        }
        // Namespace initialization must happen before patchbay/Tokio/CUDA threads.
        patchbay::init_userns()?;
    }
    match args.first().map(String::as_str) {
        Some("run") if args.len() == 2 || args.len() == 4 => {
            let mut api = Simulator::new();
            let id = api.import(Manifest::read(&args[1])?, true)?;
            let sim = api.get_mut(&id)?;
            let mut ranks: Vec<_> = sim
                .nodes()
                .filter(|n| n.spec.role == Role::Host)
                .map(|n| (n.spec.name.clone(), n.id.clone()))
                .collect();
            ranks.sort();
            let ids: Vec<_> = ranks.iter().map(|(_, id)| id.clone()).collect();
            let inputs: Vec<_> = (1..=ids.len()).map(|i| vec![i as f64; 8]).collect();
            let mut options = NcclOptions::default();
            if args.len() == 4 {
                if args[2] != "--devices" {
                    return Err("expected --devices 0,1,...".into());
                }
                options.devices = args[3]
                    .split(',')
                    .map(str::parse)
                    .collect::<Result<_, _>>()?;
            }
            let result = sim.collective(
                &ids,
                &Collective::AllReduce {
                    inputs,
                    reduction: Reduction::Sum,
                },
                &options,
            )?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({"ranks":ranks,"result":result}))?
            );
        }
        None | Some("json") => {
            let mut api = Simulator::new();
            let stdin = io::stdin();
            let mut reader = stdin.lock();
            let mut out = io::stdout().lock();
            loop {
                let mut line = String::new();
                if reader.by_ref().take(1_048_577).read_line(&mut line)? == 0 {
                    break;
                }
                if line.len() > 1024 * 1024 {
                    return Err("command exceeds 1 MiB".into());
                }
                if line.trim().is_empty() {
                    continue;
                }
                writeln!(out, "{}", api.respond(&line))?;
                out.flush()?;
            }
        }
        _ => {
            return Err(
                "usage: datacenter-simulator [json | run manifest.json [--devices 0,1,...]]".into(),
            );
        }
    }
    Ok(())
}
