// SPDX-License-Identifier: RPL-1.5
use datacenter_simulator::{Simulator, model::*};

fn exact_budget(sim: &mut Simulation) {
    let mut limits = sim.limits().clone();
    limits.data_bytes = sim.usage().unwrap().data_bytes;
    sim.set_limits(limits).unwrap();
}

#[test]
fn oob_respects_retained_data_budget() {
    let mut api = Simulator::new();
    let id = api.create("budget").unwrap().id().to_owned();
    let sim = api.get_mut(&id).unwrap();
    sim.set_auto_oob(false, false).unwrap();
    sim.create_node(NodeSpec::host("host")).unwrap();
    exact_budget(sim);
    let limit = sim.limits().data_bytes;
    let before = serde_json::to_value(&*sim).unwrap();
    let result = sim.set_auto_oob(true, true);
    assert!(result.is_err());
    assert_eq!(serde_json::to_value(&*sim).unwrap(), before);
    let used = sim.usage().unwrap().data_bytes;
    println!("OOB result={result:?}, limit={limit}, used={used}");
    assert!(used <= limit, "OOB mutation exceeded accepted byte budget");
}

#[test]
fn shutdown_respects_retained_data_budget() {
    let mut api = Simulator::new();
    let id = api.create("budget").unwrap().id().to_owned();
    let sim = api.get_mut(&id).unwrap();
    sim.create_node(NodeSpec::host("host")).unwrap();
    sim.start(None).unwrap();
    exact_budget(sim);
    let limit = sim.limits().data_bytes;
    let before = serde_json::to_value(&*sim).unwrap();
    let result = sim.shutdown(true);
    assert!(result.is_err());
    assert_eq!(serde_json::to_value(&*sim).unwrap(), before);
    let used = sim.usage().unwrap().data_bytes;
    println!("shutdown result={result:?}, limit={limit}, used={used}");
    assert!(used <= limit, "shutdown exceeded accepted byte budget");
}

#[test]
fn rebuild_respects_generation_byte_growth() {
    let mut api = Simulator::new();
    let id = api.create("budget").unwrap().id().to_owned();
    let sim = api.get_mut(&id).unwrap();
    let node = sim.create_node(NodeSpec::host("host")).unwrap();
    while sim.node(&node).unwrap().generation < 9 {
        sim.rebuild(None).unwrap();
    }
    exact_budget(sim);
    let limit = sim.limits().data_bytes;
    let before = serde_json::to_value(&*sim).unwrap();
    let result = sim.rebuild(None);
    assert!(result.is_err());
    assert_eq!(serde_json::to_value(&*sim).unwrap(), before);
    let used = sim.usage().unwrap().data_bytes;
    println!(
        "rebuild result={result:?}, generation={}, limit={limit}, used={used}",
        sim.node(&node).unwrap().generation
    );
    assert!(
        used <= limit,
        "generation digit growth exceeded accepted byte budget"
    );
}

fn connect(sim: &mut Simulation, a: &str, b: &str, port: &str, latency: u64) {
    let x = sim.create_interface(a, port, InterfaceType::Data).unwrap();
    let y = sim.create_interface(b, port, InterfaceType::Data).unwrap();
    sim.create_link(
        [&x, &y],
        LinkSpec {
            bandwidth_bps: 8_000_000_000,
            latency_ns: latency,
            up: true,
        },
    )
    .unwrap();
}

#[test]
fn downstream_fifo_follows_arrival_time() {
    let mut api = Simulator::new();
    let id = api.create("fifo").unwrap().id().to_owned();
    let sim = api.get_mut(&id).unwrap();
    let slow = sim.create_node(NodeSpec::host("slow")).unwrap();
    let fast = sim.create_node(NodeSpec::host("fast")).unwrap();
    let dst = sim.create_node(NodeSpec::host("dst")).unwrap();
    let mut switch = NodeSpec::host("switch");
    switch.role = Role::Switch;
    let sw = sim.create_node(switch).unwrap();
    connect(sim, &slow, &sw, "slow", 1000);
    connect(sim, &fast, &sw, "fast", 0);
    connect(sim, &sw, &dst, "out", 0);
    sim.start(None).unwrap();
    let traces = sim
        .schedule_transfers(&[(&slow, &dst, 8, 0), (&fast, &dst, 8, 0)])
        .unwrap();
    let first = &traces[0];
    let second = &traces[1];
    assert!(sim.schedule_transfer(&fast, &dst, 8, 0).is_err());
    let mut reversed = sim.clone();
    reversed.shutdown(false).unwrap();
    reversed.start(None).unwrap();
    let opposite = reversed
        .schedule_transfers(&[(&fast, &dst, 8, 0), (&slow, &dst, 8, 0)])
        .unwrap();
    assert_eq!(first.completed_ns, opposite[1].completed_ns);
    assert_eq!(second.completed_ns, opposite[0].completed_ns);
    println!(
        "slow arrives switch={}, takes egress={}; fast arrives switch={}, takes egress={}",
        first.hops[0].arrival_ns,
        first.hops[1].start_ns,
        second.hops[0].arrival_ns,
        second.hops[1].start_ns
    );
    assert_eq!(
        second.hops[1].start_ns, 8,
        "earlier arrival should use idle egress before the slow transfer"
    );
}

