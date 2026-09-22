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

use crate::{id, model::*};
use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub struct Simulator {
    simulations: BTreeMap<String, Simulation>,
    max_simulations: usize,
}
impl Default for Simulator {
    fn default() -> Self {
        Self::with_capacity_limit(128)
    }
}
impl Simulator {
    pub fn with_capacity_limit(max_simulations: usize) -> Self {
        Self {
            simulations: BTreeMap::new(),
            max_simulations,
        }
    }
    pub(crate) fn check_capacity(&self) -> Result<()> {
        if self.simulations.len() >= self.max_simulations {
            return Err(Error::Conflict(
                "simulation session capacity reached".into(),
            ));
        }
        Ok(())
    }
    pub fn new() -> Self {
        Self::default()
    }
    pub fn create(&mut self, name: &str) -> Result<&Simulation> {
        self.check_capacity()?;
        if name.trim().is_empty() {
            return Err(Error::Invalid("simulation name is empty".into()));
        }
        let id = id();
        let now = Utc::now();
        let mut sim = Simulation {
            id: id.clone(),
            name: name.into(),
            state: State::Inactive,
            created: now,
            modified: now,
            metadata: None,
            sleep_at: None,
            expires_at: None,
            auto_oob_enabled: false,
            enable_dhcp: false,
            nodes: BTreeMap::new(),
            interfaces: BTreeMap::new(),
            links: BTreeMap::new(),
            services: BTreeMap::new(),
            instructions: BTreeMap::new(),
            checkpoints: BTreeMap::new(),
            runtime: BTreeMap::new(),
            ztp_script: None,
            history: Vec::new(),
            clock_ns: 0,
            scheduling_frontier_ns: 0,
            history_limit: 10_000,
            next_mac: 0,
            available: BTreeMap::new(),
        };
        sim.event("created");
        self.simulations.insert(id.clone(), sim);
        self.get(&id)
    }
    pub fn list(
        &self,
        name: Option<&str>,
        state: Option<State>,
        offset: usize,
        limit: usize,
    ) -> Vec<&Simulation> {
        self.simulations
            .values()
            .filter(|s| name.is_none_or(|v| s.name == v) && state.is_none_or(|v| s.state == v))
            .skip(offset)
            .take(limit)
            .collect()
    }
    pub fn get(&self, id: &str) -> Result<&Simulation> {
        self.simulations
            .get(id)
            .ok_or_else(|| Error::NotFound(id.into()))
    }
    pub fn get_mut(&mut self, id: &str) -> Result<&mut Simulation> {
        self.simulations
            .get_mut(id)
            .ok_or_else(|| Error::NotFound(id.into()))
    }
    pub fn delete(&mut self, id: &str) -> Result<()> {
        self.get(id)?.inactive()?;
        self.simulations.remove(id);
        Ok(())
    }
    pub fn clone_simulation(
        &mut self,
        source: &str,
        name: &str,
        checkpoint: Option<&str>,
        attempt_start: bool,
    ) -> Result<String> {
        let original = self.get(source)?;
        let mut manifest = original.export()?;
        manifest.name = name.into();
        // Reject stale configuration before allocating anything in the destination.
        let saved = checkpoint
            .map(|key| {
                let cp = original
                    .checkpoints
                    .get(key)
                    .ok_or_else(|| Error::NotFound(key.into()))?;
                original.validate_checkpoint(cp)?;
                let runtime = original
                    .nodes
                    .values()
                    .map(|n| {
                        (
                            n.spec.name.clone(),
                            cp.runtime[&n.id].clone(),
                            n.management_ip.clone(),
                        )
                    })
                    .collect::<Vec<_>>();
                let instructions = original
                    .instructions
                    .values()
                    .map(|i| {
                        (
                            original.nodes[&i.node].spec.name.clone(),
                            serde_json::to_value(&i.data).expect("instruction serialization"),
                            i.run_again_on_rebuild,
                            cp.instruction_states[&i.id].clone(),
                        )
                    })
                    .collect::<Vec<_>>();
                Ok::<_, Error>((runtime, instructions))
            })
            .transpose()?;
        let leases: Vec<_> = original
            .nodes
            .values()
            .map(|n| (n.spec.name.clone(), n.management_ip.clone()))
            .collect();
        let cloned = self.import(manifest, false)?;
        for (name, ip) in leases {
            let sim = self.get_mut(&cloned)?;
            let node = sim.node_named(&name)?.id.clone();
            sim.nodes.get_mut(&node).unwrap().management_ip = ip;
        }
        if let Some((saved, mut states)) = saved {
            let sim = self.get_mut(&cloned)?;
            for (name, runtime, management_ip) in saved {
                let id = sim.node_named(&name)?.id.clone();
                sim.runtime.insert(id.clone(), runtime);
                sim.nodes.get_mut(&id).unwrap().management_ip = management_ip;
            }
            for instruction in sim.instructions.values_mut() {
                let node_name = &sim.nodes[&instruction.node].spec.name;
                let data = serde_json::to_value(&instruction.data)?;
                let index = states
                    .iter()
                    .position(|(n, d, r, _)| {
                        n == node_name && *d == data && *r == instruction.run_again_on_rebuild
                    })
                    .expect("validated checkpoint instruction mapping");
                instruction.state = states.remove(index).3;
            }
        }
        if attempt_start && let Err(error) = self.get_mut(&cloned)?.start(None) {
            self.simulations.remove(&cloned);
            return Err(error);
        }
        Ok(cloned)
    }
    /// Apply explicit wall-clock deadlines; callers control the scheduling loop.
    pub fn tick(&mut self, now: DateTime<Utc>) -> Result<Vec<String>> {
        let expired: Vec<_> = self
            .simulations
            .values()
            .filter(|s| s.expires_at.is_some_and(|t| t <= now))
            .map(|s| s.id.clone())
            .collect();
        for id in &expired {
            self.simulations.remove(id);
        }
        for sim in self.simulations.values_mut() {
            if sim.state == State::Active && sim.sleep_at.is_some_and(|t| t <= now) {
                sim.shutdown(false)?;
            }
        }
        Ok(expired)
    }
    pub(crate) fn insert(&mut self, sim: Simulation) {
        self.simulations.insert(sim.id.clone(), sim);
    }
}

