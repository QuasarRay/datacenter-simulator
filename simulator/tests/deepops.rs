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
fn report_for(collective: &str) -> Value {
    let reduction = ["all_reduce", "reduce", "reduce_scatter"].contains(&collective);
    let rooted = ["reduce", "broadcast", "scatter", "gather"].contains(&collective);
    let split = [
        "all_gather",
        "reduce_scatter",
        "alltoall",
        "alltoallv",
        "scatter",
        "gather",
        "hypercube",
    ]
    .contains(&collective);
    let ops = if reduction {
        vec!["sum", "prod", "min", "max", "avg", "mulsum"]
    } else if collective == "hypercube" {
        vec![""]
    } else if collective == "sendrecv" {
        vec!["sum"]
    } else {
        vec!["none"]
    };
    let roots = if rooted { vec![0, 1] } else { vec![-1] };
    let mut rows = Vec::new();
    for size in [
        256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072, 262144, 524288, 1048576,
    ] {
        for op in &ops {
            for root in &roots {
                rows.push(json!({"redop":op,"root":root.to_string(),"size":size,"count":size / if split {16} else {8},"type":"double",
                "out_of_place":{"nwrong":0.0,"time":32.0},
                "in_place":{"nwrong":if ["alltoall","alltoallv","sendrecv"].contains(&collective) {Value::Null} else {json!(0.0)},"time":30.0}}));
            }
        }
    }
    let mut args = vec![format!("/opt/simulator/nccl-tests/build/{collective}_perf")];
    args.extend(
        "-g 1 -t 1 -b 256 -e 1M -f 2 -n 5 -w 1 -c 1 -d double -T 60"
            .split_whitespace()
            .map(String::from),
    );
    if reduction {
        args.extend(["-o", "all"].map(String::from));
    }
    if rooted {
        args.extend(["-r", "-1"].map(String::from));
    }
    args.extend([
        "-J".into(),
        format!("/var/tmp/simulator-nccl/{collective}.json"),
    ]);
    json!({"version":4,"nccl_version":23102,"end_time":"2026-09-15T00:00:00", "args":args,
        "env":["NCCL_NET=Socket","NCCL_NET_PLUGIN=none","NCCL_SOCKET_IFNAME==simnccl","NCCL_IB_DISABLE=1","NCCL_P2P_DISABLE=1","NCCL_SHM_DISABLE=1","OMPI_MCA_pml=ob1","OMPI_MCA_btl=self,tcp","OMPI_MCA_btl_tcp_if_include=simnccl","OMPI_MCA_oob_tcp_if_include=simnccl"],
        "config":{"validation":1,"ngpus":1,"nthreads":1,"devices":[{"rank":0,"hostname":"gpu1","device_info":"NVIDIA GPU"},{"rank":1,"hostname":"gpu2","device_info":"NVIDIA GPU"}]},
        "results":rows,"out_of_bounds":{"count":0,"okay":"true"},"errors":[""]})
}
fn report() -> Value {
    report_for("all_reduce")
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
    let mut r = json!({"ok":true,"gpu_job_ran":true,"gpu_job_ok":true,"nodes_unavailable":0,"gpus_configured":2,"nodes":[{"name":"gpu1","gpus_configured":1},{"name":"gpu2","gpus_configured":1}]});
    validate_slurm(&r, &compute).unwrap();
    r["gpu_job_ran"] = json!(false);
    assert!(validate_slurm(&r, &compute).is_err());
    r["gpu_job_ran"] = json!(true);
    r["nodes"][1]["name"] = json!("another-cluster");
    assert!(validate_slurm(&r, &compute).is_err());
}

#[test]
fn root_and_reduction_coverage_cannot_be_silently_skipped() {
    let compute = config().compute;
    let mut r = report();
    r["results"].as_array_mut().unwrap().pop();
    assert!(validate_nccl(&r, "all_reduce", &compute).is_err());
    for collective in datacenter_simulator::deepops::COLLECTIVES {
        let r = report_for(collective);
        validate_nccl(&r, collective, &compute).unwrap();
        let rows = r["results"].as_array().unwrap();
        // Remove each grid cell in turn: marginal sets can remain complete.
        for i in 0..rows.len() {
            let mut missing = r.clone();
            missing["results"].as_array_mut().unwrap().remove(i);
            assert!(validate_nccl(&missing, collective, &compute).is_err());
        }
    }
    let mut duplicate = report_for("reduce");
    let row = duplicate["results"][0].clone();
    duplicate["results"].as_array_mut().unwrap().push(row);
    assert!(validate_nccl(&duplicate, "reduce", &compute).is_err());
}
