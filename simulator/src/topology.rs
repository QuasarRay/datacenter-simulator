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

use crate::model::*;
use petgraph::{
    algo::astar,
    graph::{DiGraph, NodeIndex},
};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize)]
pub struct Hop {
    pub link: String,
    pub from: String,
    pub to: String,
    pub start_ns: u64,
    pub serialized_ns: u64,
    pub arrival_ns: u64,
}
#[derive(Debug, Clone, Serialize)]
pub struct Transmission {
    pub source: String,
    pub destination: String,
    pub bytes: usize,
    pub submitted_ns: u64,
    pub completed_ns: u64,
    pub hops: Vec<Hop>,
}
pub(crate) fn serialization_ns(bytes: usize, bandwidth: u64) -> Result<u64> {
    let bits_ns = (bytes as u128) * 8 * 1_000_000_000;
    u64::try_from(bits_ns.div_ceil(bandwidth as u128))
        .map_err(|_| Error::Invalid("transfer duration overflow".into()))
}
impl Simulation {
    /// Compute a minimum unloaded-latency path. Hosts never forward transit traffic.
    /// Parallel links remain separate graph edges. OOB and PCIe do not route data.
    pub fn route(&self, source: &str, destination: &str, bytes: usize) -> Result<Vec<String>> {
        self.active()?;
        self.node(source)?;
        self.node(destination)?;
        if bytes > 16 * 1024 * 1024 {
            return Err(Error::Invalid("transfer exceeds 16 MiB".into()));
        }
        let mut graph = DiGraph::<String, (String, u64)>::new();
        let mut index: BTreeMap<&str, NodeIndex> = BTreeMap::new();
        let mut ordered_nodes: Vec<_> = self
            .nodes
            .values()
            .filter(|n| n.state == State::Active)
            .collect();
        ordered_nodes.sort_by_key(|n| &n.spec.name);
        for n in ordered_nodes {
            index.insert(&n.id, graph.add_node(n.id.clone()));
        }
        let start = *index
            .get(source)
            .ok_or_else(|| Error::State("source node is inactive".into()))?;
        let goal = *index
            .get(destination)
            .ok_or_else(|| Error::State("destination node is inactive".into()))?;
        // Sorting by endpoint names and link specs makes equal-cost routes reproducible
        // across imports that allocate fresh resource UUIDs.
        let mut links: Vec<_> = self.links.values().collect();
        links.sort_by_key(|l| {
            let a = &self.interfaces[&l.interfaces[0]];
            let b = &self.interfaces[&l.interfaces[1]];
            (
                self.nodes[&a.node].spec.name.clone(),
                a.name.clone(),
                self.nodes[&b.node].spec.name.clone(),
                b.name.clone(),
            )
        });
        for link in links {
            if !link.spec.up {
                continue;
            }
            let a = &self.interfaces[&link.interfaces[0]];
            let b = &self.interfaces[&link.interfaces[1]];
            if a.interface_type != InterfaceType::Data {
                continue;
            }
            let cost = serialization_ns(bytes, link.spec.bandwidth_bps)?
                .checked_add(link.spec.latency_ns)
                .ok_or_else(|| Error::Invalid("route cost overflow".into()))?;
            for (from, to) in [(a, b), (b, a)] {
                if from.node != source && self.nodes[&from.node].spec.role != Role::Switch {
                    continue;
                }
                if let (Some(&x), Some(&y)) =
                    (index.get(from.node.as_str()), index.get(to.node.as_str()))
                {
                    graph.add_edge(x, y, (link.id.clone(), cost));
                }
            }
        }
        let (_, nodes) = astar(&graph, start, |n| n == goal, |e| e.weight().1, |_| 0u64)
            .ok_or_else(|| Error::NoRoute(source.into(), destination.into()))?;
        // Match the lowest-cost parallel edge used by the path algorithm.
        nodes
            .windows(2)
            .map(|w| {
                graph
                    .edges_connecting(w[0], w[1])
                    .min_by_key(|e| e.weight().1)
                    .map(|e| e.weight().0.clone())
                    .ok_or_else(|| Error::NoRoute(source.into(), destination.into()))
            })
            .collect()
    }
    pub fn transfer(
        &mut self,
        source: &str,
        destination: &str,
        bytes: usize,
    ) -> Result<Transmission> {
        let trace = self.schedule_transfer(source, destination, bytes, self.clock_ns)?;
        self.clock_ns = trace.completed_ns;
        Ok(trace)
    }
    /// Submit concurrent transfers with the same time to model link contention.
    /// Work is store-and-forward, full duplex, and FIFO on each directed link.
    pub fn schedule_transfer(
        &mut self,
        source: &str,
        destination: &str,
        bytes: usize,
        at_ns: u64,
    ) -> Result<Transmission> {
        if at_ns < self.clock_ns {
            return Err(Error::Invalid(
                "submission precedes completed simulation time".into(),
            ));
        }
        let route = self.route(source, destination, bytes)?;
        let mut available = self.available.clone();
        let mut time = at_ns;
        let mut current = source.to_string();
        let mut hops = Vec::new();
        for link_id in route {
            let link = &self.links[&link_id];
            let a = &self.interfaces[&link.interfaces[0]].node;
            let b = &self.interfaces[&link.interfaces[1]].node;
            let next = if &current == a { b } else { a };
            let key = (link_id.clone(), current.clone());
            let start = time.max(*available.get(&key).unwrap_or(&0));
            let serialized = start
                .checked_add(serialization_ns(bytes, link.spec.bandwidth_bps)?)
                .ok_or_else(|| Error::Invalid("simulation clock overflow".into()))?;
            time = serialized
                .checked_add(link.spec.latency_ns)
                .ok_or_else(|| Error::Invalid("simulation clock overflow".into()))?;
            available.insert(key, serialized);
            hops.push(Hop {
                link: link_id,
                from: current,
                to: next.clone(),
                start_ns: start,
                serialized_ns: serialized,
                arrival_ns: time,
            });
            current = next.clone();
        }
        self.available = available;
        Ok(Transmission {
            source: source.into(),
            destination: destination.into(),
            bytes,
            submitted_ns: at_ns,
            completed_ns: time,
            hops,
        })
    }
}
