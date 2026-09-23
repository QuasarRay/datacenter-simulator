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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid input: {0}")]
    Invalid(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("invalid state: {0}")]
    State(String),
    #[error("no route from {0} to {1}")]
    NoRoute(String, String),
    #[error("unsupported capability: {0}")]
    Unsupported(String),
    #[error("timeout: {0}")]
    Timeout(String),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum State {
    Inactive,
    Active,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    #[default]
    Host,
    Switch,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum InterfaceType {
    #[default]
    #[serde(rename = "DATA_PLANE_INTF")]
    Data,
    #[serde(rename = "OOB_INTF")]
    Oob,
    #[serde(rename = "PCIE_INTF")]
    Pcie,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Resources {
    pub cpu: u32,
    pub memory: u64,
    pub storage: u64,
}
impl Default for Resources {
    fn default() -> Self {
        Self {
            cpu: 2,
            memory: 1024,
            storage: 10,
        }
    }
}
impl Resources {
    pub fn validate(&self) -> Result<()> {
        if self.cpu == 0 || self.memory == 0 || self.storage == 0 {
            return Err(Error::Invalid(
                "cpu, memory MiB, and storage GB must be positive".into(),
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeSpec {
    pub name: String,
    #[serde(default = "default_image")]
    pub image: String,
    #[serde(default)]
    pub role: Role,
    #[serde(default)]
    pub resources: Resources,
    #[serde(default)]
    pub labels: BTreeMap<String, serde_json::Value>,
}
fn default_image() -> String {
    "generic/ubuntu2204".into()
}
impl NodeSpec {
    pub fn host(name: &str) -> Self {
        Self {
            name: name.into(),
            image: default_image(),
            role: Role::Host,
            resources: Resources::default(),
            labels: BTreeMap::new(),
        }
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct Node {
    pub id: String,
    pub simulation: String,
    #[serde(flatten)]
    pub spec: NodeSpec,
    pub state: State,
    pub generation: u64,
    pub user_data: Option<String>,
    pub meta_data: Option<String>,
    pub management_ip: Option<String>,
    pub(crate) manifest_fields: BTreeMap<String, serde_json::Value>,
}
#[derive(Debug, Clone, Serialize)]
pub struct Interface {
    pub id: String,
    pub node: String,
    pub name: String,
    pub interface_type: InterfaceType,
    pub mac_address: String,
    pub connection: Option<String>,
    pub split_parent: Option<String>,
    pub split_children: Vec<String>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LinkSpec {
    pub bandwidth_bps: u64,
    pub latency_ns: u64,
    #[serde(default = "yes")]
    pub up: bool,
}
fn yes() -> bool {
    true
}
impl Default for LinkSpec {
    fn default() -> Self {
        Self {
            bandwidth_bps: 100_000_000_000,
            latency_ns: 1000,
            up: true,
        }
    }
}
impl LinkSpec {
    /// Native netem queue in packets, at 1500-byte MTU: two BDPs plus 1024 packets.
    /// Analytical transfers have unbounded lossless queues and do not model packet drops.
    pub fn native_queue_packets(&self) -> Result<u32> {
        let bdp = (self.bandwidth_bps as u128 * self.latency_ns as u128)
            .div_ceil(8 * 1_000_000_000 * 1500);
        u32::try_from(2 * bdp + 1024)
            .map_err(|_| Error::Invalid("native queue capacity exceeds u32 packets".into()))
    }
    pub fn validate(&self) -> Result<()> {
        if self.bandwidth_bps == 0 || self.latency_ns > 1_000_000_000_000 {
            return Err(Error::Invalid(
                "link needs positive bandwidth and latency <= 1000 seconds".into(),
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct Link {
    pub id: String,
    pub interfaces: [String; 2],
    #[serde(flatten)]
    pub spec: LinkSpec,
}
#[derive(Debug, Clone, Serialize)]
pub struct HistoryEntry {
    pub object_id: String,
    pub model: String,
    pub created: DateTime<Utc>,
    pub actor: String,
    pub description: String,
    pub severity: String,
    pub labels: Vec<String>,
}
#[derive(Debug, Clone, Serialize)]
pub struct HistoryFilters {
    pub actors: BTreeSet<String>,
    pub severities: BTreeSet<String>,
    pub labels: BTreeSet<String>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ServiceType {
    SSH,
    HTTPS,
    HTTP,
    OTHER,
}
#[derive(Debug, Clone, Serialize)]
pub struct Service {
    pub id: String,
    pub name: String,
    pub interface: String,
    pub node_port: u16,
    pub service_type: ServiceType,
    // A local service descriptor is not a published NVIDIA Air tunnel.
    pub worker_port: Option<u16>,
    pub worker_fqdn: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "executor", rename_all = "lowercase", deny_unknown_fields)]
pub enum InstructionData {
    Init { hostname: String },
    File { files: BTreeMap<String, String> },
    Shell { commands: Vec<String> },
}
#[derive(Debug, Clone, Serialize)]
pub struct Instruction {
    pub id: String,
    pub node: String,
    pub data: InstructionData,
    pub run_again_on_rebuild: bool,
    pub state: String,
}
#[derive(Debug, Clone, Default, Serialize)]
pub struct NodeRuntime {
    pub hostname: String,
    pub files: BTreeMap<String, String>,
}
#[derive(Debug, Clone, Serialize)]
pub struct Checkpoint {
    pub id: String,
    pub name: String,
    pub favorite: bool,
    pub state: String,
    pub created: DateTime<Utc>,
    #[serde(skip)]
    pub(crate) runtime: BTreeMap<String, NodeRuntime>,
    #[serde(skip)]
    pub(crate) configuration: serde_json::Value,
    #[serde(skip)]
    pub(crate) instruction_states: BTreeMap<String, String>,
    #[serde(skip)]
    pub(crate) retained_bytes: usize,
}
#[derive(Debug, Clone, Serialize)]
pub struct Simulation {
    pub(crate) limits: crate::limits::Limits,
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) state: State,
    pub(crate) created: DateTime<Utc>,
    pub(crate) modified: DateTime<Utc>,
    pub(crate) metadata: Option<String>,
    pub(crate) sleep_at: Option<DateTime<Utc>>,
    pub(crate) expires_at: Option<DateTime<Utc>>,
    pub(crate) auto_oob_enabled: bool,
    pub(crate) enable_dhcp: bool,
    pub(crate) nodes: BTreeMap<String, Node>,
    pub(crate) interfaces: BTreeMap<String, Interface>,
    pub(crate) links: BTreeMap<String, Link>,
    pub(crate) services: BTreeMap<String, Service>,
    pub(crate) instructions: BTreeMap<String, Instruction>,
    pub(crate) checkpoints: BTreeMap<String, Checkpoint>,
    pub(crate) runtime: BTreeMap<String, NodeRuntime>,
    pub(crate) ztp_script: Option<String>,
    pub(crate) history: Vec<HistoryEntry>,
    pub(crate) clock_ns: u64,
    pub(crate) scheduling_frontier_ns: u64,
    pub(crate) history_limit: usize,
    #[serde(skip)]
    pub(crate) next_mac: u64,
    #[serde(skip)]
    pub(crate) available: BTreeMap<(String, String), u64>,
}
pub(crate) fn name(value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 63
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(Error::Invalid(format!(
            "name must contain 1..63 ASCII letters, digits, hyphens or underscores: {value}"
        )));
    }
    Ok(())
}
impl Simulation {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn state(&self) -> State {
        self.state
    }
    pub fn clock_ns(&self) -> u64 {
        self.clock_ns
    }
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }
    pub fn interfaces(&self) -> impl Iterator<Item = &Interface> {
        self.interfaces.values()
    }
    pub fn links(&self) -> impl Iterator<Item = &Link> {
        self.links.values()
    }
    pub fn services(&self) -> impl Iterator<Item = &Service> {
        self.services.values()
    }
    pub fn instructions(&self) -> impl Iterator<Item = &Instruction> {
        self.instructions.values()
    }
    pub fn checkpoints(&self) -> impl Iterator<Item = &Checkpoint> {
        self.checkpoints.values()
    }
    /// Retain the newest `limit` events; zero disables retention.
    pub fn set_history_limit(&mut self, limit: usize) {
        self.history_limit = limit;
        let excess = self.history.len().saturating_sub(limit);
        self.history.drain(..excess);
    }
    pub fn history(&self) -> &[HistoryEntry] {
        &self.history
    }
    pub fn history_filters(&self) -> HistoryFilters {
        HistoryFilters {
            actors: self.history.iter().map(|e| e.actor.clone()).collect(),
            severities: self.history.iter().map(|e| e.severity.clone()).collect(),
            labels: self.history.iter().flat_map(|e| e.labels.clone()).collect(),
        }
    }
    pub fn node(&self, id: &str) -> Result<&Node> {
        self.nodes.get(id).ok_or_else(|| Error::NotFound(id.into()))
    }
    pub fn node_named(&self, name: &str) -> Result<&Node> {
        self.nodes
            .values()
            .find(|n| n.spec.name == name)
            .ok_or_else(|| Error::NotFound(name.into()))
    }
    pub fn interface(&self, id: &str) -> Result<&Interface> {
        self.interfaces
            .get(id)
            .ok_or_else(|| Error::NotFound(id.into()))
    }
    pub fn interface_named(&self, node: &str, name: &str) -> Result<&Interface> {
        self.node(node)?;
        self.interfaces
            .values()
            .find(|i| i.node == node && i.name == name)
            .ok_or_else(|| Error::NotFound(format!("{node}:{name}")))
    }
    pub fn runtime(&self, node: &str) -> Result<&NodeRuntime> {
        self.node(node)?;
        self.runtime
            .get(node)
            .ok_or_else(|| Error::State("node has no runtime".into()))
    }
    pub(crate) fn inactive(&self) -> Result<()> {
        if self.state != State::Inactive {
            return Err(Error::State("topology edits require INACTIVE".into()));
        }
        Ok(())
    }
    pub(crate) fn active(&self) -> Result<()> {
        if self.state != State::Active {
            return Err(Error::State("operation requires ACTIVE".into()));
        }
        Ok(())
    }
    pub(crate) fn event(&mut self, description: &str) {
        self.modified = Utc::now();
        if self.history_limit == 0 {
            return;
        }
        if self.history.len() >= self.history_limit {
            self.history
                .drain(..=self.history.len() - self.history_limit);
        }
        self.history.push(HistoryEntry {
            object_id: self.id.clone(),
            model: "simulation".into(),
            created: self.modified,
            actor: "local".into(),
            description: description.into(),
            severity: "INFO".into(),
            labels: vec!["simulator".into()],
        });
    }
}
