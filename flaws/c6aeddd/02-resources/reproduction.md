# CPU-only reproduction

Run in a disposable checkout of `c6aedddfbb9ac569b5e0eb68acc616452bf66136` with its pinned submodules. Save the following as `simulator/examples/audit_resource_probe.rs`, then execute:

```sh
cargo run --release --locked --manifest-path simulator/Cargo.toml --example audit_resource_probe
```

The program uses public APIs and no native resources. Timings vary with machine/load; compare depth trends, not the exact microseconds. The report's measurements were taken with Rust 1.98.0. The temporary probe was removed from the audit PRs; only Markdown is committed.

```rust
use datacenter_simulator::{Simulator, Simulation, model::*, fabric::Fabric};
use std::{collections::BTreeMap, time::Instant};
fn pair() -> (Simulation,String,String) {
    let mut api=Simulator::new(); let id=api.create("probe").unwrap().id().to_string();
    let s=api.get_mut(&id).unwrap(); s.set_history_limit(0);
    let a=s.create_node(NodeSpec::host("a")).unwrap(); let b=s.create_node(NodeSpec::host("b")).unwrap();
    let x=s.create_interface(&a,"eth1",InterfaceType::Data).unwrap(); let y=s.create_interface(&b,"eth1",InterfaceType::Data).unwrap();
    s.create_link([&x,&y],LinkSpec::default()).unwrap(); s.start(None).unwrap(); (s.clone(),a,b)
}
fn main() {
    if let Some(path)=std::env::args().nth(1) { println!("{:?}",datacenter_simulator::evidence::FileIdentity::read(path)); return; }
    for depth in [1usize,1024,8192,32768] {
        let (mut s,a,b)=pair(); let mut f=Fabric::new(&s,65536).unwrap();
        let x=f.create_qp(&s,&a).unwrap(); let y=f.create_qp(&s,&b).unwrap(); f.connect(&s,&x,&y).unwrap();
        let src=f.register(&s,&a,vec![42],false).unwrap(); let dst=f.register(&s,&b,vec![0],true).unwrap();
        for i in 0..depth { f.post_receive(&s,&y,&dst,0..1,i as u64).unwrap(); }
        let began=Instant::now();
        for i in 0..512 { f.send(&mut s,&x,&src,0..1,i).unwrap();f.poll(&s,&x).unwrap();f.poll(&s,&y).unwrap();f.post_receive(&s,&y,&dst,0..1,i).unwrap(); }
        println!("queue depth={depth} sends=512 elapsed_us={}",began.elapsed().as_micros());
    }
    let (s,_,_)=pair();
    for limit in [0usize,1000,10000] {
        let mut s=s.clone();s.set_history_limit(limit);
        for _ in 0..limit {s.set_schedule(None,None).unwrap();}
        let began=Instant::now(); for _ in 0..20000 {s.set_schedule(None,None).unwrap();}
        println!("history limit={limit} updates=20000 elapsed_us={}",began.elapsed().as_micros());
    }
    for checkpoints in [0usize,8,16] {
        let (mut s,a,_)=pair();s.shutdown(false).unwrap();
        s.create_instruction(&a,InstructionData::File{files:BTreeMap::from([("/payload".into(),"x".repeat(1024*1024))])},true).unwrap();
        s.start(None).unwrap();
        for _ in 0..checkpoints {s.create_checkpoint("cp").unwrap();}
        let began=Instant::now();s.reset_nodes(std::slice::from_ref(&a),false).unwrap();
        println!("lifecycle checkpoints={checkpoints} reset_us={} retained_checkpoint_bytes={}",began.elapsed().as_micros(),s.usage().unwrap().checkpoint_bytes);
    }
    let (mut s,a,b)=pair();let mut f=Fabric::new(&s,16).unwrap();let x=f.create_qp(&s,&a).unwrap();let y=f.create_qp(&s,&b).unwrap();f.connect(&s,&x,&y).unwrap();let src=f.register(&s,&a,vec![42],false).unwrap();
    println!("rnr result={:?} state={:?} completion={:?}",f.send(&mut s,&x,&src,0..1,7),f.state(&s,&x),f.poll(&s,&x));
    let mut limits=s.limits().clone();limits.data_bytes=s.usage().unwrap().data_bytes;s.set_limits(limits).unwrap();
    println!("shutdown_full result={:?} state={:?}",s.shutdown(false),s.state());
    let mut api=Simulator::new();let id=api.create("limited").unwrap().id().to_string();let sim=api.get_mut(&id).unwrap();sim.create_node(NodeSpec::host("host")).unwrap();let mut limits=sim.limits().clone();limits.interfaces=1;sim.set_limits(limits).unwrap();
    let copy=api.clone_simulation(&id,"copy",None,false).unwrap();println!("clone_interfaces_budget original={} cloned={}",api.get(&id).unwrap().limits().interfaces,api.get(&copy).unwrap().limits().interfaces);
}
```
