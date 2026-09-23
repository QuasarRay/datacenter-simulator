// SPDX-License-Identifier: RPL-1.5
use datacenter_simulator::{Simulator, manifest::Manifest, model::*};
use serde_json::json;
use std::collections::BTreeMap;

fn small() -> (Simulator, String, String, String) {
    let mut api = Simulator::new();
    let id = api.create("budgets").unwrap().id().to_owned();
    let sim = api.get_mut(&id).unwrap();
    let node = sim.create_node(NodeSpec::host("a")).unwrap();
    let port = sim
        .create_interface(&node, "eth0", InterfaceType::Data)
        .unwrap();
    (api, id, node, port)
}
fn state(sim: &Simulation) -> serde_json::Value {
    serde_json::to_value(sim).unwrap()
}

#[test]
fn counts_and_bulk_breakout_fail_before_mutation() {
    let (mut api, id, node, port) = small();
    let sim = api.get_mut(&id).unwrap();
    let mut limits = sim.limits().clone();
    limits.interfaces = 3;
    limits.services = 1;
    limits.instructions = 1;
    limits.checkpoints = 1;
    limits.links = 0;
    sim.set_limits(limits).unwrap();
    let before = state(sim);
    assert!(sim.breakout(&port, 4).is_err());
    assert_eq!(state(sim), before);
    let port2 = sim
        .create_interface(&node, "eth1", InterfaceType::Data)
        .unwrap();
    let before = state(sim);
    assert!(
        sim.create_link([&port, &port2], LinkSpec::default())
            .is_err()
    );
    assert_eq!(state(sim), before);
    sim.create_service(&port, "ssh", 22, ServiceType::SSH)
        .unwrap();
    sim.create_instruction(
        &node,
        InstructionData::Init {
            hostname: "guest".into(),
        },
        true,
    )
    .unwrap();
    sim.create_checkpoint("first").unwrap();
    let before = state(sim);
    assert!(
        sim.create_service(&port, "http", 80, ServiceType::HTTP)
            .is_err()
    );
    assert!(
        sim.create_instruction(
            &node,
            InstructionData::Init {
                hostname: "other".into()
            },
            true
        )
        .is_err()
    );
    assert!(sim.create_checkpoint("second").is_err());
    assert_eq!(state(sim), before);
}

#[test]
fn byte_budgets_cover_metadata_files_and_checkpoint_retention() {
    let (mut api, id, node, _) = small();
    let sim = api.get_mut(&id).unwrap();
    let mut limits = sim.limits().clone();
    limits.data_bytes = sim.usage().unwrap().data_bytes + 4096;
    limits.checkpoint_bytes = 1;
    sim.set_limits(limits).unwrap();
    let before = state(sim);
    assert!(sim.update(None, Some(Some("x".repeat(8192)))).is_err());
    assert!(
        sim.assign_configs(&[(node.clone(), Some("x".repeat(8192)), None)])
            .is_err()
    );
    assert!(sim.create_ztp_script("x".repeat(8192)).is_err());
    assert!(sim.create_checkpoint("oversize").is_err());
    assert_eq!(state(sim), before);
    sim.create_instruction(
        &node,
        InstructionData::File {
            files: BTreeMap::from([("/data".into(), "x".repeat(1800))]),
        },
        true,
    )
    .unwrap();
    let mut limits = sim.limits().clone();
    limits.data_bytes = sim.usage().unwrap().data_bytes + 10;
    sim.set_limits(limits).unwrap();
    let before = state(sim);
    assert!(sim.start(None).is_err());
    assert!(sim.rebuild(None).is_err());
    assert_eq!(state(sim), before);
}

#[test]
fn manifests_canonicalize_services_and_instruction_application() {
    let (mut api, id, node, port) = small();
    let sim = api.get_mut(&id).unwrap();
    sim.create_service(&port, "z", 81, ServiceType::HTTP)
        .unwrap();
    sim.create_service(&port, "a", 22, ServiceType::SSH)
        .unwrap();
    for value in ["z", "a"] {
        sim.create_instruction(
            &node,
            InstructionData::File {
                files: BTreeMap::from([("/same".into(), value.into())]),
            },
            true,
        )
        .unwrap();
    }
    let manifest = sim.export().unwrap();
    let expected = serde_json::to_value(&manifest).unwrap();
    for _ in 0..8 {
        let cloned = api.import(manifest.clone(), true).unwrap();
        let sim = api.get(&cloned).unwrap();
        assert_eq!(
            serde_json::to_value(sim.export().unwrap()).unwrap(),
            expected
        );
        assert_eq!(
            sim.runtime(&sim.node_named("a").unwrap().id).unwrap().files["/same"],
            "z"
        );
    }
}

