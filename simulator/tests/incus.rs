#![cfg(feature = "incus")]
use datacenter_simulator::incus::Config;
use std::collections::BTreeSet;

fn config() -> Config {
    Config {
        project: "ncp-test".into(),
        socket: "/var/lib/incus/unix.socket".into(),
        storage_pool: "default".into(),
        image_fingerprint: "a".repeat(64),
        manifest: std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../integrations/incus/topology.json"),
        max_memory_mib: 8192,
    }
}
#[test]
fn explicit_ports_and_no_inherited_profile_or_network() {
    let plan = config().plan().unwrap();
    assert_eq!(plan.nodes.len(), 5);
    assert_eq!(plan.cables.len(), 4);
    let mut names = BTreeSet::new();
    for node in &plan.nodes {
        assert!(node.spec["profiles"].as_array().unwrap().is_empty());
        assert_eq!(node.spec["type"], "container");
        assert_eq!(node.spec["config"]["security.privileged"], "false");
        for (name, device) in node.spec["devices"].as_object().unwrap() {
            if name == "root" {
                continue;
            }
            assert_eq!(device["nictype"], "p2p");
            let host = device["host_name"].as_str().unwrap();
            assert!(host.len() <= 15 && names.insert(host.to_string()));
            assert!(device.get("network").is_none());
        }
    }
    assert_eq!(names.len(), 8);
    for cable in &plan.cables {
        assert!(cable.peers.iter().all(|p| names.contains(p)));
    }
}
#[test]
fn reject_unsafe_names_unknown_image_and_over_budget() {
    for project in ["default", "ncp-x&project=default", "ncp-a/b", "ncp-$(id)"] {
        let mut c = config();
        c.project = project.into();
        assert!(c.plan().is_err());
    }
    let mut c = config();
    c.image_fingerprint = "latest".into();
    assert!(c.plan().is_err());
    let mut c = config();
    c.max_memory_mib = 1;
    assert!(c.plan().is_err());
}
#[test]
fn independent_runs_never_reuse_owner_or_peer_names() {
    let a = config().plan().unwrap();
    let b = config().plan().unwrap();
    assert_ne!(a.owner, b.owner);
    let names: BTreeSet<_> = a.cables.iter().flat_map(|c| c.peers.iter()).collect();
    assert!(
        b.cables
            .iter()
            .flat_map(|c| c.peers.iter())
            .all(|p| !names.contains(p))
    );
}
