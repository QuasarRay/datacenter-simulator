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
    if args.first().is_some_and(|s| s.starts_with("incus-")) {
        #[cfg(feature = "incus")]
        {
            use datacenter_simulator::incus::{self, Config};
            use std::path::Path;
            match args[0].as_str() {
                "incus-plan" if args.len() == 2 => println!("{}", serde_json::to_string_pretty(&Config::read(Path::new(&args[1]))?.plan()?)?),
                "incus-up" if args.len() == 3 => incus::up(Config::read(Path::new(&args[1]))?, Path::new(&args[2]))?,
                "incus-down" if args.len() == 2 => incus::down(Path::new(&args[1]))?,
                _ => return Err("usage: incus-plan config.json | incus-up config.json NEW_DIR | incus-down STATE_DIR".into()),
            }
            return Ok(());
        }
        #[cfg(not(feature = "incus"))]
        return Err("Incus requires --features incus".into());
    }
    if args
        .first()
        .is_some_and(|arg| matches!(arg.as_str(), "--help" | "-h" | "help"))
    {
        print_help();
        return Ok(());
    }
    if args.first().map(String::as_str) == Some("doctor") {
        if args.len() > 2 {
            return Err("usage: doctor [portable|nccl|ibsim|vm|mokka|netbox]".into());
        }
        let report =
            datacenter_simulator::doctor::inspect(args.get(1).map_or("portable", String::as_str))?;
        println!("{}", serde_json::to_string_pretty(&report)?);
        if report["ok"] != true {
            return Err("prerequisites missing; see doctor report".into());
        }
        return Ok(());
    }
    if args.first().is_some_and(|s| s.starts_with("netbox-")) {
        #[cfg(feature = "netbox")]
        {
            use datacenter_simulator::netbox::{self, NetBoxConfig, Snapshot};
            if args.len() != 3 {
                return Err("usage: netbox-import config.json NEW_DIR | netbox-compile config.json snapshot.json".into());
            }
            let config: NetBoxConfig = serde_json::from_slice(&datacenter_simulator::input::read(
                &args[1],
                datacenter_simulator::input::CONFIG_LIMIT,
            )?)?;
            match args[0].as_str() {
                "netbox-import" => netbox::import(&config, std::path::Path::new(&args[2]))?,
                "netbox-compile" => {
                    let snapshot: Snapshot =
                        serde_json::from_slice(&datacenter_simulator::input::read(
                            &args[2],
                            datacenter_simulator::input::SNAPSHOT_LIMIT,
                        )?)?;
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&netbox::compile(&config, &snapshot)?)?
                    );
                }
                _ => return Err("unknown NetBox command".into()),
            }
            return Ok(());
        }
        #[cfg(not(feature = "netbox"))]
        return Err("NetBox integration requires --features netbox".into());
    }
    if args.first().is_some_and(|s| s.starts_with("mokka-")) {
        #[cfg(feature = "mokka")]
        {
            use datacenter_simulator::mokka::{self, MokkaConfig};
            if args.len() < 2 {
                return Err("expected a Mokka config path".into());
            }
            let config = MokkaConfig::read(std::path::Path::new(&args[1]))?;
            match args[0].as_str() {
                "mokka-plan" if args.len()==2=>println!("{}",serde_json::to_string_pretty(&config.plan()?)?),
                "mokka-render" if args.len()==3=>{ mokka::render(&config,std::path::Path::new(&args[2]))?; },
                "mokka-apply" if args.len()==3=>mokka::apply(&config,std::path::Path::new(&args[2]))?,
                _=>return Err("usage: mokka-plan config.json | mokka-render config.json NEW_DIR | mokka-apply config.json NEW_DIR".into()),
            }
            return Ok(());
        }
        #[cfg(not(feature = "mokka"))]
        return Err("Mokka integration requires --features mokka".into());
    }
    if args.first().map(String::as_str) == Some("ibsim-plan") {
        #[cfg(feature = "ibsim")]
        {
            if args.len() != 2 {
                return Err("usage: ibsim-plan manifest.json".into());
            }
            let mut api = Simulator::new();
            let id = api.import(Manifest::read(&args[1])?, false)?;
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &datacenter_simulator::infiniband::topology(api.get(&id)?)?.1
                )?
            );
            return Ok(());
        }
        #[cfg(not(feature = "ibsim"))]
        return Err("InfiniBand integration requires --features ibsim".into());
    }
    if args.first().is_some_and(|arg| {
        matches!(
            arg.as_str(),
            "deepops-plan" | "deepops-run" | "deepops-vm-smoke"
        )
    }) {
        if args.len() != 2 {
            return Err("expected a DeepOps JSON config path".into());
        }
        let config =
            datacenter_simulator::deepops::DeepOpsConfig::read(std::path::Path::new(&args[1]))?;
        if args[0] == "deepops-plan" {
            let (_, plan) = config.plan()?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
            return Ok(());
        }
        #[cfg(feature = "vm")]
        return datacenter_simulator::deepops_runtime::run(config, args[0] == "deepops-vm-smoke")
            .map_err(Into::into);
        #[cfg(not(feature = "vm"))]
        return Err("DeepOps execution requires --features vm".into());
    }
    #[cfg(all(feature = "nccl", not(feature = "nccl-check")))]
    {
        if args.first().map(String::as_str) == Some("__nccl_rank") {
            return datacenter_simulator::nccl_worker::run(&args[1..]).map_err(Into::into);
        }
    }
    match args.first().map(String::as_str) {
        Some("run") if args.len() >= 2 && args.len() % 2 == 0 => {
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
            for pair in args[2..].as_chunks::<2>().0 {
                match pair[0].as_str() {
                    "--devices" => {
                        options.devices = pair[1]
                            .split(',')
                            .map(str::parse)
                            .collect::<Result<_, _>>()?
                    }
                    "--libraries" => {
                        options.libraries =
                            Some(serde_json::from_slice(&datacenter_simulator::input::read(
                                &pair[1],
                                datacenter_simulator::input::CONFIG_LIMIT,
                            )?)?)
                    }
                    _ => {
                        return Err(
                            "expected --devices 0,1,... or --libraries pinned-libraries.json"
                                .into(),
                        );
                    }
                }
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
                let mut line = Vec::new();
                if reader
                    .by_ref()
                    .take(1_048_577)
                    .read_until(b'\n', &mut line)?
                    == 0
                {
                    break;
                }
                if line.len() > 1024 * 1024 {
                    // Drain this command in bounded chunks and keep the session usable.
                    while !line.ends_with(b"\n") {
                        line.clear();
                        if reader.by_ref().take(8192).read_until(b'\n', &mut line)? == 0 {
                            break;
                        }
                    }
                    writeln!(
                        out,
                        "{{\"ok\":false,\"error\":\"JSON command exceeds the 1 MiB transport limit; use the Rust API for larger buffers\"}}"
                    )?;
                    out.flush()?;
                    continue;
                }
                match std::str::from_utf8(&line) {
                    Ok(line) if line.trim().is_empty() => continue,
                    Ok(line) => writeln!(out, "{}", api.respond(line))?,
                    Err(_) => writeln!(out, "{{\"ok\":false,\"error\":\"command is not UTF-8\"}}")?,
                }
                out.flush()?;
            }
        }
        _ => {
            return Err(
                "usage: datacenter-simulator [json | run manifest.json [--devices 0,1,...] | deepops-plan config.json | deepops-run config.json | deepops-vm-smoke config.json]".into(),
            );
        }
    }
    Ok(())
}

