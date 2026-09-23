// SPDX-License-Identifier: RPL-1.5
// Portable model construction only. Not a native-fabric or laptop capacity claim.
use datacenter_simulator::{Simulator, model::*};
use std::time::Instant;
fn main() {
    for count in [128, 256, 512, 1024, 2048] {
        let mut api = Simulator::new();
        let id = api.create("scale").unwrap().id().to_owned();
        let sim = api.get_mut(&id).unwrap();
        let started = Instant::now();
        for i in 0..count {
            let n = sim
                .create_node(NodeSpec::host(&format!("host{i:04}")))
                .unwrap();
            sim.create_interface(&n, "eth1", InterfaceType::Data)
                .unwrap();
        }
        let ms = started.elapsed().as_secs_f64() * 1000.0;
        let memory = std::fs::read_to_string("/proc/self/status")
            .unwrap()
            .lines()
            .find(|l| l.starts_with("VmHWM:"))
            .unwrap()
            .to_owned();
        println!(
            "nodes={count} interfaces={count} build_ms={ms:.3} serialized_data={} {memory}",
            sim.usage().unwrap().data_bytes
        );
    }
}
