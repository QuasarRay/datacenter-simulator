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

use datacenter_simulator::deepops::{
    DeepOpsConfig, validate_inventory, validate_nccl, validate_slurm,
};
use serde_json::{Value, json};
use std::path::Path;
fn config() -> DeepOpsConfig {
    DeepOpsConfig::read(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../integrations/deepops/config.example.json"),
    )
    .unwrap()
}
fn report() -> Value {
    // JSON v4 schema from pinned nccl-tests/src/util.cu, including string booleans.
    json!({"version":4,"nccl_version":23102,"end_time":"2026-09-15T00:00:00",
        "args":["/opt/simulator/nccl-tests/build/all_reduce_perf","-c","1"],
        "env":["NCCL_NET=Socket","NCCL_NET_PLUGIN=none","NCCL_SOCKET_IFNAME==simnccl","NCCL_IB_DISABLE=1","NCCL_P2P_DISABLE=1","NCCL_SHM_DISABLE=1","OMPI_MCA_btl_tcp_if_include=simnccl","OMPI_MCA_oob_tcp_if_include=simnccl"],
        "config":{"validation":1,"ngpus":1,"nthreads":1,"devices":[{"rank":0,"hostname":"gpu1","device_info":"NVIDIA GPU"},{"rank":1,"hostname":"gpu2","device_info":"NVIDIA GPU"}]},
        "results":[{"size":256,"out_of_place":{"nwrong":0.0,"time":32.0},"in_place":{"nwrong":0.0,"time":30.0}}],
        "out_of_bounds":{"count":0,"okay":"true"},"errors":[""]})
}
#[test]
fn inventory_contains_only_isolated_guests() {
    let (_, plan) = config().plan().unwrap();
    assert_eq!(plan.guests.len(), 3);
    assert_eq!(plan.guests[0].address.to_string(), "10.253.0.1");
    let hosts: serde_json::Map<_, _> = plan
        .guests
        .iter()
        .map(|g| {
            (
                g.name.clone(),
                json!({"ansible_host":g.address.to_string()}),
            )
        })
        .collect();
    let mut resolved = json!({"_meta":{"hostvars":hosts}});
    validate_inventory(&resolved, &plan.guests).unwrap();
    resolved["_meta"]["hostvars"]["host-machine"] = json!({"ansible_host":"127.0.0.1"});
    assert!(validate_inventory(&resolved, &plan.guests).is_err());
    resolved["_meta"]["hostvars"]
        .as_object_mut()
        .unwrap()
        .remove("host-machine");
    resolved["_meta"]["hostvars"]["gpu1"]["ansible_host"] = json!("10.0.2.15");
    assert!(validate_inventory(&resolved, &plan.guests).is_err());
}
#[test]
fn refuses_device_aliases_and_unsupported_hypercube_size() {
    let mut c = config();
    c.vfio.insert("gpu2".into(), vec!["0000:41:00.0".into()]);
    assert!(c.plan().is_err());
    let mut c = config();
    c.compute.push("gpu3".into());
    assert!(c.plan().is_err());
    let mut c = config();
    c.controller = "gpu1".into();
    assert!(c.plan().is_err());
}
#[test]
fn upstream_nccl_checked_multihost_report_is_accepted() {
    validate_nccl(&report(), "all_reduce", &config().compute).unwrap();
}
#[test]
fn nccl_rejects_skipped_empty_and_corrupt_correctness_reports() {
    let compute = config().compute;
    for (pointer, replacement) in [
        ("/config/validation", json!(0)),
        ("/results", json!([])),
        ("/results/0/in_place/nwrong", json!(1)),
        ("/results/0/out_of_place/nwrong", Value::Null),
        ("/out_of_bounds/okay", json!("false")),
        ("/end_time", Value::Null),
        ("/nccl_version", json!(12345)),
        ("/errors", json!(["unhandled CUDA error"])),
    ] {
        let mut r = report();
        *r.pointer_mut(pointer).unwrap() = replacement;
        assert!(
            validate_nccl(&r, "all_reduce", &compute).is_err(),
            "accepted {pointer}"
        );
    }
}
#[test]
fn nccl_rejects_single_host_and_fabric_bypass() {
    let compute = config().compute;
    let mut r = report();
    r["config"]["devices"][1]["hostname"] = json!("gpu1");
    assert!(validate_nccl(&r, "all_reduce", &compute).is_err());
    let mut r = report();
    r["env"][2] = json!("NCCL_SOCKET_IFNAME=management0");
    assert!(validate_nccl(&r, "all_reduce", &compute).is_err());
    let mut r = report();
    r["config"]["devices"][1]["rank"] = json!(0);
    assert!(validate_nccl(&r, "all_reduce", &compute).is_err());
}
#[test]
fn slurm_requires_a_real_gpu_job_and_every_compute_node() {
    let compute = config().compute;
    let mut r = json!({"ok":true,"gpu_job_ran":true,"gpu_job_ok":true,"nodes_unavailable":0,"gpus_configured":2,"nodes":[{"name":"gpu1"},{"name":"gpu2"}]});
    validate_slurm(&r, &compute).unwrap();
    r["gpu_job_ran"] = json!(false);
    assert!(validate_slurm(&r, &compute).is_err());
    r["gpu_job_ran"] = json!(true);
    r["nodes"][1]["name"] = json!("another-cluster");
    assert!(validate_slurm(&r, &compute).is_err());
}
