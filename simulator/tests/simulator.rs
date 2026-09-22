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
    Simulation, Simulator,
    collective::Reduction,
    fabric::{Fabric, Opcode},
    manifest::Manifest,
    model::*,
};
use proptest::prelude::*;
use std::collections::BTreeMap;

fn star(n: usize) -> (Simulator, String, Vec<String>, Vec<String>) {
    let mut api = Simulator::new();
    let id = api.create("test").unwrap().id().to_string();
    let sim = api.get_mut(&id).unwrap();
    let mut switch = NodeSpec::host("switch");
    switch.role = Role::Switch;
    let switch = sim.create_node(switch).unwrap();
    let mut ranks = Vec::new();
    let mut links = Vec::new();
    for i in 0..n {
        let host = sim.create_node(NodeSpec::host(&format!("gpu{i}"))).unwrap();
        let a = sim
            .create_interface(&host, "eth0", InterfaceType::Data)
            .unwrap();
        let b = sim
            .create_interface(&switch, &format!("swp{i}"), InterfaceType::Data)
            .unwrap();
        links.push(
            sim.create_link(
                [&a, &b],
                LinkSpec {
                    bandwidth_bps: 8_000_000_000,
                    latency_ns: 100,
                    up: true,
                },
            )
            .unwrap(),
        );
        ranks.push(host);
    }
    sim.start(None).unwrap();
    (api, id, ranks, links)
}
fn snapshot(sim: &Simulation) -> serde_json::Value {
    serde_json::to_value(sim).unwrap()
}