#[test]
fn breakout_children_must_remain_data_interfaces() {
    let (mut api, id, _, port) = small();
    api.get_mut(&id).unwrap().breakout(&port, 2).unwrap();
    let manifest = api.get(&id).unwrap().export().unwrap();
    let mut value = serde_json::to_value(manifest).unwrap();
    let children = value["content"]["nodes"]["a"]["interfaces"]
        .as_object_mut()
        .unwrap();
    let (_, child) = children
        .iter_mut()
        .find(|(_, v)| !v["split_parent"].is_null())
        .unwrap();
    child["interface_type"] = json!(InterfaceType::Oob);
    let count = api.list(None, None, 0, 100).len();
    assert!(
        api.import(Manifest::from_json(&value.to_string()).unwrap(), false)
            .is_err()
    );
    assert_eq!(api.list(None, None, 0, 100).len(), count);
}

#[test]
fn rename_preserves_guest_hostname_until_explicit_rebuild_and_active_services_are_immutable() {
    let (mut api, id, node, port) = small();
    let sim = api.get_mut(&id).unwrap();
    let service = sim
        .create_service(&port, "ssh", 22, ServiceType::SSH)
        .unwrap();
    sim.update_node(&node, NodeSpec::host("renamed")).unwrap();
    assert_eq!(sim.runtime(&node).unwrap().hostname, "a");
    sim.rebuild(None).unwrap();
    assert_eq!(sim.runtime(&node).unwrap().hostname, "renamed");
    sim.start(None).unwrap();
    let before = state(sim);
    assert!(sim.delete_service(&service).is_err());
    assert!(
        sim.create_service(&port, "new", 80, ServiceType::HTTP)
            .is_err()
    );
    assert_eq!(state(sim), before);
}

#[test]
fn bounded_files_reject_oversize_and_identity_detects_replacement() {
    use datacenter_simulator::{evidence::FileIdentity, input};
    let path = std::env::temp_dir().join(format!("audit-{}", uuid::Uuid::new_v4()));
    std::fs::write(&path, b"pinned").unwrap();
    let identity = FileIdentity::read(&path).unwrap();
    identity.verify().unwrap();
    std::fs::write(&path, b"changed").unwrap();
    assert!(identity.verify().is_err());
    assert_eq!(input::read(&path, 7).unwrap(), b"changed");
    assert!(input::read(&path, 6).is_err());
    std::fs::remove_file(path).unwrap();
}

#[cfg(feature = "linux")]
#[test]
fn native_addresses_ignore_ids_and_endpoint_orientation() {
    use datacenter_simulator::linux::LinuxFabric;
    let manifest = Manifest::from_json(include_str!("../examples/gpu-spine-leaf.json")).unwrap();
    let mut api = Simulator::new();
    let first = api.import(manifest.clone(), false).unwrap();
    let mut reversed = manifest;
    reversed.content.links.reverse();
    for link in &mut reversed.content.links {
        link.endpoints.reverse();
    }
    let second = api.import(reversed, false).unwrap();
    let by_name = |id: &str| {
        let sim = api.get(id).unwrap();
        LinuxFabric::link_addresses(sim)
            .unwrap()
            .into_iter()
            .map(|(id, ip)| {
                let i = sim.interface(&id).unwrap();
                (
                    format!("{}:{}", sim.node(&i.node).unwrap().spec.name, i.name),
                    ip,
                )
            })
            .collect::<BTreeMap<_, _>>()
    };
    assert_eq!(by_name(&first), by_name(&second));
}

#[test]
fn checkpoint_restore_budgets_instruction_states_before_committing() {
    let (mut api, id, node, _) = small();
    let sim = api.get_mut(&id).unwrap();
    sim.create_instruction(
        &node,
        InstructionData::Init {
            hostname: "a".into(),
        },
        false,
    )
    .unwrap();
    let pending = sim.create_checkpoint("pending").unwrap();
    sim.start(None).unwrap();
    sim.shutdown(false).unwrap();
    let complete = sim.create_checkpoint("complete").unwrap();
    sim.rebuild(Some(&pending)).unwrap();
    let mut limits = sim.limits().clone();
    limits.data_bytes = sim.usage().unwrap().data_bytes;
    sim.set_limits(limits).unwrap();
    let before = state(sim);
    assert!(sim.start(Some(&complete)).is_err());
    assert_eq!(state(sim), before);
}
