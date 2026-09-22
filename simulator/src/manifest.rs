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

//! Air's JSON manifest envelope plus explicit local topology extensions.
//! Unknown node image metadata is preserved. Executable topology fields are strict.
use crate::{api::Simulator, model::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format: String,
    pub name: String,
    #[serde(default)]
    pub ztp: Option<String>,
    pub content: Topology,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Topology {
    pub nodes: BTreeMap<String, ManifestNode>,
    #[serde(default)]
    pub links: Vec<ManifestLink>,
    #[serde(default)]
    pub oob: bool,
    #[serde(default)]
    pub enable_dhcp: bool,
    #[serde(default)]
    pub services: Vec<ManifestService>,
    #[serde(default)]
    pub instructions: Vec<ManifestInstruction>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestNode {
    pub os: String,
    #[serde(default = "cpu")]
    pub cpu: u32,
    #[serde(default = "memory")]
    pub memory: u64,
    #[serde(default = "storage")]
    pub storage: u64,
    #[serde(default)]
    pub role: Role,
    #[serde(default)]
    pub interfaces: BTreeMap<String, ManifestInterface>,
    #[serde(default)]
    pub labels: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub user_data: Option<String>,
    #[serde(default)]
    pub meta_data: Option<String>,
    #[serde(flatten)]
    pub image_metadata: BTreeMap<String, serde_json::Value>,
}
fn cpu() -> u32 {
    2
}
fn memory() -> u64 {
    1024
}
fn storage() -> u64 {
    10
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ManifestInterface {
    #[serde(default)]
    pub interface_type: InterfaceType,
    #[serde(default)]
    pub split_parent: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestLink {
    pub endpoints: [String; 2],
    #[serde(default)]
    pub condition: LinkSpec,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestService {
    pub name: String,
    pub interface: String,
    pub node_port: u16,
    pub service_type: ServiceType,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestInstruction {
    pub node: String,
    pub data: InstructionData,
    pub run_again_on_rebuild: bool,
}
impl Manifest {
    pub fn from_json(json: &str) -> Result<Self> {
        let mut value: serde_json::Value = serde_json::from_str(json)?;
        // SDK permits the JSON content to be a serialized JSON string.
        if let Some(content) = value.get("content").and_then(|c| c.as_str()) {
            let parsed = serde_json::from_str(content)?;
            value["content"] = parsed;
        }
        Ok(serde_json::from_value(value)?)
    }
    pub fn read(path: impl AsRef<std::path::Path>) -> Result<Self> {
        use std::io::Read;
        const LIMIT: u64 = 16 * 1024 * 1024;
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(LIMIT + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 > LIMIT {
            return Err(Error::Invalid("manifest exceeds 16 MiB".into()));
        }
        let text = std::str::from_utf8(&bytes).map_err(|e| Error::Invalid(e.to_string()))?;
        Self::from_json(text)
    }
}
fn endpoint<'a>(sim: &'a Simulation, value: &str) -> Result<&'a Interface> {
    let (n, i) = value
        .split_once(':')
        .ok_or_else(|| Error::Invalid(format!("endpoint needs node:interface: {value}")))?;
    sim.interface_named(&sim.node_named(n)?.id, i)
}
impl Simulator {
    /// Imports transactionally through the same public validators as individual edits.
    pub fn import(&mut self, manifest: Manifest, attempt_start: bool) -> Result<String> {
        self.check_capacity()?;
        if !manifest.format.eq_ignore_ascii_case("JSON") {
            return Err(Error::Unsupported(format!(
                "manifest format {}",
                manifest.format
            )));
        }
        let mut staging = Simulator::new();
        let key = staging.create(&manifest.name)?.id().to_string();
        let sim = staging.get_mut(&key)?;
        for (node_name, m) in &manifest.content.nodes {
            for field in m.image_metadata.keys() {
                if ![
                    "positioning",
                    "features",
                    "pxehost",
                    "secureboot",
                    "oob",
                    "network_pci",
                ]
                .contains(&field.as_str())
                {
                    return Err(Error::Invalid(format!(
                        "unknown node field {node_name}.{field}; put custom metadata in labels"
                    )));
                }
            }
            let id = sim.create_node(NodeSpec {
                name: node_name.clone(),
                image: m.os.clone(),
                role: m.role,
                resources: Resources {
                    cpu: m.cpu,
                    memory: m.memory,
                    storage: m.storage,
                },
                labels: m.labels.clone(),
            })?;
            let node = sim.nodes.get_mut(&id).unwrap();
            node.manifest_fields = m.image_metadata.clone();
            node.user_data = m.user_data.clone();
            node.meta_data = m.meta_data.clone();
            for (iface, spec) in &m.interfaces {
                sim.create_interface(&id, iface, spec.interface_type)?;
            }
            for (iface, spec) in &m.interfaces {
                if let Some(parent) = &spec.split_parent {
                    let child = sim.interface_named(&id, iface)?.id.clone();
                    let parent_id = sim.interface_named(&id, parent)?.id.clone();
                    if parent == iface
                        || m.interfaces
                            .get(parent)
                            .is_some_and(|p| p.split_parent.is_some())
                    {
                        return Err(Error::Invalid("cyclic or nested breakout".into()));
                    }
                    sim.interfaces.get_mut(&child).unwrap().split_parent = Some(parent_id.clone());
                    sim.interfaces
                        .get_mut(&parent_id)
                        .unwrap()
                        .split_children
                        .push(child);
                }
            }
        }
        for i in sim.interfaces.values() {
            if !i.split_children.is_empty()
                && (![2, 4, 8].contains(&i.split_children.len())
                    || i.interface_type != InterfaceType::Data)
            {
                return Err(Error::Invalid("invalid breakout lane count or type".into()));
            }
        }
        for link in &manifest.content.links {
            let a = endpoint(sim, &link.endpoints[0])?.id.clone();
            let b = endpoint(sim, &link.endpoints[1])?.id.clone();
            sim.create_link([&a, &b], link.condition)?;
        }
        for svc in &manifest.content.services {
            let i = endpoint(sim, &svc.interface)?.id.clone();
            sim.create_service(&i, &svc.name, svc.node_port, svc.service_type)?;
        }
        for instruction in &manifest.content.instructions {
            let node = sim.node_named(&instruction.node)?.id.clone();
            sim.create_instruction(
                &node,
                instruction.data.clone(),
                instruction.run_again_on_rebuild,
            )?;
        }
        sim.set_auto_oob(manifest.content.oob, manifest.content.enable_dhcp)?;
        if let Some(ztp) = manifest.ztp {
            sim.create_ztp_script(ztp)?;
        }
        if attempt_start {
            sim.start(None)?;
        }
        self.insert(sim.clone());
        Ok(key)
    }
}
impl Simulation {
    pub fn export(&self) -> Result<Manifest> {
        let endpoint = |id: &str| {
            let i = &self.interfaces[id];
            format!("{}:{}", self.nodes[&i.node].spec.name, i.name)
        };
        let nodes = self
            .nodes
            .values()
            .map(|n| {
                let interfaces = self
                    .interfaces
                    .values()
                    .filter(|i| i.node == n.id)
                    .map(|i| {
                        (
                            i.name.clone(),
                            ManifestInterface {
                                interface_type: i.interface_type,
                                split_parent: i
                                    .split_parent
                                    .as_ref()
                                    .map(|p| self.interfaces[p].name.clone()),
                            },
                        )
                    })
                    .collect();
                (
                    n.spec.name.clone(),
                    ManifestNode {
                        os: n.spec.image.clone(),
                        cpu: n.spec.resources.cpu,
                        memory: n.spec.resources.memory,
                        storage: n.spec.resources.storage,
                        role: n.spec.role,
                        interfaces,
                        labels: n.spec.labels.clone(),
                        user_data: n.user_data.clone(),
                        meta_data: n.meta_data.clone(),
                        image_metadata: n.manifest_fields.clone(),
                    },
                )
            })
            .collect();
        let mut links: Vec<_> = self
            .links
            .values()
            .map(|l| ManifestLink {
                endpoints: [endpoint(&l.interfaces[0]), endpoint(&l.interfaces[1])],
                condition: l.spec,
            })
            .collect();
        links.sort_by(|a, b| a.endpoints.cmp(&b.endpoints));
        let services = self
            .services
            .values()
            .map(|s| ManifestService {
                name: s.name.clone(),
                interface: endpoint(&s.interface),
                node_port: s.node_port,
                service_type: s.service_type,
            })
            .collect();
        let instructions = self
            .instructions
            .values()
            .map(|i| ManifestInstruction {
                node: self.nodes[&i.node].spec.name.clone(),
                data: i.data.clone(),
                run_again_on_rebuild: i.run_again_on_rebuild,
            })
            .collect();
        Ok(Manifest {
            format: "JSON".into(),
            name: self.name.clone(),
            ztp: self.ztp_script.clone(),
            content: Topology {
                nodes,
                links,
                oob: self.auto_oob_enabled,
                enable_dhcp: self.enable_dhcp,
                services,
                instructions,
            },
        })
    }
}
