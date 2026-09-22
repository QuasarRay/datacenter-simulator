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
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};
use serde::Serialize;
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap};

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
/// One immutable routing graph per operation. Edge choice depends on latency only;
/// packet serialization affects transfer timing, never the selected native route.
pub(crate) struct RoutingGraph {
    graph: DiGraph<String, (String, u128)>,
    index: BTreeMap<String, NodeIndex>,
    switches: BTreeSet<NodeIndex>,
}
impl RoutingGraph {
    pub(crate) fn new(sim: &Simulation) -> Result<Self> {
        sim.active()?;
        let mut graph = DiGraph::new();
        let mut index = BTreeMap::new();
        let mut switches = BTreeSet::new();
        let mut nodes: Vec<_> = sim
            .nodes
            .values()
            .filter(|n| n.state == State::Active)
            .collect();
        nodes.sort_by_key(|n| &n.spec.name);
        for n in nodes {
            let ix = graph.add_node(n.id.clone());
            index.insert(n.id.clone(), ix);
            if n.spec.role == Role::Switch {
                switches.insert(ix);
            }
        }
        let mut links: Vec<_> = sim.links.values().collect();
        links.sort_by_key(|l| {
            let a = &sim.interfaces[&l.interfaces[0]];
            let b = &sim.interfaces[&l.interfaces[1]];
            (
                sim.nodes[&a.node].spec.name.clone(),
                a.name.clone(),
                sim.nodes[&b.node].spec.name.clone(),
                b.name.clone(),
            )
        });
        for link in links {
            link.spec.validate()?;
            let a = sim.interface(&link.interfaces[0])?;
            let b = sim.interface(&link.interfaces[1])?;
            if !link.spec.up || a.interface_type != InterfaceType::Data {
                continue;
            }
            if let (Some(&x), Some(&y)) = (index.get(&a.node), index.get(&b.node)) {
                let cost = u128::from(link.spec.latency_ns) * (graph.node_count() as u128 + 1) + 1;
                let weight = (link.id.clone(), cost);
                graph.add_edge(x, y, weight.clone());
                graph.add_edge(y, x, weight);
            }
        }
        Ok(Self {
            graph,
            index,
            switches,
        })
    }
    fn predecessors(&self, source: &str) -> Result<BTreeMap<NodeIndex, (NodeIndex, String)>> {
        let start = *self
            .index
            .get(source)
            .ok_or_else(|| Error::State("source node is inactive".into()))?;
        let mut distances = BTreeMap::from([(start, 0u128)]);
        let mut previous = BTreeMap::new();
        let mut pending = BinaryHeap::from([Reverse((0u128, start))]);
        while let Some(Reverse((cost, node))) = pending.pop() {
            if cost != distances[&node] || (node != start && !self.switches.contains(&node)) {
                continue;
            }
            for edge in self.graph.edges(node) {
                let next = edge.target();
                let distance = cost + edge.weight().1;
                if distances.get(&next).is_none_or(|old| distance < *old) {
                    distances.insert(next, distance);
                    previous.insert(next, (node, edge.weight().0.clone()));
                    pending.push(Reverse((distance, next)));
                }
            }
        }
        Ok(previous)
    }
    #[cfg(feature = "linux")]
    pub(crate) fn routes(&self, source: &str) -> Result<BTreeMap<String, String>> {
        let previous = self.predecessors(source)?;
        let start = self.index[source];
        // Memoize first hops, so long paths are not reconstructed for every destination.
        let mut first: BTreeMap<NodeIndex, String> = BTreeMap::new();
        for &dest in previous.keys() {
            let mut node = dest;
            let mut trail = Vec::new();
            let link = loop {
                if let Some(link) = first.get(&node) {
                    break link.clone();
                }
                let (parent, link) = &previous[&node];
                trail.push(node);
                if *parent == start {
                    break link.clone();
                }
                node = *parent;
            };
            for node in trail {
                first.insert(node, link.clone());
            }
        }
        Ok(first
            .into_iter()
            .map(|(n, l)| (self.graph[n].clone(), l))
            .collect())
    }
    fn route(&self, source: &str, destination: &str) -> Result<Vec<String>> {
        let goal = *self
            .index
            .get(destination)
            .ok_or_else(|| Error::State("destination node is inactive".into()))?;
        let start = *self
            .index
            .get(source)
            .ok_or_else(|| Error::State("source node is inactive".into()))?;
        let previous = self.predecessors(source)?;
        let mut node = goal;
        let mut links = Vec::new();
        while node != start {
            let (parent, link) = previous
                .get(&node)
                .ok_or_else(|| Error::NoRoute(source.into(), destination.into()))?;
            links.push(link.clone());
            node = *parent;
        }
        links.reverse();
        Ok(links)
    }
}
impl Simulation {
    /// Static minimum-latency routing, shared with Linux; hosts cannot carry transit.
    pub fn route(&self, source: &str, destination: &str, bytes: usize) -> Result<Vec<String>> {
        self.node(source)?;
        self.node(destination)?;
        if bytes > 16 * 1024 * 1024 {
            return Err(Error::Invalid("transfer exceeds 16 MiB".into()));
        }
        RoutingGraph::new(self)?.route(source, destination)
    }
    /// Reachability through switch components, with direct host links handled separately.
    /// Hosts attached to several components must never merge those components.
    pub(crate) fn validate_rank_connectivity(&self, ranks: &[String]) -> Result<()> {
        let graph = RoutingGraph::new(self)?;
        let mut components = petgraph::unionfind::UnionFind::new(graph.graph.node_count());
        for edge in graph.graph.edge_references() {
            if graph.switches.contains(&edge.source()) && graph.switches.contains(&edge.target()) {
                components.union(edge.source().index(), edge.target().index());
            }
        }
        let mut attachments = Vec::new();
        let mut direct = BTreeSet::new();
        for rank in ranks {
            let node = *graph
                .index
                .get(rank)
                .ok_or_else(|| Error::State("rank is inactive".into()))?;
            let mut networks = BTreeSet::new();
            for edge in graph.graph.edges(node) {
                if graph.switches.contains(&edge.target()) {
                    networks.insert(components.find(edge.target().index()));
                } else {
                    direct.insert((rank.clone(), graph.graph[edge.target()].clone()));
                }
            }
            attachments.push(networks);
        }
        if let Some(first) = attachments.first() {
            let common = attachments
                .iter()
                .skip(1)
                .fold(first.clone(), |a, b| a.intersection(b).copied().collect());
            if !common.is_empty() {
                return Ok(());
            }
        }
        for (i, a) in ranks.iter().enumerate() {
            for (j, b) in ranks.iter().enumerate().skip(i + 1) {
                if attachments[i].is_disjoint(&attachments[j])
                    && !direct.contains(&(a.clone(), b.clone()))
                {
                    return Err(Error::NoRoute(a.clone(), b.clone()));
                }
            }
        }
        Ok(())
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
    /// Submit in nondecreasing timestamp order; equal times model concurrent contention.
    /// Work is store-and-forward, full duplex, and FIFO on each directed link.
    pub fn schedule_transfer(
        &mut self,
        source: &str,
        destination: &str,
        bytes: usize,
        at_ns: u64,
    ) -> Result<Transmission> {
        if at_ns < self.clock_ns.max(self.scheduling_frontier_ns) {
            return Err(Error::Invalid(
                "submission precedes completed time or the monotonic scheduling frontier".into(),
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
        self.scheduling_frontier_ns = at_ns;
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
