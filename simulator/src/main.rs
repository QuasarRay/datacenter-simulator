use std::io::{self,BufRead,Write};
use datacenter_simulator::{Simulator,collective::Reduction,manifest::Manifest,model::Role};

fn main()->Result<(),Box<dyn std::error::Error>> {
    let args:Vec<_>=std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("run") if args.len()==2=>{
            let mut api=Simulator::new();
            let id=api.import(Manifest::read(&args[1])?,true)?;
            let sim=api.get_mut(&id)?;
            let mut ranks:Vec<_>=sim.nodes().filter(|n|n.spec.role==Role::Host).map(|n|(n.spec.name.clone(),n.id.clone())).collect();
            ranks.sort();
            let ids:Vec<_>=ranks.iter().map(|(_,id)|id.clone()).collect();
            let inputs:Vec<_>=(1..=ids.len()).map(|i|vec![i as f64;8]).collect();
            let result=sim.all_reduce(&ids,&inputs,Reduction::Sum)?;
            println!("{}",serde_json::to_string_pretty(&serde_json::json!({"backend":"portable","ranks":ranks,"result":result}))?);
        }
        None | Some("json")=>{
            let mut api=Simulator::new(); let stdin=io::stdin(); let mut reader=stdin.lock(); let mut out=io::stdout().lock();
            loop {
                let mut line=String::new();
                if reader.read_line(&mut line)?==0 { break; }
                if line.len()>1024*1024 { return Err("command exceeds 1 MiB".into()); }
                if line.trim().is_empty() { continue; }
                writeln!(out,"{}",api.respond(&line))?; out.flush()?;
            }
        }
        _=>return Err("usage: datacenter-simulator [json | run manifest.json]".into()),
    }
    Ok(())
}
