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

//! DeepOps configuration and strict validation of upstream test evidence.
use crate::{Error, Result, Simulation, Simulator, manifest::Manifest, model::Role};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

pub const DEEPOPS_COMMIT: &str = "dc80499ea34b3f36563ed039421076de15071517";
pub const NCCL_TESTS_COMMIT: &str = "b4d5beebca8a76cf01335f724d154b9b9d394d96";
pub const NCCL_COMMIT: &str = "16b55c792f902080c38820c9c2bd766337909da3";
pub const COLLECTIVES: &[&str] = &[
    "all_reduce",
    "all_gather",
    "broadcast",
    "reduce_scatter",
    "reduce",
    "alltoall",
    "alltoallv",
    "scatter",
    "gather",
    "sendrecv",
    "hypercube",
];

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeepOpsConfig {
    pub repository: PathBuf,
    pub manifest: PathBuf,
    pub image: PathBuf,
    pub image_sha256: String,
    pub state_dir: PathBuf,
    pub ansible_bin: PathBuf,
    pub provisioner: String,
    pub controller: String,
    pub compute: Vec<String>,
    /// PCI functions already bound to vfio-pci. The simulator never unbinds host devices.
    #[serde(default)]
    pub vfio: BTreeMap<String, Vec<String>>,
    #[serde(default = "deadline")]
    pub timeout_secs: u64,
    #[serde(default = "boot_deadline")]
    pub boot_timeout_secs: u64,
}
fn deadline() -> u64 {
    7200
}
fn boot_deadline() -> u64 {
    600
}
#[derive(Debug, Clone, Serialize)]
pub struct GuestPlan {
    pub name: String,
    pub node_id: String,
    pub address: std::net::Ipv4Addr,
    pub cpus: u32,
    pub memory_mib: u64,
    pub disk_gib: u64,
    pub vfio: Vec<String>,
}
#[derive(Debug, Serialize)]
pub struct DeepOpsPlan {
    pub guests: Vec<GuestPlan>,
    pub provisioner_id: String,
    pub inventory: Value,
    pub deepops_revision: &'static str,
    pub nccl_revision: &'static str,
    pub nccl_tests_revision: &'static str,
}
impl DeepOpsConfig {
    pub fn read(path: &Path) -> Result<Self> {
        let path = std::fs::canonicalize(path)?;
        let parent = path
            .parent()
            .ok_or_else(|| Error::Invalid("config has no parent".into()))?;
        let mut config: Self = serde_json::from_str(&std::fs::read_to_string(&path)?)?;
        for value in [
            &mut config.repository,
            &mut config.manifest,
            &mut config.image,
            &mut config.state_dir,
            &mut config.ansible_bin,
        ] {
            if value.is_relative() {
                *value = parent.join(&*value);
            }
        }
        Ok(config)
    }
    pub fn plan(&self) -> Result<(Simulation, DeepOpsPlan)> {
        if self.compute.len() < 2
            || self.compute.len() > 16
            || !self.compute.len().is_power_of_two()
            || !(1..=14400).contains(&self.timeout_secs)
            || !(1..=1800).contains(&self.boot_timeout_secs)
            || self.image_sha256.len() != 64
            || !self.image_sha256.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(Error::Invalid(
                "require 2, 4, 8 or 16 compute nodes, bounded deadlines and an image SHA-256"
                    .into(),
            ));
        }
        let names: Vec<_> = std::iter::once(&self.controller)
            .chain(self.compute.iter())
            .collect();
        let unique: BTreeSet<_> = names.iter().copied().collect();
        if unique.len() != names.len() || unique.contains(&self.provisioner) {
            return Err(Error::Invalid(
                "controller, compute and provisioner must be distinct".into(),
            ));
        }
        if self.vfio.keys().any(|name| !self.compute.contains(name)) {
            return Err(Error::Invalid(
                "VFIO devices may only be assigned to compute guests".into(),
            ));
        }
        let mut pci = BTreeSet::new();
        for device in self.vfio.values().flatten() {
            if !valid_bdf(device) || !pci.insert(device) {
                return Err(Error::Invalid(
                    "invalid or duplicate VFIO PCI function".into(),
                ));
            }
        }
        for name in names
            .iter()
            .copied()
            .chain(std::iter::once(&self.provisioner))
        {
            if name.starts_with('-')
                || name.ends_with('-')
                || !name
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            {
                return Err(Error::Invalid(
                    "guest/provisioner names must be lowercase DNS hostnames".into(),
                ));
            }
        }
        let mut api = Simulator::new();
        let id = api.import(Manifest::read(&self.manifest)?, true)?;
        let sim = api.get(&id)?;
        let provisioner = sim.node_named(&self.provisioner)?;
        if provisioner.spec.role != Role::Host {
            return Err(Error::Invalid("provisioner must be a host".into()));
        }
        let mut guests = Vec::new();
        for (index, name) in names.iter().enumerate() {
            let node = sim.node_named(name)?;
            let r = &node.spec.resources;
            if node.spec.role != Role::Host
                || r.cpu == 0
                || r.cpu > 64
                || r.memory < 512
                || r.memory > 131072
                || r.storage < 8
                || r.storage > 1024
            {
                return Err(Error::Invalid(
                    "VM nodes need 1..=64 CPUs, 512..=131072 MiB RAM and 8..=1024 GiB disk".into(),
                ));
            }
            sim.route(&provisioner.id, &node.id, 0)?;
            for peer in &names {
                sim.route(&node.id, &sim.node_named(peer)?.id, 0)?;
            }
            guests.push(GuestPlan {
                name: (*name).clone(),
                node_id: node.id.clone(),
                address: std::net::Ipv4Addr::new(10, 253, 0, index as u8 + 1),
                cpus: r.cpu,
                memory_mib: r.memory,
                disk_gib: r.storage,
                vfio: self.vfio.get(*name).cloned().unwrap_or_default(),
            });
        }
        let host = |guest: &GuestPlan| json!({"ansible_host":guest.address.to_string(), "ansible_user":"deepops", "ansible_python_interpreter":"/usr/bin/python3"});
        let controller = BTreeMap::from([(guests[0].name.clone(), host(&guests[0]))]);
        let computes: BTreeMap<_, _> = guests[1..]
            .iter()
            .map(|g| (g.name.clone(), host(g)))
            .collect();
        let inventory = json!({"all":{"children":{"slurm-cluster":{"children":{
            "slurm-master":{"hosts":controller}, "slurm-node":{"hosts":computes},
            "slurm-login":{"hosts":{self.controller.clone():{}}}
        }}}}});
        Ok((
            sim.clone(),
            DeepOpsPlan {
                guests,
                provisioner_id: provisioner.id.clone(),
                inventory,
                deepops_revision: DEEPOPS_COMMIT,
                nccl_revision: NCCL_COMMIT,
                nccl_tests_revision: NCCL_TESTS_COMMIT,
            },
        ))
    }
}
fn valid_bdf(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 12
        && bytes[4] == b':'
        && bytes[7] == b':'
        && bytes[10] == b'.'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| [4, 7, 10].contains(&i) || b.is_ascii_hexdigit())
        && bytes[11] <= b'7'
}
/// Check the inventory Ansible actually resolved, before applying any playbook.
pub fn validate_inventory(value: &Value, guests: &[GuestPlan]) -> Result<()> {
    let hosts = value["_meta"]["hostvars"]
        .as_object()
        .ok_or_else(|| Error::Invalid("Ansible inventory has no hostvars".into()))?;
    if hosts.len() != guests.len() {
        return Err(Error::Invalid(
            "Ansible inventory contains unintended hosts".into(),
        ));
    }
    for guest in guests {
        if hosts
            .get(&guest.name)
            .and_then(|h| h["ansible_host"].as_str())
            != Some(guest.address.to_string().as_str())
        {
            return Err(Error::Invalid(format!(
                "Ansible target mismatch for {}",
                guest.name
            )));
        }
    }
    Ok(())
}
pub fn validate_slurm(value: &Value, compute: &[String]) -> Result<()> {
    let names: BTreeSet<_> = value["nodes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|n| n["name"].as_str())
        .collect();
    let expected: BTreeSet<_> = compute.iter().map(String::as_str).collect();
    if value["ok"] != true
        || value["gpu_job_ok"] != true
        || value["gpu_job_ran"] != true
        || value["nodes_unavailable"] != 0
        || names != expected
        || value["nodes"].as_array().is_none_or(|nodes| {
            nodes.len() != compute.len()
                || nodes
                    .iter()
                    .any(|n| n["gpus_configured"].as_u64().unwrap_or(0) == 0)
        })
        || value["gpus_configured"].as_u64().unwrap_or(0) < compute.len() as u64
    {
        return Err(Error::State(format!(
            "DeepOps GPU validation failed: {value}"
        )));
    }
    Ok(())
}
/// Validate the pinned upstream JSON v4 format (its booleans are strings).
/// Require real multi-host device records and checked data for every result row.
pub fn validate_nccl(value: &Value, collective: &str, compute: &[String]) -> Result<()> {
    let fail = || {
        Error::State(format!(
            "invalid or unsuccessful nccl-tests report for {collective}"
        ))
    };
    if !COLLECTIVES.contains(&collective)
        || value["version"] != 4
        || value["nccl_version"] != 23102
        || value["config"]["validation"].as_u64().unwrap_or(0) == 0
        || value["config"]["ngpus"] != 1
        || value["config"]["nthreads"] != 1
        || value["out_of_bounds"]["count"] != 0
        || value["out_of_bounds"]["okay"] != "true"
        || value["end_time"].as_str().is_none_or(str::is_empty)
    {
        return Err(fail());
    }
    let args = value["args"].as_array().ok_or_else(fail)?;
    if !args
        .first()
        .and_then(Value::as_str)
        .is_some_and(|s| s.ends_with(&format!("/{collective}_perf")))
    {
        return Err(fail());
    }
    let devices = value["config"]["devices"].as_array().ok_or_else(fail)?;
    let hosts: BTreeSet<_> = devices
        .iter()
        .filter_map(|d| d["hostname"].as_str())
        .collect();
    let ranks: BTreeSet<_> = devices.iter().filter_map(|d| d["rank"].as_u64()).collect();
    if devices.len() != compute.len()
        || hosts != compute.iter().map(String::as_str).collect()
        || ranks != (0..compute.len() as u64).collect()
        || devices
            .iter()
            .any(|d| d["device_info"].as_str().is_none_or(str::is_empty))
    {
        return Err(fail());
    }
    let env: BTreeSet<_> = value["env"]
        .as_array()
        .ok_or_else(fail)?
        .iter()
        .filter_map(Value::as_str)
        .collect();
    for required in [
        "NCCL_NET=Socket",
        "NCCL_NET_PLUGIN=none",
        "NCCL_SOCKET_IFNAME==simnccl",
        "NCCL_IB_DISABLE=1",
        "NCCL_P2P_DISABLE=1",
        "NCCL_SHM_DISABLE=1",
        "OMPI_MCA_btl_tcp_if_include=simnccl",
        "OMPI_MCA_oob_tcp_if_include=simnccl",
    ] {
        if !env.contains(required) {
            return Err(fail());
        }
    }
    let rows = value["results"].as_array().ok_or_else(fail)?;
    if rows.is_empty() {
        return Err(fail());
    }
    if ["all_reduce", "reduce", "reduce_scatter"].contains(&collective) {
        let operations: BTreeSet<_> = rows
            .iter()
            .filter_map(|row| row["redop"].as_str())
            .collect();
        if ["sum", "prod", "min", "max", "avg", "mulsum"]
            .iter()
            .any(|op| !operations.contains(op))
        {
            return Err(fail());
        }
    }
    if ["broadcast", "reduce", "scatter", "gather"].contains(&collective) {
        let roots: BTreeSet<u64> = rows
            .iter()
            .filter_map(|row| row["root"].as_str()?.trim().parse().ok())
            .collect();
        if roots != (0..compute.len() as u64).collect() {
            return Err(fail());
        }
    }
    let mut nonzero = false;
    for row in rows {
        nonzero |= row["size"].as_u64().unwrap_or(0) > 0;
        let mut checked = false;
        for kind in ["out_of_place", "in_place"] {
            let result = &row[kind];
            if result.is_null() {
                continue;
            }
            if result["nwrong"].as_f64() != Some(0.0)
                || result["time"]
                    .as_f64()
                    .is_none_or(|t| !t.is_finite() || t < 0.0)
            {
                return Err(fail());
            }
            checked = true;
        }
        if !checked {
            return Err(fail());
        }
    }
    if !nonzero
        || value["errors"]
            .as_array()
            .ok_or_else(fail)?
            .iter()
            .any(|e| e.as_str() != Some(""))
    {
        return Err(fail());
    }
    Ok(())
}