impl Simulation {
    pub fn update(
        &mut self,
        new_name: Option<&str>,
        metadata: Option<Option<String>>,
    ) -> Result<()> {
        if new_name.is_some_and(|n| n.trim().is_empty()) {
            return Err(Error::Invalid("simulation name is empty".into()));
        }
        if let Some(n) = new_name {
            self.name = n.into();
        }
        if let Some(m) = metadata {
            self.metadata = m;
        }
        self.event("updated");
        Ok(())
    }
    pub fn set_schedule(
        &mut self,
        sleep_at: Option<DateTime<Utc>>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<()> {
        if let (Some(s), Some(e)) = (sleep_at, expires_at)
            && s > e
        {
            return Err(Error::Invalid("sleep_at is later than expires_at".into()));
        }
        self.sleep_at = sleep_at;
        self.expires_at = expires_at;
        self.event("schedule updated");
        Ok(())
    }
    pub fn create_node(&mut self, spec: NodeSpec) -> Result<String> {
        self.inactive()?;
        name(&spec.name)?;
        spec.resources.validate()?;
        if spec.image.trim().is_empty() {
            return Err(Error::Invalid("image is empty".into()));
        }
        if self.nodes.len() >= 4096 {
            return Err(Error::Invalid("4096 node limit".into()));
        }
        if self.nodes.values().any(|n| n.spec.name == spec.name) {
            return Err(Error::Conflict(spec.name));
        }
        let key = id();
        self.runtime.insert(
            key.clone(),
            NodeRuntime {
                hostname: spec.name.clone(),
                files: BTreeMap::new(),
            },
        );
        self.nodes.insert(
            key.clone(),
            Node {
                id: key.clone(),
                simulation: self.id.clone(),
                spec,
                state: State::Inactive,
                generation: 0,
                user_data: None,
                meta_data: None,
                management_ip: None,
                manifest_fields: BTreeMap::new(),
            },
        );
        self.assign_management_addresses();
        self.event("node created");
        Ok(key)
    }
    pub fn update_node(&mut self, node: &str, spec: NodeSpec) -> Result<()> {
        self.inactive()?;
        self.node(node)?;
        name(&spec.name)?;
        spec.resources.validate()?;
        if spec.image.trim().is_empty() {
            return Err(Error::Invalid("image is empty".into()));
        }
        if self
            .nodes
            .values()
            .any(|n| n.id != node && n.spec.name == spec.name)
        {
            return Err(Error::Conflict(spec.name));
        }
        self.nodes.get_mut(node).unwrap().spec = spec;
        self.event("node updated");
        Ok(())
    }
    pub fn delete_node(&mut self, node: &str) -> Result<()> {
        self.inactive()?;
        self.node(node)?;
        let interfaces: Vec<_> = self
            .interfaces
            .values()
            .filter(|i| i.node == node)
            .map(|i| i.id.clone())
            .collect();
        for i in interfaces {
            self.remove_interface(&i);
        }
        self.instructions.retain(|_, i| i.node != node);
        self.runtime.remove(node);
        self.nodes.remove(node);
        // Saved runtime of a different topology cannot be restored.
        self.checkpoints.clear();
        self.event("node deleted");
        Ok(())
    }
    pub fn create_interface(
        &mut self,
        node: &str,
        interface_name: &str,
        interface_type: InterfaceType,
    ) -> Result<String> {
        self.inactive()?;
        self.node(node)?;
        name(interface_name)?;
        if interface_name == "lo" || interface_name.len() > 15 {
            return Err(Error::Invalid(
                "Linux interface names must be <= 15 bytes".into(),
            ));
        }
        if self
            .interfaces
            .values()
            .any(|i| i.node == node && i.name == interface_name)
        {
            return Err(Error::Conflict(interface_name.into()));
        }
        let key = id();
        // A monotonic, locally administered 40-bit identity is unique within this fabric.
        if self.next_mac >= (1u64 << 40) {
            return Err(Error::Conflict("MAC address space exhausted".into()));
        }
        let bytes = self.next_mac.to_be_bytes();
        self.next_mac += 1;
        let mac = format!(
            "02:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]
        );
        self.interfaces.insert(
            key.clone(),
            Interface {
                id: key.clone(),
                node: node.into(),
                name: interface_name.into(),
                interface_type,
                mac_address: mac,
                connection: None,
                split_parent: None,
                split_children: Vec::new(),
            },
        );
        self.event("interface created");
        Ok(key)
    }
    fn remove_interface(&mut self, interface: &str) {
        let links: Vec<_> = self
            .links
            .values()
            .filter(|l| l.interfaces.iter().any(|i| i == interface))
            .map(|l| l.id.clone())
            .collect();
        for l in links {
            self.remove_link(&l);
        }
        self.services.retain(|_, s| s.interface != interface);
        self.interfaces.remove(interface);
    }
    pub fn delete_interface(&mut self, interface: &str) -> Result<()> {
        self.inactive()?;
        let i = self.interface(interface)?;
        if i.split_parent.is_some() || !i.split_children.is_empty() {
            return Err(Error::Conflict(
                "revert breakout before deleting interface".into(),
            ));
        }
        self.remove_interface(interface);
        self.event("interface deleted");
        Ok(())
    }
    pub fn breakout(&mut self, interface: &str, split_count: usize) -> Result<Vec<String>> {
        self.inactive()?;
        let parent = self.interface(interface)?.clone();
        if ![2, 4, 8].contains(&split_count) || parent.interface_type != InterfaceType::Data {
            return Err(Error::Invalid(
                "data interfaces support 2, 4 or 8 lanes".into(),
            ));
        }
        if parent.connection.is_some()
            || parent.split_parent.is_some()
            || !parent.split_children.is_empty()
            || self.services.values().any(|s| s.interface == interface)
        {
            return Err(Error::Conflict(
                "breakout requires an unused unsplit interface".into(),
            ));
        }
        let names: Vec<_> = (0..split_count)
            .map(|i| format!("{}s{i}", parent.name))
            .collect();
        for n in &names {
            if n.len() > 15
                || self
                    .interfaces
                    .values()
                    .any(|i| i.node == parent.node && i.name == *n)
            {
                return Err(Error::Conflict(n.clone()));
            }
        }
        let mut children = Vec::new();
        for n in names {
            let child = self.create_interface(&parent.node, &n, parent.interface_type)?;
            self.interfaces.get_mut(&child).unwrap().split_parent = Some(interface.into());
            children.push(child);
        }
        self.interfaces.get_mut(interface).unwrap().split_children = children.clone();
        self.event("interface breakout");
        Ok(children)
    }
    pub fn revert_breakout(&mut self, interface: &str) -> Result<()> {
        self.inactive()?;
        let parent = self.interface(interface)?.clone();
        if parent.split_children.is_empty() {
            return Err(Error::Conflict("interface has no breakout".into()));
        }
        for child in &parent.split_children {
            if self.interface(child)?.connection.is_some()
                || self.services.values().any(|s| s.interface == *child)
            {
                return Err(Error::Conflict("breakout child is in use".into()));
            }
        }
        for child in parent.split_children {
            self.remove_interface(&child);
        }
        self.interfaces
            .get_mut(interface)
            .unwrap()
            .split_children
            .clear();
        self.event("interface breakout reverted");
        Ok(())
    }
    pub fn create_link(&mut self, endpoints: [&str; 2], spec: LinkSpec) -> Result<String> {
        self.inactive()?;
        spec.validate()?;
        let a = self.interface(endpoints[0])?;
        let b = self.interface(endpoints[1])?;
        if a.id == b.id || a.node == b.node {
            return Err(Error::Invalid("link requires two different nodes".into()));
        }
        if a.interface_type != b.interface_type {
            return Err(Error::Invalid("incompatible interface types".into()));
        }
        if a.connection.is_some()
            || b.connection.is_some()
            || !a.split_children.is_empty()
            || !b.split_children.is_empty()
        {
            return Err(Error::Conflict(
                "interface already connected or broken out".into(),
            ));
        }
        let key = id();
        self.links.insert(
            key.clone(),
            Link {
                id: key.clone(),
                interfaces: [endpoints[0].into(), endpoints[1].into()],
                spec,
            },
        );
        self.interfaces.get_mut(endpoints[0]).unwrap().connection = Some(endpoints[1].into());
        self.interfaces.get_mut(endpoints[1]).unwrap().connection = Some(endpoints[0].into());
        self.event("link created");
        Ok(key)
    }
    fn remove_link(&mut self, link: &str) {
        if let Some(l) = self.links.remove(link) {
            for i in l.interfaces {
                if let Some(i) = self.interfaces.get_mut(&i) {
                    i.connection = None;
                }
            }
            self.available.retain(|(l, _), _| l != link);
        }
    }
    pub fn delete_link(&mut self, link: &str) -> Result<()> {
        self.inactive()?;
        if !self.links.contains_key(link) {
            return Err(Error::NotFound(link.into()));
        }
        self.remove_link(link);
        self.event("link deleted");
        Ok(())
    }
    pub fn set_link(&mut self, link: &str, spec: LinkSpec) -> Result<()> {
        spec.validate()?;
        self.links
            .get_mut(link)
            .ok_or_else(|| Error::NotFound(link.into()))?
            .spec = spec;
        self.event("link conditions updated");
        Ok(())
    }
    pub fn set_auto_oob(&mut self, enabled: bool, dhcp: bool) -> Result<()> {
        self.inactive()?;
        self.auto_oob_enabled = enabled;
        self.enable_dhcp = enabled && dhcp;
        self.assign_management_addresses();
        self.event("OOB updated");
        Ok(())
    }
    fn assign_management_addresses(&mut self) {
        // Preserve leases when adding nodes. Allocate new leases in name order.
        if !(self.auto_oob_enabled && self.enable_dhcp) {
            for node in self.nodes.values_mut() {
                node.management_ip = None;
            }
            return;
        }
        let mut used: BTreeSet<String> = self
            .nodes
            .values()
            .filter_map(|n| n.management_ip.clone())
            .collect();
        let mut nodes: Vec<_> = self.nodes.values_mut().collect();
        nodes.sort_by(|a, b| a.spec.name.cmp(&b.spec.name));
        let mut host = 2;
        for node in nodes {
            if node.management_ip.is_some() {
                continue;
            }
            loop {
                let ip = format!("172.31.{}.{}/16", host / 256, host % 256);
                host += 1;
                if used.insert(ip.clone()) {
                    node.management_ip = Some(ip);
                    break;
                }
            }
        }
    }
    pub fn create_service(
        &mut self,
        interface: &str,
        name: &str,
        node_port: u16,
        service_type: ServiceType,
    ) -> Result<String> {
        let i = self.interface(interface)?;
        if !i.split_children.is_empty() || node_port == 0 {
            return Err(Error::Invalid(
                "service needs usable interface and nonzero port".into(),
            ));
        }
        if self
            .services
            .values()
            .any(|s| s.interface == interface && s.node_port == node_port)
        {
            return Err(Error::Conflict("service port already registered".into()));
        }
        let key = id();
        self.services.insert(
            key.clone(),
            Service {
                id: key.clone(),
                name: name.into(),
                interface: interface.into(),
                node_port,
                service_type,
                worker_port: None,
                worker_fqdn: None,
            },
        );
        self.event("service registered");
        Ok(key)
    }
    pub fn delete_service(&mut self, service: &str) -> Result<()> {
        self.services
            .remove(service)
            .ok_or_else(|| Error::NotFound(service.into()))?;
        self.event("service deleted");
        Ok(())
    }
    pub fn ztp_script(&self) -> Option<&str> {
        self.ztp_script.as_deref()
    }
    pub fn create_ztp_script(&mut self, content: String) -> Result<()> {
        self.inactive()?;
        if self.ztp_script.is_some() {
            return Err(Error::Conflict("ZTP script exists".into()));
        }
        self.ztp_script = Some(content);
        self.event("ZTP script created");
        Ok(())
    }
    pub fn update_ztp_script(&mut self, content: String) -> Result<()> {
        self.inactive()?;
        if self.ztp_script.is_none() {
            return Err(Error::NotFound("ZTP script".into()));
        }
        self.ztp_script = Some(content);
        self.event("ZTP script updated");
        Ok(())
    }
    pub fn delete_ztp_script(&mut self) -> Result<()> {
        self.inactive()?;
        self.ztp_script
            .take()
            .ok_or_else(|| Error::NotFound("ZTP script".into()))?;
        self.event("ZTP script deleted");
        Ok(())
    }
    pub fn create_instruction(
        &mut self,
        node: &str,
        data: InstructionData,
        run_again_on_rebuild: bool,
    ) -> Result<String> {
        self.inactive()?;
        self.node(node)?;
        match &data {
            InstructionData::Init{hostname}=>name(hostname)?,
            InstructionData::File{files}=>{
                for path in files.keys() {
                    if !path.starts_with('/') || path.split('/').any(|c|c=="..") || path.contains('\0') { return Err(Error::Invalid(format!("invalid guest path {path}"))); }
                }
            }
            InstructionData::Shell{..}=>return Err(Error::Unsupported("guest shell execution requires a VM/container backend; network namespaces share the host filesystem".into())),
        }
        let key = id();
        self.instructions.insert(
            key.clone(),
            Instruction {
                id: key.clone(),
                node: node.into(),
                data,
                run_again_on_rebuild,
                state: "PENDING".into(),
            },
        );
        self.event("node instruction created");
        Ok(key)
    }
    pub fn delete_instruction(&mut self, instruction: &str) -> Result<()> {
        self.inactive()?;
        self.instructions
            .remove(instruction)
            .ok_or_else(|| Error::NotFound(instruction.into()))?;
        self.event("node instruction deleted");
        Ok(())
    }
    pub fn assign_configs(
        &mut self,
        assignments: &[(String, Option<String>, Option<String>)],
    ) -> Result<()> {
        self.inactive()?;
        let mut unique = BTreeSet::new();
        for (node, _, _) in assignments {
            self.node(node)?;
            if !unique.insert(node) {
                return Err(Error::Invalid("duplicate node assignment".into()));
            }
        }
        for (node, user, meta) in assignments {
            let n = self.nodes.get_mut(node).unwrap();
            n.user_data = user.clone();
            n.meta_data = meta.clone();
        }
        self.event("node configs assigned");
        Ok(())
    }
    fn apply_instructions(&mut self, selected: Option<&BTreeSet<String>>, rebuild: bool) {
        for instruction in self.instructions.values_mut() {
            if selected.is_some_and(|s| !s.contains(&instruction.node)) {
                continue;
            }
            if instruction.state == "COMPLETE" && !(rebuild && instruction.run_again_on_rebuild) {
                continue;
            }
            let runtime = self.runtime.entry(instruction.node.clone()).or_default();
            match &instruction.data {
                InstructionData::Init { hostname } => runtime.hostname = hostname.clone(),
                InstructionData::File { files } => runtime.files.extend(files.clone()),
                InstructionData::Shell { .. } => continue,
            }
            instruction.state = "COMPLETE".into();
        }
    }
    pub fn start(&mut self, checkpoint: Option<&str>) -> Result<()> {
        self.inactive()?;
        if self.nodes.is_empty() {
            return Err(Error::Invalid("simulation has no nodes".into()));
        }
        if self
            .ztp_script
            .as_ref()
            .is_some_and(|s| !s.trim().is_empty())
        {
            return Err(Error::Unsupported(
                "executing guest ZTP requires a VM/container backend".into(),
            ));
        }
        if self
            .nodes
            .values()
            .any(|n| n.user_data.is_some() || n.meta_data.is_some())
        {
            return Err(Error::Unsupported(
                "executing cloud-init requires a VM/container backend".into(),
            ));
        }
        if let Some(cp) = checkpoint {
            self.restore_checkpoint(cp)?;
        } else {
            self.apply_instructions(None, false);
        }
        self.state = State::Active;
        for node in self.nodes.values_mut() {
            node.state = State::Active;
        }
        self.event("started");
        Ok(())
    }
    pub fn shutdown(&mut self, create_checkpoint: bool) -> Result<Option<String>> {
        self.active()?;
        let checkpoint = if create_checkpoint {
            Some(self.create_checkpoint("shutdown")?)
        } else {
            None
        };
        self.state = State::Inactive;
        for n in self.nodes.values_mut() {
            n.state = State::Inactive;
            n.generation += 1;
        }
        self.available.clear();
        self.scheduling_frontier_ns = self.clock_ns;
        self.event("shutdown");
        Ok(checkpoint)
    }
    pub fn rebuild(&mut self, checkpoint: Option<&str>) -> Result<()> {
        self.inactive()?;
        if let Some(cp) = checkpoint {
            self.restore_checkpoint(cp)?;
        } else {
            self.runtime = self
                .nodes
                .values()
                .map(|n| {
                    (
                        n.id.clone(),
                        NodeRuntime {
                            hostname: n.spec.name.clone(),
                            files: BTreeMap::new(),
                        },
                    )
                })
                .collect();
            self.apply_instructions(None, true);
        }
        for n in self.nodes.values_mut() {
            n.generation += 1;
        }
        self.available.clear();
        self.scheduling_frontier_ns = self.clock_ns;
        self.event("rebuilt");
        Ok(())
    }
    pub fn reset_nodes(&mut self, nodes: &[String], rebuild: bool) -> Result<()> {
        self.active()?;
        let unique: BTreeSet<_> = nodes.iter().cloned().collect();
        if unique.len() != nodes.len() {
            return Err(Error::Invalid("duplicate nodes in bulk operation".into()));
        }
        for node in nodes {
            self.node(node)?;
        }
        for node in nodes {
            let n = self.nodes.get_mut(node).unwrap();
            n.generation += 1;
            if rebuild {
                self.runtime.insert(
                    node.clone(),
                    NodeRuntime {
                        hostname: n.spec.name.clone(),
                        files: BTreeMap::new(),
                    },
                );
            }
        }
        if rebuild {
            self.apply_instructions(Some(&unique), true);
        }
        // Reset cancels all scheduled traffic, including transit reservations.
        if !nodes.is_empty() {
            self.available.clear();
            self.scheduling_frontier_ns = self.clock_ns;
        }
        self.event(if rebuild {
            "nodes rebuilt"
        } else {
            "nodes reset"
        });
        Ok(())
    }
    pub fn create_checkpoint(&mut self, name: &str) -> Result<String> {
        let key = id();
        self.checkpoints.insert(
            key.clone(),
            Checkpoint {
                id: key.clone(),
                name: name.into(),
                favorite: false,
                state: "COMPLETE".into(),
                created: Utc::now(),
                runtime: self.runtime.clone(),
                configuration: self.checkpoint_configuration()?,
                instruction_states: self
                    .instructions
                    .values()
                    .map(|i| (i.id.clone(), i.state.clone()))
                    .collect(),
            },
        );
        self.event("checkpoint created");
        Ok(key)
    }
    pub fn update_checkpoint(
        &mut self,
        checkpoint: &str,
        name: String,
        favorite: bool,
    ) -> Result<()> {
        let cp = self
            .checkpoints
            .get_mut(checkpoint)
            .ok_or_else(|| Error::NotFound(checkpoint.into()))?;
        cp.name = name;
        cp.favorite = favorite;
        self.event("checkpoint updated");
        Ok(())
    }
    pub fn delete_checkpoint(&mut self, checkpoint: &str) -> Result<()> {
        self.checkpoints
            .remove(checkpoint)
            .ok_or_else(|| Error::NotFound(checkpoint.into()))?;
        self.event("checkpoint deleted");
        Ok(())
    }
    fn checkpoint_configuration(&self) -> Result<serde_json::Value> {
        // Store the exact configuration, not a lossy hash or just node IDs.
        let mut nodes = serde_json::to_value(&self.nodes)?;
        for node in nodes.as_object_mut().unwrap().values_mut() {
            let node = node.as_object_mut().unwrap();
            node.remove("state");
            node.remove("generation");
        }
        let mut instructions = serde_json::to_value(&self.instructions)?;
        for i in instructions.as_object_mut().unwrap().values_mut() {
            i.as_object_mut().unwrap().remove("state");
        }
        Ok(
            serde_json::json!({"nodes": nodes, "interfaces": self.interfaces,
            "links": self.links, "services": self.services, "instructions": instructions,
            "ztp": self.ztp_script, "oob": self.auto_oob_enabled, "dhcp": self.enable_dhcp}),
        )
    }
    fn validate_checkpoint(&self, cp: &Checkpoint) -> Result<()> {
        if !cp.runtime.keys().eq(self.nodes.keys())
            || !cp.instruction_states.keys().eq(self.instructions.keys())
            || cp.configuration != self.checkpoint_configuration()?
        {
            return Err(Error::Conflict(
                "checkpoint configuration differs; create a new checkpoint after edits".into(),
            ));
        }
        Ok(())
    }
    fn restore_checkpoint(&mut self, checkpoint: &str) -> Result<()> {
        let cp = self
            .checkpoints
            .get(checkpoint)
            .ok_or_else(|| Error::NotFound(checkpoint.into()))?;
        self.validate_checkpoint(cp)?;
        self.runtime = cp.runtime.clone();
        for (id, state) in &cp.instruction_states {
            self.instructions.get_mut(id).unwrap().state = state.clone();
        }
        self.available.clear();
        self.scheduling_frontier_ns = self.clock_ns;
        Ok(())
    }
    pub fn wait_for_state(&self, target: State) -> Result<()> {
        // Synchronous local transitions are complete when the mutating method returns.
        if self.state != target {
            return Err(Error::Timeout(format!(
                "state is {:?}, requested {target:?}",
                self.state
            )));
        }
        Ok(())
    }
    pub fn required_resources(&self) -> Result<Resources> {
        let mut r = Resources {
            cpu: 0,
            memory: 0,
            storage: 0,
        };
        for n in self.nodes.values() {
            r.cpu = r
                .cpu
                .checked_add(n.spec.resources.cpu)
                .ok_or_else(|| Error::Invalid("CPU total overflow".into()))?;
            r.memory = r
                .memory
                .checked_add(n.spec.resources.memory)
                .ok_or_else(|| Error::Invalid("memory total overflow".into()))?;
            r.storage = r
                .storage
                .checked_add(n.spec.resources.storage)
                .ok_or_else(|| Error::Invalid("storage total overflow".into()))?;
        }
        Ok(r)
    }
}