#[test]
fn lifecycle_is_strict_and_failed_edits_are_atomic() {
    let (mut api, id, ranks, _) = star(2);
    let sim = api.get_mut(&id).unwrap();
    let before = snapshot(sim);
    assert!(sim.create_node(NodeSpec::host("late")).is_err());
    assert!(sim.start(None).is_err());
    assert_eq!(snapshot(sim), before);
    assert!(
        sim.reset_nodes(&[ranks[0].clone(), "missing".into()], false)
            .is_err()
    );
    assert_eq!(snapshot(sim), before);
    sim.shutdown(false).unwrap();
    sim.wait_for_state(State::Inactive).unwrap();
    assert!(sim.transfer(&ranks[0], &ranks[1], 1).is_err());
    sim.start(None).unwrap();
    sim.wait_for_state(State::Active).unwrap();
}
#[test]
fn invalid_import_never_leaks_a_simulation() {
    let mut api = Simulator::new();
    let manifest=Manifest::from_json(r#"{"format":"JSON","name":"bad","content":{"nodes":{"a":{"os":"test"}},"links":[{"endpoints":["a:e0","missing:e0"]}]}}"#).unwrap();
    assert!(api.import(manifest, false).is_err());
    assert!(api.list(None, None, 0, 100).is_empty());
    assert!(
        Manifest::from_json(r#"{"format":"JSON","name":"bad","content":{"nodes":{},"typo":true}}"#)
            .is_err()
    );
}
#[test]
fn sdk_example_envelope_and_string_content_import() {
    let mut api = Simulator::new();
    let m=Manifest::from_json(r#"{"format":"JSON","name":"JSON Sim","ztp":null,"content":{"nodes":{"node1":{"cpu":2,"memory":1024,"storage":10,"os":"generic/ubuntu2204","positioning":{"x":0,"y":0},"features":{"uefi":false,"tpm":false},"pxehost":false,"secureboot":false,"oob":false,"network_pci":{}}},"links":[],"oob":false}}"#).unwrap();
    let id = api.import(m, false).unwrap();
    let output = api.get(&id).unwrap().export().unwrap();
    assert_eq!(
        output.content.nodes["node1"].image_metadata["features"]["uefi"],
        false
    );
    let escaped = serde_json::json!({"format":"JSON","name":"string","content":r#"{"nodes":{"a":{"os":"test"}}}"#});
    assert!(Manifest::from_json(&escaped.to_string()).is_ok());
}
#[test]
fn export_import_preserves_edges_resources_and_breakout() {
    let (mut api, id, ranks, _) = star(3);
    api.get_mut(&id).unwrap().shutdown(false).unwrap();
    let sim = api.get_mut(&id).unwrap();
    let unused = sim
        .create_interface(&ranks[0], "spare", InterfaceType::Data)
        .unwrap();
    sim.breakout(&unused, 4).unwrap();
    let expected = serde_json::to_value(sim.export().unwrap()).unwrap();
    let new_id = api
        .import(api.get(&id).unwrap().export().unwrap(), false)
        .unwrap();
    assert_eq!(
        serde_json::to_value(api.get(&new_id).unwrap().export().unwrap()).unwrap(),
        expected
    );
    assert_ne!(
        api.get(&id).unwrap().node_named("gpu0").unwrap().id,
        api.get(&new_id).unwrap().node_named("gpu0").unwrap().id
    );
}
#[test]
fn endpoints_cannot_be_reused_or_cross_simulation() {
    let (mut api, id, ranks, _) = star(2);
    let sim = api.get_mut(&id).unwrap();
    sim.shutdown(false).unwrap();
    let a = sim.interface_named(&ranks[0], "eth0").unwrap().id.clone();
    let b = sim.interface_named(&ranks[1], "eth0").unwrap().id.clone();
    let before = snapshot(sim);
    assert!(sim.create_link([&a, &b], LinkSpec::default()).is_err());
    assert!(
        sim.create_link([&a, "foreign"], LinkSpec::default())
            .is_err()
    );
    assert_eq!(snapshot(sim), before);
}
#[test]
fn breakout_in_use_rejected_and_revert_removes_children() {
    let (mut api, id, ranks, _) = star(2);
    let sim = api.get_mut(&id).unwrap();
    sim.shutdown(false).unwrap();
    let connected = sim.interface_named(&ranks[0], "eth0").unwrap().id.clone();
    assert!(sim.breakout(&connected, 4).is_err());
    let parent = sim
        .create_interface(&ranks[0], "spare", InterfaceType::Data)
        .unwrap();
    let children = sim.breakout(&parent, 4).unwrap();
    assert!(sim.delete_interface(&children[0]).is_err());
    sim.revert_breakout(&parent).unwrap();
    assert!(children.iter().all(|id| sim.interface(id).is_err()));
}
#[test]
fn partition_does_not_use_oob_as_a_shortcut() {
    let (mut api, id, ranks, links) = star(2);
    let sim = api.get_mut(&id).unwrap();
    sim.shutdown(false).unwrap();
    sim.set_auto_oob(true, true).unwrap();
    sim.start(None).unwrap();
    let condition = LinkSpec {
        up: false,
        ..LinkSpec::default()
    };
    sim.set_link(&links[0], condition).unwrap();
    assert!(matches!(
        sim.transfer(&ranks[0], &ranks[1], 10),
        Err(Error::NoRoute(..))
    ));
    assert!(sim.node(&ranks[0]).unwrap().management_ip.is_some());
}
#[test]
fn hosts_are_not_transit_routers() {
    let (mut api, id, ranks, _) = star(2);
    let sim = api.get_mut(&id).unwrap();
    sim.shutdown(false).unwrap();
    let node = sim.node_named("switch").unwrap().clone();
    let mut spec = node.spec;
    spec.role = Role::Host;
    sim.update_node(&node.id, spec).unwrap();
    sim.start(None).unwrap();
    assert!(sim.route(&ranks[0], &ranks[1], 8).is_err());
}
#[test]
fn link_serialization_contention_and_full_duplex() {
    let (mut api, id, ranks, _) = star(2);
    let sim = api.get_mut(&id).unwrap();
    let first = sim
        .schedule_transfer(&ranks[0], &ranks[1], 1000, 0)
        .unwrap();
    assert_eq!(first.completed_ns, 2200);
    let second = sim
        .schedule_transfer(&ranks[0], &ranks[1], 1000, 0)
        .unwrap();
    assert_eq!(second.completed_ns, 3200);
    let reverse = sim
        .schedule_transfer(&ranks[1], &ranks[0], 1000, 0)
        .unwrap();
    assert_eq!(reverse.completed_ns, 2200);
}
#[test]
fn parallel_links_and_fault_rerouting() {
    let (mut api, id, ranks, links) = star(2);
    let sim = api.get_mut(&id).unwrap();
    sim.shutdown(false).unwrap();
    let switch = sim.node_named("switch").unwrap().id.clone();
    let a = sim
        .create_interface(&ranks[0], "eth1", InterfaceType::Data)
        .unwrap();
    let b = sim
        .create_interface(&switch, "swp99", InterfaceType::Data)
        .unwrap();
    let parallel = sim
        .create_link(
            [&a, &b],
            LinkSpec {
                bandwidth_bps: 1_000_000,
                latency_ns: 10000,
                up: true,
            },
        )
        .unwrap();
    sim.start(None).unwrap();
    assert_eq!(sim.route(&ranks[0], &ranks[1], 1000).unwrap()[0], links[0]);
    let off = LinkSpec {
        up: false,
        ..LinkSpec::default()
    };
    sim.set_link(&links[0], off).unwrap();
    assert_eq!(sim.route(&ranks[0], &ranks[1], 1000).unwrap()[0], parallel);
}
#[test]
fn disconnected_collective_is_rejected_without_mutation() {
    let (mut api, id, ranks, links) = star(3);
    let sim = api.get_mut(&id).unwrap();
    sim.set_link(
        &links[2],
        LinkSpec {
            up: false,
            ..LinkSpec::default()
        },
    )
    .unwrap();
    let before = snapshot(sim);
    assert!(matches!(
        sim.all_reduce(&ranks, &vec![vec![1.0; 7]; 3], Reduction::Sum),
        Err(Error::NoRoute(..))
    ));
    assert_eq!(snapshot(sim), before);
}
#[test]
fn checkpoints_capture_guest_state_and_clone_has_fresh_ids() {
    let (mut api, id, ranks, _) = star(1);
    let sim = api.get_mut(&id).unwrap();
    sim.shutdown(false).unwrap();
    sim.create_instruction(
        &ranks[0],
        InstructionData::File {
            files: BTreeMap::from([("/etc/test".into(), "before".into())]),
        },
        false,
    )
    .unwrap();
    sim.start(None).unwrap();
    let cp = sim.shutdown(true).unwrap().unwrap();
    sim.rebuild(None).unwrap();
    assert!(sim.runtime(&ranks[0]).unwrap().files.is_empty());
    sim.start(Some(&cp)).unwrap();
    assert_eq!(sim.runtime(&ranks[0]).unwrap().files["/etc/test"], "before");
    let cloned = api.clone_simulation(&id, "clone", Some(&cp), true).unwrap();
    let node = api.get(&cloned).unwrap().node_named("gpu0").unwrap();
    assert_ne!(node.id, ranks[0]);
    assert_eq!(
        api.get(&cloned).unwrap().runtime(&node.id).unwrap().files["/etc/test"],
        "before"
    );
}
#[test]
fn unsupported_guest_code_is_never_reported_as_executed() {
    let (mut api, id, ranks, _) = star(1);
    let sim = api.get_mut(&id).unwrap();
    sim.shutdown(false).unwrap();
    assert!(matches!(
        sim.create_instruction(
            &ranks[0],
            InstructionData::Shell {
                commands: vec!["touch /tmp/x".into()]
            },
            true
        ),
        Err(Error::Unsupported(_))
    ));
    sim.create_ztp_script("echo boot".into()).unwrap();
    assert!(matches!(sim.start(None), Err(Error::Unsupported(_))));
    assert_eq!(sim.state(), State::Inactive);
}
#[test]
fn delete_cascades_and_services_are_immutable_descriptors() {
    let (mut api, id, ranks, _) = star(2);
    let sim = api.get_mut(&id).unwrap();
    sim.shutdown(false).unwrap();
    let i = sim.interface_named(&ranks[0], "eth0").unwrap().id.clone();
    sim.create_service(&i, "ssh", 22, ServiceType::SSH).unwrap();
    assert!(sim.services().next().unwrap().worker_port.is_none());
    assert!(
        sim.create_service(&i, "other", 22, ServiceType::SSH)
            .is_err()
    );
    sim.delete_node(&ranks[0]).unwrap();
    assert_eq!(sim.links().count(), 1);
    assert_eq!(sim.services().count(), 0);
    assert!(sim.interfaces().all(|i| {
        i.connection
            .as_ref()
            .is_none_or(|id| sim.interface(id).is_ok())
    }));
}
#[test]
fn software_verbs_transfer_and_protection() {
    let (mut api, id, ranks, _) = star(2);
    let sim = api.get_mut(&id).unwrap();
    let mut f = Fabric::new(sim, 4).unwrap();
    let a = f.create_qp(sim, &ranks[0]).unwrap();
    let b = f.create_qp(sim, &ranks[1]).unwrap();
    f.connect(sim, &a, &b).unwrap();
    let src = f.register(sim, &ranks[0], vec![1, 2, 3, 4], false).unwrap();
    let dst = f.register(sim, &ranks[1], vec![0; 8], true).unwrap();
    assert!(f.send(sim, &a, &src, 0..4, 1).is_err());
    assert_eq!(sim.clock_ns(), 0);
    f.post_receive(sim, &b, &dst, 2..6, 99).unwrap();
    assert!(f.deregister(&dst).is_err());
    f.send(sim, &a, &src, 0..4, 1).unwrap();
    assert_eq!(f.bytes(sim, &dst).unwrap(), [0, 0, 1, 2, 3, 4, 0, 0]);
    assert_eq!(f.poll(sim, &a).unwrap().unwrap().opcode, Opcode::Send);
    assert_eq!(f.poll(sim, &b).unwrap().unwrap().wr_id, 99);
    let before = f.bytes(sim, &dst).unwrap().to_vec();
    let clock = sim.clock_ns();
    assert!(f.write(sim, &a, &src, 0..4, &dst, 0, "bad-key", 2).is_err());
    assert_eq!(f.bytes(sim, &dst).unwrap(), before);
    assert_eq!(sim.clock_ns(), clock);
    let key = f.remote_key(sim, &dst).unwrap().to_string();
    f.write(sim, &a, &src, 0..4, &dst, 0, &key, 2).unwrap();
    assert_eq!(&f.bytes(sim, &dst).unwrap()[..4], &[1, 2, 3, 4]);
    sim.reset_nodes(&[ranks[0].clone()], false).unwrap();
    assert!(f.bytes(sim, &src).is_err());
    assert!(f.poll(sim, &a).is_err());
}
#[test]
fn verbs_queue_limits_and_cross_domain_access_are_checked() {
    let (mut api, id, ranks, _) = star(2);
    let sim = api.get_mut(&id).unwrap();
    let mut f = Fabric::new(sim, 1).unwrap();
    let a = f.create_qp(sim, &ranks[0]).unwrap();
    let b = f.create_qp(sim, &ranks[1]).unwrap();
    f.connect(sim, &a, &b).unwrap();
    let src = f.register(sim, &ranks[0], vec![7; 4], false).unwrap();
    let dst = f.register(sim, &ranks[1], vec![0; 4], false).unwrap();
    assert!(f.post_receive(sim, &b, &src, 0..4, 1).is_err());
    assert!(f.post_receive(sim, &b, &dst, 0..5, 1).is_err());
    f.post_receive(sim, &b, &dst, 0..4, 1).unwrap();
    assert!(f.post_receive(sim, &b, &dst, 0..4, 2).is_err());
    f.send(sim, &a, &src, 0..4, 1).unwrap();
    f.post_receive(sim, &b, &dst, 0..4, 2).unwrap();
    assert!(f.send(sim, &a, &src, 0..4, 2).is_err());
    f.poll(sim, &a).unwrap();
    f.poll(sim, &b).unwrap();
    f.send(sim, &a, &src, 0..4, 2).unwrap();
}
#[cfg(any(not(feature = "nccl"), feature = "nccl-check"))]
#[test]
fn every_collective_requires_native_execution_including_empty_and_single_rank() {
    let (api, id, ranks, _) = star(2);
    let sim = api.get(&id).unwrap();
    let before = snapshot(sim);
    for result in [
        sim.all_reduce(&ranks, &[vec![1.0], vec![2.0]], Reduction::Sum),
        sim.broadcast(&ranks, 1, &[4.0]),
        sim.all_gather(&ranks, &[vec![1.0], vec![2.0]]),
        sim.reduce_scatter(&ranks, &[vec![1.0, 2.0], vec![3.0, 4.0]], Reduction::Sum),
        sim.all_reduce(&ranks[..1], &[vec![]], Reduction::Sum),
        sim.all_gather(&ranks, &[vec![], vec![]]),
    ] {
        assert!(matches!(result, Err(Error::Unsupported(_))));
    }
    assert_eq!(snapshot(sim), before);
}
#[test]
fn collective_shapes_and_device_placement_are_validated_before_execution() {
    use datacenter_simulator::collective::{Collective, NcclOptions};
    assert!(
        Collective::AllGather {
            inputs: vec![vec![1.0], vec![]]
        }
        .shape(2)
        .is_err()
    );
    assert!(
        Collective::ReduceScatter {
            inputs: vec![vec![1.0; 3]; 2],
            reduction: Reduction::Sum
        }
        .shape(2)
        .is_err()
    );
    assert!(
        Collective::Broadcast {
            root: 2,
            input: vec![1.0]
        }
        .shape(2)
        .is_err()
    );
    assert!(
        Collective::AllReduce {
            inputs: vec![vec![f64::NAN]],
            reduction: Reduction::Sum
        }
        .shape(1)
        .is_err()
    );
    assert!(
        Collective::AllGather {
            inputs: vec![vec![1.0; 1_048_577]; 3]
        }
        .shape(3)
        .is_err()
    );
    for devices in [vec![0, 0], vec![-1, 1], vec![0]] {
        assert!(
            NcclOptions {
                devices,
                ..NcclOptions::default()
            }
            .devices_for(2)
            .is_err()
        );
    }
    assert!(
        NcclOptions {
            timeout_secs: 0,
            ..NcclOptions::default()
        }
        .devices_for(2)
        .is_err()
    );
}
#[cfg(any(not(feature = "nccl"), feature = "nccl-check"))]
#[test]
fn json_collectives_never_return_cpu_results() {
    let (mut api, id, ranks, _) = star(2);
    for fields in [
        serde_json::json!({"operation":"all_reduce", "inputs":[[1.0],[2.0]],"reduction":"sum"}),
        serde_json::json!({"operation":"broadcast", "root":1,"input":[4.0]}),
        serde_json::json!({"operation":"all_gather", "inputs":[[1.0],[2.0]]}),
        serde_json::json!({"operation":"reduce_scatter", "inputs":[[1.0,2.0],[3.0,4.0]],"reduction":"sum"}),
    ] {
        let mut command = fields;
        command["simulation"] = id.clone().into();
        command["ranks"] = serde_json::to_value(&ranks).unwrap();
        let response = api.respond(&command.to_string());
        assert_eq!(response["ok"], false);
        assert!(
            response["error"]
                .as_str()
                .unwrap()
                .contains("native --features nccl")
        );
        assert!(response.get("result").is_none());
    }
}
#[cfg(any(not(feature = "nccl"), feature = "nccl-check"))]
#[test]
fn scenario_cli_fails_without_native_nccl() {
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_datacenter-simulator"))
        .args(["run", "examples/gpu-spine-leaf.json"])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("native --features nccl"));
    assert!(result.stdout.is_empty());
}
#[test]
fn schedules_shutdown_and_expire_only_the_due_simulations() {
    let (mut api, id, _, _) = star(1);
    let now = chrono::Utc::now();
    api.get_mut(&id)
        .unwrap()
        .set_schedule(Some(now), Some(now + chrono::Duration::seconds(10)))
        .unwrap();
    assert!(api.tick(now).unwrap().is_empty());
    assert_eq!(api.get(&id).unwrap().state(), State::Inactive);
    assert_eq!(
        api.tick(now + chrono::Duration::seconds(10)).unwrap(),
        vec![id.clone()]
    );
    assert!(api.get(&id).is_err());
}
#[test]
fn json_commands_reject_unknown_fields_and_preserve_session() {
    let mut api = Simulator::new();
    assert_eq!(
        api.respond(r#"{"operation":"create","name":"x","typo":true}"#)["ok"],
        false
    );
    let created = api.respond(r#"{"operation":"create","name":"x"}"#);
    assert_eq!(created["ok"], true);
    assert_eq!(
        api.respond(r#"{"operation":"list"}"#)["result"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(96))]
    #[test]
    fn invalid_verbs_range_never_changes_memory(end in 5usize..usize::MAX) {
        let (mut api,id,ranks,_)=star(2);let sim=api.get_mut(&id).unwrap();let mut f=Fabric::new(sim,1).unwrap();
        let a=f.create_qp(sim,&ranks[0]).unwrap();let b=f.create_qp(sim,&ranks[1]).unwrap();f.connect(sim,&a,&b).unwrap();
        let src=f.register(sim,&ranks[0],vec![1;4],false).unwrap();
        prop_assert!(f.send(sim,&a,&src,0..end,1).is_err());prop_assert_eq!(sim.clock_ns(),0);
        prop_assert_eq!(f.bytes(sim,&src).unwrap(),&[1,1,1,1]);
    }
}

#[test]
fn unrepresentable_path_duration_returns_error_without_clock_mutation() {
    let mut api = Simulator::new();
    let id = api.create("slow").unwrap().id().to_string();
    let sim = api.get_mut(&id).unwrap();
    let mut nodes = Vec::new();
    for i in 0..142 {
        let mut node = NodeSpec::host(&format!("n{i}"));
        node.role = Role::Switch;
        nodes.push(sim.create_node(node).unwrap());
    }
    for pair in nodes.windows(2) {
        let a = sim
            .create_interface(&pair[0], "out", InterfaceType::Data)
            .unwrap();
        let b = sim
            .create_interface(&pair[1], "in", InterfaceType::Data)
            .unwrap();
        sim.create_link(
            [&a, &b],
            LinkSpec {
                bandwidth_bps: 1,
                latency_ns: 0,
                up: true,
            },
        )
        .unwrap();
    }
    sim.start(None).unwrap();
    assert!(matches!(
        sim.transfer(&nodes[0], &nodes[141], 16 * 1024 * 1024),
        Err(Error::Invalid(_))
    ));
    assert_eq!(sim.clock_ns(), 0);
}

#[test]
fn audit_monotonic_scheduler_and_reset_cancel_reservations() {
    let (mut api, id, ranks, _) = star(2);
    let sim = api.get_mut(&id).unwrap();
    sim.schedule_transfer(&ranks[0], &ranks[1], 1000, 10_000)
        .unwrap();
    assert!(matches!(
        sim.schedule_transfer(&ranks[0], &ranks[1], 1000, 1),
        Err(Error::Invalid(_))
    ));
    sim.reset_nodes(&[ranks[0].clone()], false).unwrap();
    let transfer = sim
        .schedule_transfer(&ranks[0], &ranks[1], 1000, 0)
        .unwrap();
    assert_eq!(transfer.hops[0].start_ns, 0);
}

#[test]
fn audit_checkpoint_rejects_every_configuration_change_atomically() {
    for change in 0..6 {
        let (mut api, id, ranks, links) = star(2);
        let sim = api.get_mut(&id).unwrap();
        let cp = sim.shutdown(true).unwrap().unwrap();
        match change {
            0 => {
                sim.create_node(NodeSpec::host("later")).unwrap();
            }
            1 => {
                sim.create_instruction(
                    &ranks[0],
                    InstructionData::File {
                        files: BTreeMap::from([("/late".into(), "late".into())]),
                    },
                    false,
                )
                .unwrap();
            }
            2 => {
                sim.create_interface(&ranks[0], "extra", InterfaceType::Data)
                    .unwrap();
            }
            3 => {
                let mut spec = sim.node(&ranks[0]).unwrap().spec.clone();
                spec.resources.cpu += 1;
                sim.update_node(&ranks[0], spec).unwrap();
            }
            4 => {
                sim.set_link(
                    &links[0],
                    LinkSpec {
                        up: false,
                        ..LinkSpec::default()
                    },
                )
                .unwrap();
            }
            _ => {
                let iface = sim.interface_named(&ranks[0], "eth0").unwrap().id.clone();
                sim.create_service(&iface, "ssh", 22, ServiceType::SSH)
                    .unwrap();
            }
        }
        let before = snapshot(sim);
        assert!(
            matches!(sim.start(Some(&cp)), Err(Error::Conflict(_))),
            "case {change}"
        );
        assert_eq!(snapshot(sim), before);
        assert!(matches!(
            api.clone_simulation(&id, "bad clone", Some(&cp), true),
            Err(Error::Conflict(_))
        ));
        assert_eq!(api.list(None, None, 0, 100).len(), 1);
    }
}

#[test]
fn audit_pending_checkpoint_instruction_is_not_falsely_complete() {
    let mut api = Simulator::new();
    let id = api.create("pending").unwrap().id().to_string();
    let sim = api.get_mut(&id).unwrap();
    let node = sim.create_node(NodeSpec::host("a")).unwrap();
    sim.create_instruction(
        &node,
        InstructionData::Init {
            hostname: "applied".into(),
        },
        false,
    )
    .unwrap();
    let cp = sim.create_checkpoint("before-start").unwrap();
    sim.start(None).unwrap();
    sim.shutdown(false).unwrap();
    sim.rebuild(Some(&cp)).unwrap();
    assert_eq!(sim.instructions().next().unwrap().state, "PENDING");
    let clone = api.clone_simulation(&id, "clone", Some(&cp), true).unwrap();
    let sim = api.get(&clone).unwrap();
    assert_eq!(
        sim.runtime(&sim.node_named("a").unwrap().id)
            .unwrap()
            .hostname,
        "applied"
    );
}

#[test]
fn audit_addresses_macs_and_quotas_are_stable() {
    let (mut api, id, _, _) = star(2);
    let sim = api.get_mut(&id).unwrap();
    sim.shutdown(false).unwrap();
    sim.set_auto_oob(true, true).unwrap();
    let leases: BTreeMap<_, _> = sim
        .nodes()
        .map(|n| (n.spec.name.clone(), n.management_ip.clone()))
        .collect();
    sim.create_node(NodeSpec::host("aaa-new")).unwrap();
    for (name, ip) in &leases {
        assert_eq!(&sim.node_named(name).unwrap().management_ip, ip);
    }
    let macs: std::collections::BTreeSet<_> = sim.interfaces().map(|i| &i.mac_address).collect();
    assert_eq!(macs.len(), sim.interfaces().count());
    sim.set_history_limit(3);
    for _ in 0..12 {
        sim.update(None, Some(Some("x".into()))).unwrap();
    }
    assert_eq!(sim.history().len(), 3);
    let mut limited = Simulator::with_capacity_limit(1);
    limited.create("one").unwrap();
    assert!(limited.create("two").is_err());
    assert!(
        limited
            .import(api.get(&id).unwrap().export().unwrap(), false)
            .is_err()
    );
}

#[test]
fn audit_manifest_typos_are_rejected() {
    for content in [
        serde_json::json!({"nodes":{"a":{"os":"x","cpus":4}}}),
        serde_json::json!({"nodes":{"a":{"os":"x","interfaces":{"eth0":{}}},"b":{"os":"x","interfaces":{"eth0":{}}}},"links":[{"endpoints":["a:et0","b:eth0"]}]}),
    ] {
        let manifest = Manifest::from_json(
            &serde_json::json!({"format":"JSON","name":"typo","content":content}).to_string(),
        )
        .unwrap();
        let mut api = Simulator::new();
        assert!(api.import(manifest, false).is_err());
        assert!(api.list(None, None, 0, 10).is_empty());
    }
}

#[test]
fn audit_native_queue_accounts_for_high_bdp() {
    let link = LinkSpec {
        bandwidth_bps: 400_000_000_000,
        latency_ns: 1_000_000_000,
        up: true,
    };
    assert_eq!(link.native_queue_packets().unwrap(), 66_667_692);
    assert!(
        LinkSpec {
            bandwidth_bps: u64::MAX,
            latency_ns: 1_000_000_000_000,
            up: true
        }
        .native_queue_packets()
        .is_err()
    );
}

#[cfg(any(not(feature = "nccl"), feature = "nccl-check"))]
#[test]
fn audit_collective_admission_scales_without_reconstructing_all_pairs() {
    let (api, id, ranks, _) = star(1024);
    let start = std::time::Instant::now();
    let result =
        api.get(&id)
            .unwrap()
            .all_reduce(&ranks, &vec![vec![]; ranks.len()], Reduction::Sum);
    assert!(matches!(result, Err(Error::Unsupported(_))));
    assert!(start.elapsed() < std::time::Duration::from_secs(10));
}

#[test]
fn audit_json_oversize_and_invalid_utf8_do_not_destroy_session() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let mut child = Command::new(env!("CARGO_BIN_EXE_datacenter-simulator"))
        .arg("json")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    input.write_all(&vec![b'x'; 1024 * 1024]).unwrap();
    input.write_all("é\n".as_bytes()).unwrap();
    input
        .write_all(b"\xff\n{\"operation\":\"create\",\"name\":\"still-alive\"}\n")
        .unwrap();
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let rows: Vec<serde_json::Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0]["ok"], false);
    assert_eq!(rows[1]["ok"], false);
    assert_eq!(rows[2]["ok"], true);
}

#[test]
fn audit_manifest_size_is_checked_before_deserialization() {
    let path = std::env::temp_dir().join(format!("oversize-manifest-{}", uuid::Uuid::new_v4()));
    let file = std::fs::File::create(&path).unwrap();
    file.set_len(16 * 1024 * 1024 + 1).unwrap();
    assert!(matches!(Manifest::read(&path), Err(Error::Invalid(_))));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn audit_clone_preserves_management_leases_after_insertion() {
    let (mut api, id, _, _) = star(2);
    let sim = api.get_mut(&id).unwrap();
    sim.shutdown(false).unwrap();
    sim.set_auto_oob(true, true).unwrap();
    sim.create_node(NodeSpec::host("aaa")).unwrap();
    let leases: BTreeMap<_, _> = sim
        .nodes()
        .map(|n| (n.spec.name.clone(), n.management_ip.clone()))
        .collect();
    let clone = api.clone_simulation(&id, "clone", None, false).unwrap();
    for n in api.get(&clone).unwrap().nodes() {
        assert_eq!(leases[&n.spec.name], n.management_ip);
    }
}