#[test]
fn route_ties_ignore_undirected_endpoint_orientation() {
    let mut api = Simulator::new();
    let id = api.create("ties").unwrap().id().to_owned();
    let sim = api.get_mut(&id).unwrap();
    let a = sim.create_node(NodeSpec::host("a")).unwrap();
    let b = sim.create_node(NodeSpec::host("b")).unwrap();
    connect(sim, &a, &b, "p1", 100);
    connect(sim, &a, &b, "p2", 100);
    let manifest = sim.export().unwrap();
    let one = api.import(manifest.clone(), true).unwrap();
    let mut flipped = manifest;
    flipped.content.links[0].endpoints.reverse();
    let two = api.import(flipped, true).unwrap();
    let chosen = |id: &str| {
        let s = api.get(id).unwrap();
        let route = s
            .route(
                &s.node_named("a").unwrap().id,
                &s.node_named("b").unwrap().id,
                8,
            )
            .unwrap();
        s.interface(&s.links().find(|l| l.id == route[0]).unwrap().interfaces[0])
            .unwrap()
            .name
            .clone()
    };
    let before = chosen(&one);
    let after = chosen(&two);
    println!("same undirected graph: chosen port before={before}, after={after}");
    assert_eq!(
        before, after,
        "endpoint orientation must not select a different physical link"
    );
}

#[test]
fn aggregate_fabric_budgets_release_capacity_on_delete_and_poll() {
    use datacenter_simulator::fabric::{Fabric, FabricLimits};
    let mut api = Simulator::new();
    let id = api.create("fabric").unwrap().id().to_owned();
    let sim = api.get_mut(&id).unwrap();
    let n = sim.create_node(NodeSpec::host("host")).unwrap();
    sim.start(None).unwrap();
    let mut fabric = Fabric::with_limits(
        sim,
        10,
        FabricLimits {
            queue_pairs: 2,
            memory_regions: 2,
            queued_entries: 2,
            payload_bytes: 2,
        },
    )
    .unwrap();
    let a = fabric.create_qp(sim, &n).unwrap();
    let b = fabric.create_qp(sim, &n).unwrap();
    assert!(fabric.create_qp(sim, &n).is_err());
    let m = fabric.register(sim, &n, vec![1], false).unwrap();
    let target = fabric.register(sim, &n, vec![0], false).unwrap();
    assert!(fabric.register(sim, &n, vec![0], false).is_err());
    fabric.connect(sim, &a, &b).unwrap();
    fabric.post_receive(sim, &b, &target, 0..1, 1).unwrap();
    fabric.post_receive(sim, &b, &target, 0..1, 2).unwrap();
    let before = fabric.usage();
    assert!(fabric.send(sim, &a, &m, 0..1, 3).is_err());
    assert_eq!(fabric.usage(), before);
    fabric.destroy_qp(&b).unwrap();
    assert_eq!(fabric.usage().queued_entries, 0);
    fabric.deregister(&target).unwrap();
    assert_eq!(fabric.usage().payload_bytes, 1);
    let b = fabric.create_qp(sim, &n).unwrap();
    fabric.destroy_qp(&a).unwrap();
    let a = fabric.create_qp(sim, &n).unwrap();
    fabric.connect(sim, &a, &b).unwrap();
    fabric.post_receive(sim, &b, &m, 0..1, 1).unwrap();
    fabric.send(sim, &a, &m, 0..1, 3).unwrap();
    assert_eq!(fabric.usage().queued_entries, 2);
    fabric.poll(sim, &a).unwrap();
    fabric.poll(sim, &b).unwrap();
    assert_eq!(fabric.usage().queued_entries, 0);
}

