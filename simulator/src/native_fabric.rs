// SPDX-License-Identifier: RPL-1.5
//! Compile NetBox intent into owned Incus topology and native guest configuration.
//! The IPv4 pool is a lab overlay; this never writes production NetBox/IPAM state.
use crate::{
    incus,
    manifest::Manifest,
    model::Role,
    netbox::{self, Medium, NetBoxConfig, Snapshot},
};
use anyhow::{Context, Result, ensure};
use petgraph::{algo::astar, graph::DiGraph};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    net::Ipv4Addr,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FabricConfig {
    pub netbox: NetBoxConfig,
    pub project: String,
    pub socket: PathBuf,
    pub storage_pool: String,
    pub image_fingerprint: String,
    pub max_memory_mib: u64,
    pub ipv4_pool: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct NetworkFile {
    pub name: String,
    pub content: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct GuestConfig {
    pub ncp_network_files: BTreeMap<String, Vec<NetworkFile>>,
    pub ncp_forwarding: BTreeMap<String, bool>,
    pub ncp_addresses: BTreeMap<String, String>,
}

fn parts(endpoint: &str) -> Result<(&str, &str)> {
    endpoint
        .split_once(':')
        .context("expected validated node:interface endpoint")
}

/// One /30 per physical cable. Only switch-role nodes may forward transit traffic.
/// Routes are generated from actual up edges. No hidden management/NAT path exists.
pub fn network(manifest: &Manifest, pool: &str) -> Result<GuestConfig> {
    let (address, prefix) = pool.split_once('/').context("expected IPv4 CIDR pool")?;
    let prefix: u32 = prefix.parse()?;
    ensure!(
        (8..=30).contains(&prefix),
        "lab pool prefix must be /8 through /30"
    );
    let base = u32::from(address.parse::<Ipv4Addr>()?) as u64;
    let size = 1u64 << (32 - prefix);
    ensure!(
        base.is_multiple_of(size),
        "pool must be a canonical network address"
    );
    let mut consumed = 0;
    let mut addresses = BTreeMap::new();
    let mut prefixes = Vec::new();
    let mut endpoint_links = BTreeMap::new();
    for (index, link) in manifest.content.links.iter().enumerate() {
        let next = ncp_assessment_kernel::reserve(consumed, 4, size);
        ensure!(next > consumed, "IPv4 pool cannot hold every cable /30");
        let subnet = base + consumed;
        prefixes.push(format!("{}/30", Ipv4Addr::from(subnet as u32)));
        for (side, endpoint) in link.endpoints.iter().enumerate() {
            let (node, port) = parts(endpoint)?;
            let spec = manifest
                .content
                .nodes
                .get(node)
                .context("missing endpoint node")?;
            ensure!(
                spec.interfaces.contains_key(port),
                "missing endpoint interface"
            );
            ensure!(
                endpoint_links.insert(endpoint.clone(), index).is_none(),
                "port belongs to two cables"
            );
            addresses.insert(
                endpoint.clone(),
                Ipv4Addr::from((subnet + side as u64 + 1) as u32).to_string(),
            );
        }
        consumed = next;
    }
    ensure!(
        !prefixes.is_empty(),
        "native fabric needs at least one cable"
    );
    let mut files = BTreeMap::new();
    let forwarding = manifest
        .content
        .nodes
        .iter()
        .map(|(n, s)| (n.clone(), s.role == Role::Switch))
        .collect();
    for (source, spec) in &manifest.content.nodes {
        let mut graph = DiGraph::<&str, (String, String)>::new();
        let nodes: BTreeMap<_, _> = manifest
            .content
            .nodes
            .keys()
            .map(|n| (n.as_str(), graph.add_node(n)))
            .collect();
        for link in &manifest.content.links {
            if !link.condition.up {
                continue;
            }
            for side in 0..2 {
                let from = &link.endpoints[side];
                let to = &link.endpoints[1 - side];
                let (a, _) = parts(from)?;
                let (b, _) = parts(to)?;
                if a == source || manifest.content.nodes[a].role == Role::Switch {
                    graph.add_edge(nodes[a], nodes[b], (from.clone(), addresses[to].clone()));
                }
            }
        }
        let mut routes: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
        for (index, target) in manifest.content.links.iter().enumerate() {
            if !target.condition.up
                || target
                    .endpoints
                    .iter()
                    .any(|e| parts(e).is_ok_and(|(n, _)| n == source))
            {
                continue;
            }
            let best = target
                .endpoints
                .iter()
                .filter_map(|end| {
                    let (node, _) = parts(end).ok()?;
                    astar(
                        &graph,
                        nodes[source.as_str()],
                        |n| n == nodes[node],
                        |_| 1usize,
                        |_| 0,
                    )
                })
                .min_by_key(|(cost, path)| {
                    (*cost, path.iter().map(|n| n.index()).collect::<Vec<_>>())
                });
            if let Some((_, path)) = best {
                let edge = graph
                    .find_edge(path[0], path[1])
                    .context("route has no first hop")?;
                let (endpoint, gateway) = &graph[edge];
                let (_, port) = parts(endpoint)?;
                routes
                    .entry(port.into())
                    .or_default()
                    .push((prefixes[index].clone(), gateway.clone()));
            }
        }
        let mut node_files = Vec::new();
        for port in spec.interfaces.keys() {
            let endpoint = format!("{source}:{port}");
            let mut content = format!(
                "[Match]\nName={port}\n[Network]\nLinkLocalAddressing=no\nIPv6AcceptRA=no\n"
            );
            if let Some(address) = addresses.get(&endpoint) {
                content.push_str(&format!("Address={address}/30\n"));
            }
            if let Some(entries) = routes.get(port) {
                for (destination, gateway) in entries {
                    content.push_str(&format!(
                        "[Route]\nDestination={destination}\nGateway={gateway}\n"
                    ));
                }
            }
            node_files.push(NetworkFile {
                name: port.clone(),
                content,
            });
        }
        files.insert(source.clone(), node_files);
    }
    Ok(GuestConfig {
        ncp_network_files: files,
        ncp_forwarding: forwarding,
        ncp_addresses: addresses,
    })
}

/// Render only into a new directory. READY.json is the last write and seals inputs.
pub fn render(config: &FabricConfig, snapshot: &Snapshot, directory: &Path) -> Result<()> {
    ensure!(
        config.netbox.medium == Medium::Ethernet,
        "native Ethernet cannot impersonate InfiniBand"
    );
    let manifest = netbox::compile(&config.netbox, snapshot)?;
    let guests = network(&manifest, &config.ipv4_pool)?;
    // Prove/account before any daemon call or even artifact-directory creation.
    let mut used = 0;
    for node in manifest.content.nodes.values() {
        let next = ncp_assessment_kernel::reserve(used, node.memory, config.max_memory_mib);
        ensure!(next > used, "native memory budget exceeded");
        used = next;
    }
    fs::create_dir(directory).context("native plan destination must be new")?;
    let directory = fs::canonicalize(directory)?;
    let native = incus::Config {
        project: config.project.clone(),
        socket: config.socket.clone(),
        storage_pool: config.storage_pool.clone(),
        image_fingerprint: config.image_fingerprint.clone(),
        max_memory_mib: config.max_memory_mib,
        manifest: directory.join("manifest.json"),
    };
    let mut hashes = BTreeMap::<String, String>::new();
    // Declarative emission prevents artifact/hash bookkeeping from diverging.
    macro_rules! emit {
        ($name:literal, $value:expr) => {{
            let bytes = serde_json::to_vec_pretty(&$value)?;
            hashes.insert($name.into(), format!("{:x}", Sha256::digest(&bytes)));
            fs::write(directory.join($name), bytes)?;
        }};
    }
    emit!("snapshot.json", snapshot);
    emit!("manifest.json", manifest);
    emit!("provisioning-vars.json", guests);
    emit!("incus-config.json", native);
    emit!("fabric-config.json", config);
    let plan = native.plan()?;
    emit!("native-plan.json", plan);
    let ready: Value = json!({"schema":"ncp-native-plan-v1","sha256":hashes,
        "memory_mib":used,"netbox_is_read_only":true,"routing":"static-shortest-path; switch-only transit",
        "source_api":snapshot.api_url,"source_time":snapshot.fetched_at});
    fs::write(
        directory.join("READY.json"),
        serde_json::to_vec_pretty(&ready)?,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Manifest {
        serde_json::from_str(include_str!("../../integrations/incus/topology.json")).unwrap()
    }
    #[test]
    fn routed_star_compiles_every_cable_without_host_transit() {
        let m = fixture();
        let plan = network(&m, "10.88.0.0/24").unwrap();
        assert_eq!(plan.ncp_addresses.len(), 8);
        assert_eq!(plan.ncp_forwarding.values().filter(|v| **v).count(), 1);
        let a = &plan.ncp_network_files["compute-a"][0].content;
        assert_eq!(a.matches("[Route]").count(), 3);
        let mut cut = m;
        cut.content.links[1].condition.up = false;
        let p = network(&cut, "10.88.0.0/24").unwrap();
        assert!(
            !p.ncp_network_files["compute-a"][0]
                .content
                .contains("[Route]")
        );
        assert!(network(&cut, "10.88.0.0/30").is_err());
        assert!(network(&cut, "10.88.0.1/24").is_err());
    }
    #[test]
    fn duplicate_port_is_rejected_and_top_of_ipv4_space_is_bounded() {
        let mut m = fixture();
        m.content.links[1].endpoints[0] = m.content.links[0].endpoints[0].clone();
        assert!(network(&m, "10.88.0.0/24").is_err());
        let p = network(&fixture(), "255.255.255.0/24").unwrap();
        assert!(
            p.ncp_addresses
                .values()
                .all(|a| a.starts_with("255.255.255."))
        );
    }
}
