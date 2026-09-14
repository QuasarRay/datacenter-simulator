//! Real kernel topology: one patchbay device namespace per simulated node,
//! one isolated bridge per physical link, and petgraph-derived static routes.
//! Link routers have their IX interfaces disabled, preventing hidden shortcuts.
use crate::{
    model::{InterfaceType, Role, Simulation, State},
    topology::Transmission,
};
use anyhow::{Context, Result, anyhow, bail};
use patchbay::{Device, IfaceConfig, Lab, RouterPreset};
use std::{collections::BTreeMap, net::Ipv4Addr, process::Command, time::Duration};

pub struct LinuxFabric {
    // Devices and routers hold lab references; all handles are dropped together.
    devices: BTreeMap<String, Device>,
    loopbacks: BTreeMap<String, Ipv4Addr>,
    addresses: BTreeMap<String, Ipv4Addr>,
    simulation: Simulation,
    _lab: Lab,
}
fn command(device: &Device, program: &str, args: &[String]) -> Result<()> {
    let program = program.to_owned();
    let args = args.to_vec();
    device.run_sync(move || {
        let output = Command::new(&program).args(args).output()?;
        if !output.status.success() {
            bail!("{program}: {}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    })
}
fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|s| s.to_string()).collect()
}
impl LinuxFabric {
    /// Call patchbay::init_userns() in main before creating any runtime or threads.
    pub async fn build(simulation: &Simulation) -> Result<Self> {
        if simulation.state() != State::Active {
            bail!("start the model before building its Linux topology");
        }
        if simulation.links().count() > 8192 {
            bail!("Linux backend supports at most 8192 links");
        }
        if simulation
            .interfaces()
            .any(|i| i.interface_type != InterfaceType::Data || !i.split_children.is_empty())
        {
            bail!(
                "Linux backend supports unsplit data interfaces; OOB/PCIe stay in the portable model"
            );
        }
        let lab = Lab::new().await.context("create patchbay lab")?;
        let mut routers = BTreeMap::new();
        let mut addresses = BTreeMap::new();
        for (index, link) in simulation.links().enumerate() {
            // Unique /29 with two endpoints and a bridge address, no shared transit fabric.
            let base = u32::from(Ipv4Addr::new(10, 64, 0, 0)) + (index as u32) * 8;
            let subnet = format!("{}/29", Ipv4Addr::from(base));
            let router = lab
                .add_router(&format!("wire{index}"))
                .preset(RouterPreset::PublicV4)
                .downstream_cidr(subnet.parse()?)
                .build()
                .await.with_context(||format!("create link bridge {index} ({subnet})"))?;
            for (end, interface) in link.interfaces.iter().enumerate() {
                addresses.insert(interface.clone(), Ipv4Addr::from(base + 2 + end as u32));
            }
            routers.insert(link.id.clone(), router);
        }
        let mut devices = BTreeMap::new();
        let mut loopbacks = BTreeMap::new();
        let mut nodes: Vec<_> = simulation.nodes().collect();
        nodes.sort_by_key(|n| &n.spec.name);
        for (index, node) in nodes.into_iter().enumerate() {
            let mut builder = lab.add_device(&node.spec.name);
            let mut attached = false;
            for interface in simulation.interfaces().filter(|i| i.node == node.id) {
                let link = simulation
                    .links()
                    .find(|l| l.interfaces.contains(&interface.id));
                let config = if let Some(link) = link {
                    let ip = addresses[&interface.id];

                    let mut c = IfaceConfig::routed(routers[&link.id].id())
                        .addr(format!("{ip}/29").parse()?);
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
            let dev = builder.build().await.with_context(||format!("create node {}",node.spec.name))?;
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
                            "10000",
                        ]),
                    )?;
                }
            }
            let ip = Ipv4Addr::from(u32::from(Ipv4Addr::new(10, 255, 0, 1)) + index as u32);
            command(
                &dev,
                "ip",
                &args(&["address", "add", &format!("{ip}/32"), "dev", "lo"]),
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
            devices,
            loopbacks,
            addresses,
            simulation: simulation.clone(),
            _lab: lab,
        };
        fabric.configure_routes()?;
        Ok(fabric)
    }
    fn configure_routes(&self) -> Result<()> {
        for (source, dev) in &self.devices {
            // Protocol 186 marks only this simulator's static routes.
            command(dev, "ip", &args(&["route", "flush", "proto", "186"]))?;
            for (destination, ip) in &self.loopbacks {
                if source == destination {
                    continue;
                }
                let Ok(path) = self.simulation.route(source, destination, 0) else {
                    continue;
                };
                let Some(first) = path.first() else {
                    continue;
                };
                let link = self
                    .simulation
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
                    ]),
                )?;
            }
        }
        Ok(())
    }
    pub fn address(&self, node: &str) -> Result<Ipv4Addr> {
        self.loopbacks
            .get(node)
            .copied()
            .ok_or_else(|| anyhow!("unknown node {node}"))
    }
    pub async fn ping(&self, source: &str, destination: &str) -> Result<()> {
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
    pub fn device(&self, node: &str) -> Result<&Device> {
        self.devices
            .get(node)
            .ok_or_else(|| anyhow!("unknown node {node}"))
    }
    pub async fn set_link_up(&mut self, link_id: &str, up: bool) -> Result<()> {
        let link = self
            .simulation
            .links()
            .find(|l| l.id == link_id)
            .context("unknown link")?
            .clone();
        for id in &link.interfaces {
            let interface = self.simulation.interface(id)?;
            let iface = self.devices[&interface.node]
                .iface(&interface.name)
                .context("missing kernel interface")?;
            if up {
                iface.link_up().await?;
            } else {
                iface.link_down().await?;
            }
        }
        let mut spec = link.spec;
        spec.up = up;
        self.simulation.set_link(link_id, spec)?;
        self.configure_routes()
    }
    pub fn estimated_transfer(
        &mut self,
        source: &str,
        destination: &str,
        bytes: usize,
    ) -> Result<Transmission> {
        Ok(self.simulation.transfer(source, destination, bytes)?)
    }
}