#[test]
fn nccl_report_requires_the_requested_test_matrix() {
    use serde_json::json;
    let rows: Vec<_> = ["sum", "prod", "min", "max", "avg", "mulsum"]
        .iter()
        .enumerate()
        .map(|(i, op)| {
            json!({
                "redop":op,"root":format!("{}",i%2),"size":256,
                "out_of_place":{"nwrong":0.0,"time":1.0}
            })
        })
        .collect();
    let report = json!({"version":4,"nccl_version":23102,"end_time":"2026-09-23T00:00:00",
        "args":["/opt/simulator/nccl-tests/build/reduce_perf","-c","1"],
        "env":["NCCL_NET=Socket","NCCL_NET_PLUGIN=none","NCCL_SOCKET_IFNAME==simnccl","NCCL_IB_DISABLE=1","NCCL_P2P_DISABLE=1","NCCL_SHM_DISABLE=1","OMPI_MCA_pml=ob1","OMPI_MCA_btl=self,tcp","OMPI_MCA_btl_tcp_if_include=simnccl","OMPI_MCA_oob_tcp_if_include=simnccl"],
        "config":{"validation":1,"ngpus":1,"nthreads":1,"devices":[{"rank":0,"hostname":"gpu1","device_info":"fixture"},{"rank":1,"hostname":"gpu2","device_info":"fixture"}]},
        "results":rows,"out_of_bounds":{"count":0,"okay":"true"},"errors":[""]});
    let result = datacenter_simulator::deepops::validate_nccl(
        &report,
        "reduce",
        &["gpu1".into(), "gpu2".into()],
    );
    println!(
        "6 reduce rows at one size, half the root/operation combinations and no in_place results: {result:?}"
    );
    assert!(
        result.is_err(),
        "truncated requested coverage should be rejected"
    );
}

proptest::proptest! {
    #[test]
    fn fifo_matches_an_independent_single_egress_reference(
        messages in proptest::collection::vec((0u64..2000, 1usize..200), 2..8)
    ) {
        let mut api=Simulator::new();
        let id=api.create("fan-in").unwrap().id().to_owned();
        let sim=api.get_mut(&id).unwrap();
        let dst=sim.create_node(NodeSpec::host("dst")).unwrap();
        let mut spec=NodeSpec::host("sw"); spec.role=Role::Switch;
        let sw=sim.create_node(spec).unwrap();
        connect(sim,&sw,&dst,"egress",0);
        let mut sources=Vec::new();
        for (i,(delay,_)) in messages.iter().enumerate() {
            let name=format!("src{i}");
            let src=sim.create_node(NodeSpec::host(&name)).unwrap();
            connect(sim,&src,&sw,&name,*delay);
            sources.push(src);
        }
        sim.start(None).unwrap();
        let requests:Vec<_>=messages.iter().enumerate().map(|(i,(_,bytes))|
            (sources[i].as_str(),dst.as_str(),*bytes,0)).collect();
        let traces=sim.schedule_transfers(&requests).unwrap();
        let mut arrivals:Vec<_>=messages.iter().enumerate().map(|(i,(delay,bytes))|
            (*delay+*bytes as u64,i,*bytes as u64)).collect();
        arrivals.sort();
        let mut busy_until=0;
        for (arrival,i,bytes) in arrivals {
            let start=arrival.max(busy_until);
            busy_until=start+bytes;
            proptest::prop_assert_eq!(traces[i].hops[1].start_ns,start);
            proptest::prop_assert_eq!(traces[i].completed_ns,busy_until);
        }
    }
}

#[test]
fn convenience_collectives_forward_explicit_execution_options() {
    use datacenter_simulator::collective::{NcclOptions, Reduction};
    let mut api = Simulator::new();
    let id = api.create("options").unwrap().id().to_owned();
    let sim = api.get_mut(&id).unwrap();
    let node = sim.create_node(NodeSpec::host("host")).unwrap();
    sim.start(None).unwrap();
    let options = NcclOptions {
        timeout_secs: 0,
        ..Default::default()
    };
    let ranks = vec![node];
    for result in [
        sim.all_reduce(&ranks, &[vec![1.0]], Reduction::Sum, &options),
        sim.broadcast(&ranks, 0, &[1.0], &options),
        sim.all_gather(&ranks, &[vec![1.0]], &options),
        sim.reduce_scatter(&ranks, &[vec![1.0]], Reduction::Sum, &options),
    ] {
        assert!(matches!(result, Err(Error::Invalid(_))));
    }
}

#[test]
fn help_and_portable_doctor_succeed_without_external_prerequisites() {
    use std::process::Command;
    let binary = env!("CARGO_BIN_EXE_datacenter-simulator");
    let help = Command::new(binary).arg("--help").output().unwrap();
    assert!(help.status.success());
    let help = String::from_utf8(help.stdout).unwrap();
    for text in [
        "--libraries",
        "netbox-compile",
        "mokka-apply",
        "ibsim-plan",
        "doctor",
    ] {
        assert!(help.contains(text));
    }
    let report = Command::new(binary)
        .args(["doctor", "portable"])
        .output()
        .unwrap();
    assert!(report.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&report.stdout).unwrap()["ok"],
        true
    );
}
