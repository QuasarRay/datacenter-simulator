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

//! Real kernel topology: one patchbay device namespace per simulated node,
//! one isolated bridge per physical link, and petgraph-derived static routes.
//! Link routers have their IX interfaces disabled, preventing hidden shortcuts.
use crate::model::{InterfaceType, Role, Simulation, State};
use anyhow::{Context, Result, anyhow, bail};
use patchbay::{Device, IfaceConfig, Lab, RouterPreset};
use std::{collections::BTreeMap, net::Ipv4Addr, process::Command, time::Duration};

/// Dedicated routable endpoint selected by native NCCL Socket transport.
pub const NCCL_INTERFACE: &str = "simnccl";

pub struct LinuxFabric {
    // Devices and routers hold lab references; all handles are dropped together.
    devices: BTreeMap<String, Device>,
    loopbacks: BTreeMap<String, Ipv4Addr>,
    pub(crate) addresses: BTreeMap<String, Ipv4Addr>,
    pub(crate) simulation: Simulation,
    _lab: Lab,
    pub(crate) failed: bool,
    tool_identities: BTreeMap<String, crate::evidence::FileIdentity>,
}
pub(crate) fn command(device: &Device, program: &str, args: &[String]) -> Result<()> {
    let program = crate::evidence::resolve(program)?;
    let args = args.to_vec();
    device.run_sync(move || {
        let output = Command::new(&program).args(args).output()?;
        if !output.status.success() {
            bail!(
                "{}: {}",
                program.display(),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(())
    })
}
pub(crate) fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|s| s.to_string()).collect()
}
impl LinuxFabric {
    /// Call patchbay::init_userns() in main before creating any runtime or threads.
    pub async fn build(simulation: &Simulation) -> Result<Self> {
        if simulation.nodes().any(|n| {
            n.spec
                .labels
                .get("simulator.medium")
                .and_then(|v| v.as_str())
                == Some("infiniband")
        }) {
            bail!(
                "InfiniBand intended state requires the ibsim management backend; it is not an Ethernet/NCCL Socket fabric"
            );
        }
        if simulation.state() != State::Active {
            bail!("start the model before building its Linux topology");
        }
        if simulation.links().count() > 240 {
            bail!("Linux backend supports at most 240 links");
        }
        if simulation
            .interfaces()
            .any(|i| i.interface_type != InterfaceType::Data || !i.split_children.is_empty())
        {
            bail!(
                "Linux backend supports unsplit data interfaces; OOB/PCIe stay in the portable model"
            );
        }
        if simulation.interfaces().any(|i| i.name == NCCL_INTERFACE) {
            bail!("simnccl is reserved for the NCCL network endpoint");
        }
        for link in simulation.links() {
            link.spec.native_queue_packets()?;
        }
        let tool_identities = crate::evidence::tools(
            &["ip", "tc", "nft", "sysctl", "ping"].map(std::path::Path::new),
        )?;
        let lab = Lab::new().await.context("create patchbay lab")?;
        let mut routers = BTreeMap::new();
        let addresses = Self::link_addresses(simulation)?;
        let mut links: Vec<_> = simulation.links().collect();
        links.sort_by_cached_key(|link| endpoint_key(simulation, &link.interfaces));
        for (index, link) in links.into_iter().enumerate() {
            // Unique /24 with two endpoints and a bridge address, no shared transit fabric.
            let base = u32::from(Ipv4Addr::new(10, 64, 0, 0)) + (index as u32) * 256;
            let subnet = format!("{}/24", Ipv4Addr::from(base));
            let router = lab
                .add_router(&format!("wire{index}"))
                .preset(RouterPreset::PublicV4)
                .downstream_cidr(subnet.parse()?)
                .build()
                .await
                .with_context(|| format!("create link bridge {index} ({subnet})"))?;
            routers.insert(link.id.clone(), router);
        }
        let mut devices = BTreeMap::new();
        let mut loopbacks = BTreeMap::new();
        let mut nodes: Vec<_> = simulation.nodes().collect();
        nodes.sort_by_key(|n| &n.spec.name);
        for (index, node) in nodes.into_iter().enumerate() {
            let mut builder = lab
                .add_device(&node.spec.name)
                .iface(NCCL_INTERFACE, IfaceConfig::dummy());
            let mut attached = false;
            for interface in simulation.interfaces().filter(|i| i.node == node.id) {
                let link = simulation
                    .links()
                    .find(|l| l.interfaces.contains(&interface.id));
                let config = if let Some(link) = link {
                    let ip = addresses[&interface.id];

                    let mut c = IfaceConfig::routed(routers[&link.id].id())
                        .addr(format!("{ip}/24").parse()?);
                    if !link.spec.up {
                        c = c.down();
                    }
                    c
                } else {
                    IfaceConfig::dummy()
                };
                builder = builder.iface(&interface.name, config);
                attached = true;
            }
            if !attached {
                builder = builder.iface("sim0", IfaceConfig::dummy());
            }
            let dev = builder
                .build()
                .await
                .with_context(|| format!("create node {}", node.spec.name))?;
            command(&dev, "ip", &args(&["route", "flush", "default"]))?;
            command(
                &dev,
                "sysctl",
                &args(&[
                    "-qw",
                    if node.spec.role == Role::Switch {
                        "net.ipv4.ip_forward=1"
                    } else {
                        "net.ipv4.ip_forward=0"
                    },
                    "net.ipv4.conf.all.rp_filter=0",
                    "net.ipv4.conf.default.rp_filter=0",
                ]),
            )?;
            for interface in simulation.interfaces().filter(|i| i.node == node.id) {
                command(
                    &dev,
                    "sysctl",
                    &args(&[
                        "-qw",
                        &format!("net.ipv4.conf.{}.rp_filter=0", interface.name),
                    ]),
                )?;
                command(
                    &dev,
                    "ip",
                    &args(&[
                        "link",
                        "set",
                        "dev",
                        &interface.name,
                        "address",
                        &interface.mac_address,
                        "mtu",
                        "1500",
                    ]),
                )?;
                if let Some(link) = simulation
                    .links()
                    .find(|l| l.interfaces.contains(&interface.id))
                {
                    // patchbay's LinkCondition latency is integer milliseconds. tc accepts
                    // fractional microseconds, preserving the input datacenter-scale delay.
                    command(
                        &dev,
                        "tc",
                        &args(&[
                            "qdisc",
                            "replace",
                            "dev",
                            &interface.name,
                            "root",
                            "netem",
                            "delay",
                            &format!("{:.3}us", link.spec.latency_ns as f64 / 1000.0),
                            "rate",
                            &format!("{}bit", link.spec.bandwidth_bps),
                            "limit",
                            &link.spec.native_queue_packets()?.to_string(),
                        ]),
                    )?;
                }
            }
            let ip = Ipv4Addr::from(u32::from(Ipv4Addr::new(10, 255, 0, 1)) + index as u32);
            command(
                &dev,
                "ip",
                &args(&["address", "add", &format!("{ip}/32"), "dev", NCCL_INTERFACE]),
            )?;
            loopbacks.insert(node.id.clone(), ip);
            devices.insert(node.id.clone(), dev);
        }
        for router in routers.values() {
            router.run_sync(|| {
                let output = Command::new("ip")
                    .args(["link", "set", "ix", "down"])
                    .output()?;
                if !output.status.success() {
                    bail!(
                        "disable patchbay IX: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                Ok(())
            })?;
        }
        let fabric = Self {
            tool_identities,
            devices,
            loopbacks,
            addresses,
            simulation: simulation.clone(),
            _lab: lab,
            failed: false,
        };
        fabric.configure_routes()?;
        Ok(fabric)
    }
    fn configure_routes(&self) -> Result<()> {
        self.configure_model_routes(&self.simulation)
    }
    fn configure_model_routes(&self, model: &Simulation) -> Result<()> {
        let graph = crate::topology::RoutingGraph::new(model)?;
        for (source, dev) in &self.devices {
            // Protocol 186 marks only this simulator's static routes.
            command(dev, "ip", &args(&["route", "flush", "proto", "186"]))?;
            let routes = graph.routes(source)?;
            for (destination, ip) in &self.loopbacks {
                if source == destination {
                    continue;
                }
                let Some(first) = routes.get(destination) else {
                    continue;
                };
                let link = model
                    .links()
                    .find(|l| &l.id == first)
                    .context("route references missing link")?;
                let a = self.simulation.interface(&link.interfaces[0])?;
                let b = self.simulation.interface(&link.interfaces[1])?;
                let (from, to) = if &a.node == source { (a, b) } else { (b, a) };
                command(
                    dev,
                    "ip",
                    &args(&[
                        "route",
                        "replace",
                        &format!("{ip}/32"),
                        "via",
                        &self.addresses[&to.id].to_string(),
                        "dev",
                        &from.name,
                        "proto",
                        "186",
                        "src",
                        &self.loopbacks[source].to_string(),
                    ]),
                )?;
            }
        }
        Ok(())
    }
    pub fn tool_identities(&self) -> &BTreeMap<String, crate::evidence::FileIdentity> {
        &self.tool_identities
    }
    pub fn address(&self, node: &str) -> Result<Ipv4Addr> {
        self.loopbacks
            .get(node)
            .copied()
            .ok_or_else(|| anyhow!("unknown node {node}"))
    }
    pub async fn ping(&self, source: &str, destination: &str) -> Result<()> {
        self.ensure_healthy()?;
        let dev = self.devices.get(source).context("unknown source")?;
        let target = self.address(destination)?.to_string();
        let mut cmd = tokio::process::Command::new("ping");
        cmd.args(["-n", "-c", "1", "-W", "2", &target]);
        cmd.kill_on_drop(true);
        let mut child = dev.spawn_command(cmd)?;
        let status = tokio::time::timeout(Duration::from_secs(4), child.wait()).await??;
        if !status.success() {
            bail!("ping {source} -> {destination} failed");
        }
        Ok(())
    }
    /// Run real Rust socket code inside a simulated node's namespace.
    pub(crate) fn device(&self, node: &str) -> Result<&Device> {
        self.ensure_healthy()?;
        self.devices
            .get(node)
            .ok_or_else(|| anyhow!("unknown node {node}"))
    }
    /// Escape hatch for experiments. Permanently invalidates this instance:
    /// retained native handles can mutate it at any later time. Build a new lab
    /// before asking for synchronized routing or collective evidence again.
    pub fn raw_device(&mut self, node: &str) -> Result<&Device> {
        self.ensure_healthy()?;
        if !self.devices.contains_key(node) {
            bail!("unknown node {node}");
        }
        self.failed = true;
        Ok(&self.devices[node])
    }
    pub(crate) fn ensure_healthy(&self) -> Result<()> {
        if self.failed {
            bail!("fabric requires teardown after raw native access or failed rollback");
        }
        Ok(())
    }
    // Synchronous transaction: no cancellation point between kernel writes and rollback.
    pub(crate) fn set_link_up_sync(&mut self, link_id: &str, up: bool) -> Result<()> {
        let mut spec = self
            .simulation
            .links()
            .find(|l| l.id == link_id)
            .context("unknown link")?
            .spec;
        spec.up = up;
        self.set_link_spec_sync(link_id, spec)
    }
    fn set_link_spec_sync(&mut self, link_id: &str, spec: crate::model::LinkSpec) -> Result<()> {
        self.ensure_healthy()?;
        spec.validate()?;
        spec.native_queue_packets()?;
        let link = self
            .simulation
            .links()
            .find(|l| l.id == link_id)
            .context("unknown link")?
            .clone();
        if link.spec == spec {
            return Ok(());
        }
        let mut next = self.simulation.clone();
        next.set_link(link_id, spec)?;
        let result = apply_or_restore(5, |step, restoring| {
            if step == 4 {
                return self.configure_model_routes(if restoring {
                    &self.simulation
                } else {
                    &next
                });
            }
            let interface = self.simulation.interface(&link.interfaces[step % 2])?;
            let target = if restoring { link.spec } else { spec };
            if step < 2 {
                return command(
                    &self.devices[&interface.node],
                    "tc",
                    &args(&[
                        "qdisc",
                        "replace",
                        "dev",
                        &interface.name,
                        "root",
                        "netem",
                        "delay",
                        &format!("{:.3}us", target.latency_ns as f64 / 1000.0),
                        "rate",
                        &format!("{}bit", target.bandwidth_bps),
                        "limit",
                        &target.native_queue_packets()?.to_string(),
                    ]),
                );
            }
            command(
                &self.devices[&interface.node],
                "ip",
                &args(&[
                    "link",
                    "set",
                    "dev",
                    &interface.name,
                    if target.up { "up" } else { "down" },
                ]),
            )
        });
        if let Err((error, poisoned)) = result {
            self.failed = poisoned;
            return Err(error);
        }
        self.simulation = next;
        Ok(())
    }
    pub async fn set_link_up(&mut self, link_id: &str, up: bool) -> Result<()> {
        self.set_link_up_sync(link_id, up)
    }
    /// Atomically update rate, latency, queue capacity, state and derived routes.
    pub async fn set_link_spec(
        &mut self,
        link_id: &str,
        spec: crate::model::LinkSpec,
    ) -> Result<()> {
        self.set_link_spec_sync(link_id, spec)
    }
    pub fn model(&self) -> &Simulation {
        &self.simulation
    }
    pub fn link_addresses(model: &Simulation) -> Result<BTreeMap<String, Ipv4Addr>> {
        let mut links: Vec<_> = model.links().collect();
        if links.len() > 240 {
            bail!("Linux backend supports at most 240 links");
        }
        links.sort_by_cached_key(|link| endpoint_key(model, &link.interfaces));
        let mut addresses = BTreeMap::new();
        for (index, link) in links.into_iter().enumerate() {
            let mut ends = link.interfaces.clone();
            ends.sort_by_key(|i| {
                let i = &model.interfaces[i];
                (&model.nodes[&i.node].spec.name, &i.name)
            });
            for (end, id) in ends.into_iter().enumerate() {
                addresses.insert(
                    id,
                    Ipv4Addr::from(
                        u32::from(Ipv4Addr::new(10, 64, 0, 0))
                            + index as u32 * 256
                            + 2
                            + end as u32,
                    ),
                );
            }
        }
        Ok(addresses)
    }
    /// Execute a bounded TCP transfer without exposing namespace control.
    pub async fn tcp_transfer(
        &self,
        source: &str,
        destination: &str,
        payload: Vec<u8>,
    ) -> Result<Vec<u8>> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        self.ensure_healthy()?;
        if payload.len() > 1024 * 1024 {
            bail!("TCP probe exceeds 1 MiB");
        }
        let target = self.address(destination)?;
        let count = payload.len();
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let mut server = self.device(destination)?.spawn(move |_| async move {
            let listener = tokio::net::TcpListener::bind((target, 0)).await?;
            ready_tx
                .send(listener.local_addr()?)
                .map_err(|_| anyhow!("client disappeared"))?;
            let (mut stream, _) = listener.accept().await?;
            let mut bytes = vec![0; count];
            stream.read_exact(&mut bytes).await?;
            Ok::<_, anyhow::Error>(bytes)
        })?;
        struct Abort(tokio::task::AbortHandle);
        impl Drop for Abort {
            fn drop(&mut self) {
                self.0.abort();
            }
        }
        let _server = Abort(server.abort_handle());
        tokio::time::timeout(Duration::from_secs(10), async {
            let address = ready_rx.await?;
            let mut client = self.device(source)?.spawn(move |_| async move {
                let mut stream = tokio::net::TcpStream::connect(address).await?;
                stream.write_all(&payload).await?;
                Ok::<_, anyhow::Error>(())
            })?;
            let _client = Abort(client.abort_handle());
            (&mut client).await??;
            (&mut server).await?
        })
        .await?
    }
}

fn endpoint_key(model: &Simulation, interfaces: &[String; 2]) -> [(String, String); 2] {
    let mut ends = interfaces.clone().map(|id| {
        let i = &model.interfaces[&id];
        (model.nodes[&i.node].spec.name.clone(), i.name.clone())
    });
    ends.sort();
    ends
}

/// Attempt every restoration even when one fails; callers must stop using a
/// resource when restoration cannot be established.
fn apply_or_restore(
    steps: usize,
    mut operation: impl FnMut(usize, bool) -> Result<()>,
) -> std::result::Result<(), (anyhow::Error, bool)> {
    for step in 0..steps {
        if let Err(error) = operation(step, false) {
            let mut failures = Vec::new();
            for restore in 0..steps {
                if let Err(e) = operation(restore, true) {
                    failures.push(format!("{e:#}"));
                }
            }
            return Err((
                error.context(format!("rollback errors: {failures:?}")),
                !failures.is_empty(),
            ));
        }
    }
    Ok(())
}
#[cfg(test)]
mod transaction_tests {
    use super::*;
    #[test]
    fn failures_at_each_mutation_restore_all_state_and_poison_on_cleanup_failure() {
        for failure in 0..5 {
            let mut state = [false; 5];
            let mut restored = Vec::new();
            let result = apply_or_restore(5, |step, restoring| {
                if restoring {
                    restored.push(step);
                    state[step] = false;
                } else {
                    state[step] = true;
                    if step == failure {
                        bail!("injected kernel failure");
                    }
                }
                Ok(())
            });
            assert!(!result.unwrap_err().1);
            assert_eq!(state, [false; 5]);
            assert_eq!(restored, vec![0, 1, 2, 3, 4]);
        }
        let mut restored = Vec::new();
        let result = apply_or_restore(5, |step, restoring| {
            if restoring {
                restored.push(step);
            }
            if step == 0 {
                bail!("injected persistent failure");
            }
            Ok(())
        });
        assert!(result.unwrap_err().1);
        assert_eq!(restored, vec![0, 1, 2, 3, 4]);
    }
}
