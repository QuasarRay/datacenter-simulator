// SPDX-License-Identifier: RPL-1.5
#![cfg(feature = "netbox")]
use datacenter_simulator::{
    Simulator,
    netbox::{self, NetBoxConfig, Snapshot},
};

fn input() -> (NetBoxConfig, Snapshot) {
    (
        serde_json::from_str(include_str!(
            "../../integrations/netbox/config.example.json"
        ))
        .unwrap(),
        serde_json::from_str(include_str!(
            "../../integrations/netbox/snapshot.example.json"
        ))
        .unwrap(),
    )
}

#[test]
fn netbox_ids_survive_display_renames_and_preserve_physical_rates() {
    let (config, mut snapshot) = input();
    let before = netbox::compile(&config, &snapshot).unwrap();
    assert_eq!(before.content.nodes.len(), 3);
    assert_eq!(before.content.nodes["nb1"].interfaces.len(), 1);
    assert_eq!(
        before.content.links[0].condition.bandwidth_bps,
        200_000_000_000
    );
    assert_eq!(
        before.content.nodes["nb1"].labels["netbox.interfaces"]["i1"]["name"],
        "mlx5_0/1"
    );
    snapshot.devices[0].name = "任意 display name / renamed".into();
    snapshot.interfaces[0].name = "Ethernet 1/2/really-long-native-name".into();
    let after = netbox::compile(&config, &snapshot).unwrap();
    assert_eq!(
        before.content.links[0].endpoints,
        after.content.links[0].endpoints
    );
    assert_eq!(
        after.content.nodes["nb1"].labels["netbox.device_name"],
        snapshot.devices[0].name
    );
    Simulator::new().import(after, true).unwrap();
}

#[test]
fn intended_admin_state_controls_cables() {
    let (mut config, mut snapshot) = input();
    config.planned_links_up = false;
    assert!(
        !netbox::compile(&config, &snapshot).unwrap().content.links[1]
            .condition
            .up
    );
    config.planned_links_up = true;
    snapshot.interfaces[0].enabled = false;
    let m = netbox::compile(&config, &snapshot).unwrap();
    assert!(!m.content.links[0].condition.up);
    assert!(m.content.links[1].condition.up);
    snapshot.cables[1].status.value = "decommissioning".into();
    assert!(
        !netbox::compile(&config, &snapshot).unwrap().content.links[1]
            .condition
            .up
    );
}

#[test]
fn ambiguous_or_partial_cabling_never_becomes_a_shortcut() {
    let (config, snapshot) = input();
    for change in 0..6 {
        let mut s = snapshot.clone();
        match change {
            0 => s.cables[0].b_terminations[0].object_type = "dcim.frontport".into(),
            1 => s.cables[0].b_terminations[0].object_id = 999,
            2 => {
                let extra = s.cables[0].a_terminations[0].clone();
                s.cables[0].b_terminations.push(extra);
            }
            3 => s.interfaces[0].cable.as_mut().unwrap().id = 2,
            4 => s.devices[0].role.slug = "unmapped".into(),
            _ => s.interfaces[0].lag = Some(netbox::Reference { id: 42 }),
        }
        assert!(netbox::compile(&config, &s).is_err(), "case {change}");
    }
}

#[test]
fn missing_and_overflowing_speeds_require_explicit_correction() {
    let (mut config, mut s) = input();
    s.interfaces[0].speed = None;
    assert!(netbox::compile(&config, &s).is_err());
    config.default_bandwidth_bps = Some(100_000_000_000);
    assert_eq!(
        netbox::compile(&config, &s).unwrap().content.links[0]
            .condition
            .bandwidth_bps,
        100_000_000_000
    );
    s.interfaces[0].speed = Some(u64::MAX);
    assert!(netbox::compile(&config, &s).is_err());
}

#[cfg(feature = "ibsim")]
#[test]
fn intended_graph_becomes_native_ib_ports_and_initial_cable_state() {
    let (config, mut snapshot) = input();
    snapshot.interfaces[1].enabled = false;
    let m = netbox::compile(&config, &snapshot).unwrap();
    let mut api = Simulator::new();
    let id = api.import(m, true).unwrap();
    let (t, plan) = datacenter_simulator::infiniband::topology(api.get(&id).unwrap()).unwrap();
    assert_eq!(t.nodes().count(), 3);
    assert_eq!(t.links().filter(|l| l.up).count(), 1);
    assert_eq!(plan.ports["nb3:i3"], 1);
    assert_eq!(plan.ports["nb3:i4"], 2);
    assert!(!plan.native_topology.contains("[2]"));
}

#[cfg(feature = "mokka")]
#[test]
fn kubernetes_intent_uses_pinned_gpu_profiles_with_ib_mocks_off() {
    use datacenter_simulator::mokka::MokkaConfig;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../integrations/k8s-test-infra/config.example.json");
    let mut config = MokkaConfig::read(&path).unwrap();
    let plan = config.plan().unwrap();
    assert_eq!(plan.releases.len(), 2);
    assert!(!plan.nccl_tested && !plan.rdma_payload_tested);
    for release in &plan.releases {
        assert_eq!(release.expected_gpus, 2);
        assert_eq!(release.values["infiniband"]["mockTier"], "off");
        assert_eq!(release.values["nri"]["enabled"], false);
        let profile: serde_json::Value =
            serde_json::from_str(release.values["gpu"]["customConfig"].as_str().unwrap()).unwrap();
        assert_eq!(profile["infiniband"]["enabled"], false);
        assert_eq!(profile["system"]["num_devices"], release.expected_gpus);
    }
    config.nodes.insert(
        "nb1".into(),
        datacenter_simulator::mokka::NodeBinding {
            kubernetes_node: "simulator-worker".into(),
            profile: "t4".into(),
            gpu_count: 999,
        },
    );
    assert!(config.plan().is_err());
    config.nodes.get_mut("nb1").unwrap().gpu_count = 1;
    config
        .nodes
        .insert("nb2".into(), config.nodes["nb1"].clone());
    assert!(config.plan().is_err());
    config.nodes.clear();
    config.namespace = "default".into();
    assert!(config.plan().is_err());
}