fn print_help() {
    println!("datacenter-simulator
  json                              JSON-lines command API
  run MANIFEST [--devices 0,1,...] --libraries LIBRARIES.json
                                    Native NCCL; pinned absolute CUDA/NCCL paths and SHA-256 required
  deepops-plan CONFIG               Validate a VM/GPU deployment plan
  deepops-run CONFIG                Deploy and validate DeepOps (vm feature)
  deepops-vm-smoke CONFIG           Test the VM fabric (vm feature)
  netbox-import CONFIG NEW_DIR      Fetch and compile NetBox intent (netbox feature)
  netbox-compile CONFIG SNAPSHOT    Replay a checked snapshot (netbox feature)
  mokka-plan CONFIG                 Inspect mock GPU contracts (mokka feature)
  mokka-render CONFIG NEW_DIR       Render Mokka values (mokka feature)
  mokka-apply CONFIG NEW_DIR        Apply to explicitly opted-in CPU nodes (mokka feature)
  ibsim-plan MANIFEST               Inspect IB management topology (ibsim feature)
  doctor [MODE]                    Read-only prerequisite discovery (default: portable)
  --help                           Show this help
See simulator/README.md and docs/GETTING_STARTED.md for prerequisites and fidelity limits.");
    println!(
        "Compiled features: linux={} nccl={} nccl-check={} netbox={} mokka={} ibsim={} vm={}",
        cfg!(feature = "linux"),
        cfg!(feature = "nccl"),
        cfg!(feature = "nccl-check"),
        cfg!(feature = "netbox"),
        cfg!(feature = "mokka"),
        cfg!(feature = "ibsim"),
        cfg!(feature = "vm")
    );
}
