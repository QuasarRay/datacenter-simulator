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
fn collective_failure_does_not_commit_partial_traffic() {
    let (mut api, id, ranks, links) = star(3);
    let sim = api.get_mut(&id).unwrap();
    let mut off = LinkSpec {
        up: false,
        ..LinkSpec::default()
    };
    sim.set_link(&links[2], off).unwrap();
    let before = snapshot(sim);
    assert!(
        sim.all_reduce(&ranks, &vec![vec![1.0; 7]; 3], Reduction::Sum)
            .is_err()
    );
    assert_eq!(snapshot(sim), before);
    off.up = true;
    sim.set_link(&links[2], off).unwrap();
    let result = sim
        .all_reduce(&ranks, &vec![vec![1.0; 7]; 3], Reduction::Sum)
        .unwrap();
    assert_eq!(result.started_ns, 0);
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
#[test]
fn remaining_collectives_have_rank_ordered_outputs() {
    let (mut api, id, ranks, _) = star(3);
    let sim = api.get_mut(&id).unwrap();
    assert_eq!(
        sim.broadcast(&ranks, 1, &[4.0]).unwrap().outputs,
        vec![vec![4.0]; 3]
    );
    assert_eq!(
        sim.all_gather(&ranks, &[vec![1.0], vec![2.0], vec![3.0]])
            .unwrap()
            .outputs,
        vec![vec![1.0, 2.0, 3.0]; 3]
    );
    assert_eq!(
        sim.reduce_scatter(&ranks, &vec![vec![1.0, 2.0, 3.0]; 3], Reduction::Sum)
            .unwrap()
            .outputs,
        vec![vec![3.0], vec![6.0], vec![9.0]]
    );
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
    fn ring_matches_independent_scalar_oracle(n in 1usize..9, count in 0usize..33, values in prop::collection::vec(-10i32..10,264)) {
        let (mut api,id,ranks,_)=star(n);let sim=api.get_mut(&id).unwrap();
        let inputs:Vec<Vec<f64>>=(0..n).map(|r|(0..count).map(|i|values[r*count+i] as f64).collect()).collect();
        for op in [Reduction::Sum,Reduction::Min,Reduction::Max,Reduction::Product,Reduction::Average] {
            let expected:Vec<_>=(0..count).map(|i|{
                let column:Vec<_>=inputs.iter().map(|v|v[i]).collect();
                match op {Reduction::Sum=>column.iter().sum(),Reduction::Average=>column.iter().sum::<f64>()/n as f64,Reduction::Min=>column.into_iter().fold(f64::INFINITY,f64::min),Reduction::Max=>column.into_iter().fold(f64::NEG_INFINITY,f64::max),Reduction::Product=>column.iter().product()}
            }).collect();
            let result=sim.all_reduce(&ranks,&inputs,op).unwrap();
            for output in result.outputs {prop_assert_eq!(output,&expected[..]);}
        }
    }
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
