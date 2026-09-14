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

use datacenter_simulator::{Simulator, collective::Reduction, manifest::Manifest, model::Role};
use std::io::{self, BufRead, Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("run") if args.len() == 2 => {
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
            let result = sim.all_reduce(&ids, &inputs, Reduction::Sum)?;
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &serde_json::json!({"backend":"portable","ranks":ranks,"result":result})
                )?
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
        _ => return Err("usage: datacenter-simulator [json | run manifest.json]".into()),
    }
    Ok(())
}
