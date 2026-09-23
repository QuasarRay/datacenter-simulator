// SPDX-License-Identifier: RPL-1.5
//! The same Air-style physical graph, executed by patchbay's native ibsim backend.
use crate::model::{InterfaceType, Role, Simulation, State};
use anyhow::{Context, Result, bail};
use patchbay::{
    Lab,
    infiniband::{IbEndpoint, IbFabric, IbNodeKind, IbOptions, IbTopology},
};
use serde::Serialize;
use std::{collections::BTreeMap, process::Output, time::Duration};

#[derive(Serialize)]
pub struct IbPlan {
    pub scope: &'static str,
    pub rdma_payload_tested: bool,
    pub nccl_tested: bool,
    pub bandwidth_latency_enforced: bool,
    pub nodes: BTreeMap<String, String>,
    pub ports: BTreeMap<String, u8>,
    pub native_topology: String,
}

pub fn topology(sim: &Simulation) -> Result<(IbTopology, IbPlan)> {
    if sim.interfaces().any(|i| {
        i.interface_type != InterfaceType::Data
            || !i.split_children.is_empty()
            || i.split_parent.is_some()
    }) {
        bail!("ibsim requires unsplit data interfaces; export the selected InfiniBand plane");
    }
    let mut nodes: Vec<_> = sim.nodes().collect();
    nodes.sort_by_key(|n| &n.spec.name);
    let mut topology = IbTopology::new();
    let mut plan = IbPlan {
        scope: "infiniband-management",
        rdma_payload_tested: false,
        nccl_tested: false,
        bandwidth_latency_enforced: false,
        nodes: BTreeMap::new(),
        ports: BTreeMap::new(),
        native_topology: String::new(),
    };
    let mut endpoints = BTreeMap::new();
    for (index, node) in nodes.iter().enumerate() {
        if node
            .spec
            .labels
            .get("simulator.medium")
            .and_then(|v| v.as_str())
            .is_some_and(|m| m != "infiniband")
        {
            bail!("cannot import another network medium into ibsim");
        }
        let mut ports: Vec<_> = sim.interfaces().filter(|i| i.node == node.id).collect();
        ports.sort_by_key(|i| &i.name);
        let count =
            u8::try_from(ports.len()).context("ibsim supports at most 255 ports per node")?;
        let name = format!("n{index:04}");
        topology.add_node(
            &name,
            if node.spec.role == Role::Switch {
                IbNodeKind::Switch
            } else {
                IbNodeKind::Hca
            },
            count,
        )?;
        plan.nodes.insert(node.spec.name.clone(), name.clone());
        for (p, port) in ports.iter().enumerate() {
            let number = (p + 1) as u8;
            endpoints.insert(port.id.clone(), IbEndpoint::new(&name, number));
            plan.ports
                .insert(format!("{}:{}", node.spec.name, port.name), number);
        }
    }
    for link in sim.links() {
        topology.add_link(
            &link.id,
            [
                endpoints[&link.interfaces[0]].clone(),
                endpoints[&link.interfaces[1]].clone(),
            ],
            link.spec.up,
        )?;
    }
    plan.native_topology = topology.render()?;
    Ok((topology, plan))
}

pub struct IbSimulation {
    backend: IbFabric,
    model: Simulation,
    plan: IbPlan,
    stopped: bool,
}
impl IbSimulation {
    pub async fn build(sim: &Simulation, options: IbOptions) -> Result<Self> {
        if sim.state() != State::Active {
            bail!("start the model before building ibsim");
        }
        let (topology, plan) = topology(sim)?;
        let lab = Lab::new().await?;
        let backend = lab.start_infiniband(topology, options).await?;
        Ok(Self {
            backend,
            model: sim.clone(),
            plan,
            stopped: false,
        })
    }
    pub fn model(&self) -> &Simulation {
        &self.model
    }
    pub fn plan(&self) -> &IbPlan {
        &self.plan
    }
    fn native_node(&self, node: &str) -> Result<String> {
        if self.stopped {
            bail!("InfiniBand backend has been shut down");
        }
        self.plan
            .nodes
            .get(node)
            .cloned()
            .context("unknown model node")
    }
    pub async fn start_subnet_manager(&mut self, node: &str, program: &str) -> Result<()> {
        let native = self.native_node(node)?;
        self.backend.start_subnet_manager(&native, program).await
    }
    pub fn port_status(&mut self, node: &str) -> Result<patchbay::infiniband::IbPortStatus> {
        let native = self.native_node(node)?;
        self.backend.port_status(&native)
    }
    pub async fn wait_active(
        &mut self,
        node: &str,
        deadline: Duration,
    ) -> Result<patchbay::infiniband::IbPortStatus> {
        let native = self.native_node(node)?;
        self.backend.wait_active(&native, deadline).await
    }
    pub async fn shutdown(&mut self) -> Result<()> {
        self.stopped = true;
        self.model.shutdown(false)?;
        self.backend.shutdown().await
    }
    pub async fn set_link_up(&mut self, id: &str, up: bool) -> Result<()> {
        if self.stopped {
            bail!("InfiniBand backend has been shut down");
        }
        let mut spec = self
            .model
            .links()
            .find(|l| l.id == id)
            .context("unknown physical link")?
            .spec;
        spec.up = up;
        let mut candidate = self.model.clone();
        candidate.set_link(id, spec)?;
        self.backend.set_link_up(id, up).await?;
        self.model = candidate;
        Ok(())
    }
    pub async fn run(
        &mut self,
        node: &str,
        command: tokio::process::Command,
        deadline: Duration,
    ) -> Result<Output> {
        let native = self.native_node(node)?;
        self.backend.run(&native, command, deadline).await
    }
}
