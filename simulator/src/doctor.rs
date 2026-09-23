// SPDX-License-Identifier: RPL-1.5
//! Read-only prerequisite discovery. Execution still validates pinned artifacts.
use crate::{Error, Result};
use serde_json::{Value, json};
use std::{fs, path::Path};

pub fn inspect(mode: &str) -> Result<Value> {
    let (compiled, programs, paths): (bool, &[&str], &[&str]) = match mode {
        "portable" => (true, &[], &[]),
        "nccl" => (
            cfg!(all(feature = "nccl", not(feature = "nccl-check"))),
            &["ip", "tc", "nft", "sysctl", "ping", "nvidia-smi"],
            &[],
        ),
        "ibsim" => (
            cfg!(feature = "ibsim"),
            &["ip", "ibsim", "opensm", "ibnetdiscover"],
            &[],
        ),
        "vm" => (
            cfg!(feature = "vm"),
            &[
                "ip",
                "tc",
                "nft",
                "qemu-system-x86_64",
                "qemu-img",
                "ssh",
                "cloud-localds",
            ],
            &["/dev/kvm"],
        ),
        "mokka" => (cfg!(feature = "mokka"), &["kubectl", "helm"], &[]),
        "netbox" => (cfg!(feature = "netbox"), &[], &[]),
        _ => {
            return Err(Error::Invalid(
                "doctor mode: portable|nccl|ibsim|vm|mokka|netbox".into(),
            ));
        }
    };
    let programs: Vec<_> = programs
        .iter()
        .map(|name| {
            let resolved = crate::evidence::resolve(Path::new(name)).ok();
            json!({"name":name,"found":resolved.is_some(),"path":resolved})
        })
        .collect();
    let paths: Vec<_> = paths
        .iter()
        .map(|name| json!({"path":name,"exists":Path::new(name).exists()}))
        .collect();
    let mem_available_kib = fs::read_to_string("/proc/meminfo").ok().and_then(|s| {
        s.lines().find_map(|line| {
            line.strip_prefix("MemAvailable:")
                .and_then(|v| v.split_whitespace().next()?.parse::<u64>().ok())
        })
    });
    let capabilities = fs::read_to_string("/proc/self/status").ok().and_then(|s| {
        s.lines().find_map(|line| {
            line.strip_prefix("CapEff:")
                .and_then(|v| u64::from_str_radix(v.trim(), 16).ok())
        })
    });
    let namespace_privileges =
        capabilities.is_some_and(|c| c & (1 << 12) != 0 && c & (1 << 21) != 0);
    let ok = compiled
        && programs.iter().all(|p| p["found"] == true)
        && paths.iter().all(|p| p["exists"] == true)
        && (!matches!(mode, "nccl" | "ibsim" | "vm") || namespace_privileges);
    Ok(
        json!({"ok":ok,"mode":mode,"feature_compiled":compiled,"programs":programs,"paths":paths,
        "namespace_privileges":namespace_privileges,"mem_available_kib":mem_available_kib,
        "scope":"prerequisite discovery; no driver, library identity, cluster access, VM image or runtime validation",
        "guide":"docs/GETTING_STARTED.md"}),
    )
}
