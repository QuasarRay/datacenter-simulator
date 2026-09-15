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

//! A JSON-lines Rust command interface; the Python SDK is not loaded or invoked.
use crate::{
    Simulator,
    collective::{Collective, NcclOptions, Reduction},
    manifest::Manifest,
    model::*,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
#[derive(Debug, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    Create {
        name: String,
    },
    List {
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        state: Option<State>,
        #[serde(default)]
        offset: usize,
        #[serde(default = "limit")]
        limit: usize,
    },
    Get {
        simulation: String,
    },
    Delete {
        simulation: String,
    },
    Update {
        simulation: String,
        name: String,
    },
    Import {
        manifest: Manifest,
        #[serde(default)]
        attempt_start: bool,
    },
    Export {
        simulation: String,
    },
    Clone {
        simulation: String,
        name: String,
        #[serde(default)]
        checkpoint: Option<String>,
        #[serde(default)]
        attempt_start: bool,
    },
    Start {
        simulation: String,
        #[serde(default)]
        checkpoint: Option<String>,
    },
    Shutdown {
        simulation: String,
        #[serde(default)]
        create_checkpoint: bool,
    },
    Rebuild {
        simulation: String,
        #[serde(default)]
        checkpoint: Option<String>,
    },
    CreateNode {
        simulation: String,
        node: NodeSpec,
    },
    UpdateNode {
        simulation: String,
        node_id: String,
        node: NodeSpec,
    },
    DeleteNode {
        simulation: String,
        node: String,
    },
    CreateInterface {
        simulation: String,
        node: String,
        name: String,
        #[serde(default)]
        interface_type: InterfaceType,
    },
    DeleteInterface {
        simulation: String,
        interface: String,
    },
    Breakout {
        simulation: String,
        interface: String,
        split_count: usize,
    },
    RevertBreakout {
        simulation: String,
        interface: String,
    },
    CreateLink {
        simulation: String,
        interfaces: [String; 2],
        #[serde(default)]
        condition: LinkSpec,
    },
    DeleteLink {
        simulation: String,
        link: String,
    },
    SetLink {
        simulation: String,
        link: String,
        condition: LinkSpec,
    },
    AutoOob {
        simulation: String,
        enabled: bool,
        dhcp: bool,
    },
    CreateService {
        simulation: String,
        interface: String,
        name: String,
        node_port: u16,
        service_type: ServiceType,
    },
    DeleteService {
        simulation: String,
        service: String,
    },
    CreateInstruction {
        simulation: String,
        node: String,
        data: InstructionData,
        #[serde(default)]
        run_again_on_rebuild: bool,
    },
    DeleteInstruction {
        simulation: String,
        instruction: String,
    },
    CreateZtp {
        simulation: String,
        content: String,
    },
    UpdateZtp {
        simulation: String,
        content: String,
    },
    DeleteZtp {
        simulation: String,
    },
    Checkpoint {
        simulation: String,
        name: String,
    },
    DeleteCheckpoint {
        simulation: String,
        checkpoint: String,
    },
    ResetNodes {
        simulation: String,
        nodes: Vec<String>,
        #[serde(default)]
        rebuild: bool,
    },
    History {
        simulation: String,
    },
    HistoryFilters {
        simulation: String,
    },
    Resources {
        simulation: String,
    },
    Transfer {
        simulation: String,
        source: String,
        destination: String,
        bytes: usize,
    },
    AllReduce {
        simulation: String,
        #[serde(default)]
        execution: NcclOptions,
        ranks: Vec<String>,
        inputs: Vec<Vec<f64>>,
        reduction: Reduction,
    },
    Broadcast {
        simulation: String,
        #[serde(default)]
        execution: NcclOptions,
        ranks: Vec<String>,
        root: usize,
        input: Vec<f64>,
    },
    AllGather {
        simulation: String,
        #[serde(default)]
        execution: NcclOptions,
        ranks: Vec<String>,
        inputs: Vec<Vec<f64>>,
    },
    ReduceScatter {
        simulation: String,
        #[serde(default)]
        execution: NcclOptions,
        ranks: Vec<String>,
        inputs: Vec<Vec<f64>>,
        reduction: Reduction,
    },
}
fn limit() -> usize {
    100
}
fn value<T: Serialize>(result: T) -> Result<Value> {
    Ok(serde_json::to_value(result)?)
}
impl Simulator {
    pub fn execute(&mut self, command: Command) -> Result<Value> {
        match command {
            Command::Create { name } => value(self.create(&name)?),
            Command::List {
                name,
                state,
                offset,
                limit,
            } => value(self.list(name.as_deref(), state, offset, limit)),
            Command::Get { simulation } => value(self.get(&simulation)?),
            Command::Delete { simulation } => {
                self.delete(&simulation)?;
                Ok(Value::Null)
            }
            Command::Update { simulation, name } => {
                self.get_mut(&simulation)?.update(Some(&name), None)?;
                Ok(Value::Null)
            }
            Command::Import {
                manifest,
                attempt_start,
            } => value(self.import(manifest, attempt_start)?),
            Command::Export { simulation } => value(self.get(&simulation)?.export()?),
            Command::Clone {
                simulation,
                name,
                checkpoint,
                attempt_start,
            } => value(self.clone_simulation(
                &simulation,
                &name,
                checkpoint.as_deref(),
                attempt_start,
            )?),
            Command::Start {
                simulation,
                checkpoint,
            } => {
                self.get_mut(&simulation)?.start(checkpoint.as_deref())?;
                Ok(Value::Null)
            }
            Command::Shutdown {
                simulation,
                create_checkpoint,
            } => value(self.get_mut(&simulation)?.shutdown(create_checkpoint)?),
            Command::Rebuild {
                simulation,
                checkpoint,
            } => {
                self.get_mut(&simulation)?.rebuild(checkpoint.as_deref())?;
                Ok(Value::Null)
            }
            Command::CreateNode { simulation, node } => {
                value(self.get_mut(&simulation)?.create_node(node)?)
            }
            Command::UpdateNode {
                simulation,
                node_id,
                node,
            } => {
                self.get_mut(&simulation)?.update_node(&node_id, node)?;
                Ok(Value::Null)
            }
            Command::DeleteNode { simulation, node } => {
                self.get_mut(&simulation)?.delete_node(&node)?;
                Ok(Value::Null)
            }
            Command::CreateInterface {
                simulation,
                node,
                name,
                interface_type,
            } => value(self.get_mut(&simulation)?.create_interface(
                &node,
                &name,
                interface_type,
            )?),
            Command::DeleteInterface {
                simulation,
                interface,
            } => {
                self.get_mut(&simulation)?.delete_interface(&interface)?;
                Ok(Value::Null)
            }
            Command::Breakout {
                simulation,
                interface,
                split_count,
            } => value(
                self.get_mut(&simulation)?
                    .breakout(&interface, split_count)?,
            ),
            Command::RevertBreakout {
                simulation,
                interface,
            } => {
                self.get_mut(&simulation)?.revert_breakout(&interface)?;
                Ok(Value::Null)
            }
            Command::CreateLink {
                simulation,
                interfaces,
                condition,
            } => value(
                self.get_mut(&simulation)?
                    .create_link([&interfaces[0], &interfaces[1]], condition)?,
            ),
            Command::DeleteLink { simulation, link } => {
                self.get_mut(&simulation)?.delete_link(&link)?;
                Ok(Value::Null)
            }
            Command::SetLink {
                simulation,
                link,
                condition,
            } => {
                self.get_mut(&simulation)?.set_link(&link, condition)?;
                Ok(Value::Null)
            }
            Command::AutoOob {
                simulation,
                enabled,
                dhcp,
            } => {
                self.get_mut(&simulation)?.set_auto_oob(enabled, dhcp)?;
                Ok(Value::Null)
            }
            Command::CreateService {
                simulation,
                interface,
                name,
                node_port,
                service_type,
            } => value(self.get_mut(&simulation)?.create_service(
                &interface,
                &name,
                node_port,
                service_type,
            )?),
            Command::DeleteService {
                simulation,
                service,
            } => {
                self.get_mut(&simulation)?.delete_service(&service)?;
                Ok(Value::Null)
            }
            Command::CreateInstruction {
                simulation,
                node,
                data,
                run_again_on_rebuild,
            } => value(self.get_mut(&simulation)?.create_instruction(
                &node,
                data,
                run_again_on_rebuild,
            )?),
            Command::DeleteInstruction {
                simulation,
                instruction,
            } => {
                self.get_mut(&simulation)?
                    .delete_instruction(&instruction)?;
                Ok(Value::Null)
            }
            Command::CreateZtp {
                simulation,
                content,
            } => {
                self.get_mut(&simulation)?.create_ztp_script(content)?;
                Ok(Value::Null)
            }
            Command::UpdateZtp {
                simulation,
                content,
            } => {
                self.get_mut(&simulation)?.update_ztp_script(content)?;
                Ok(Value::Null)
            }
            Command::DeleteZtp { simulation } => {
                self.get_mut(&simulation)?.delete_ztp_script()?;
                Ok(Value::Null)
            }
            Command::Checkpoint { simulation, name } => {
                value(self.get_mut(&simulation)?.create_checkpoint(&name)?)
            }
            Command::DeleteCheckpoint {
                simulation,
                checkpoint,
            } => {
                self.get_mut(&simulation)?.delete_checkpoint(&checkpoint)?;
                Ok(Value::Null)
            }
            Command::ResetNodes {
                simulation,
                nodes,
                rebuild,
            } => {
                self.get_mut(&simulation)?.reset_nodes(&nodes, rebuild)?;
                Ok(Value::Null)
            }
            Command::History { simulation } => value(self.get(&simulation)?.history()),
            Command::HistoryFilters { simulation } => {
                value(self.get(&simulation)?.history_filters())
            }
            Command::Resources { simulation } => {
                value(self.get(&simulation)?.required_resources()?)
            }
            Command::Transfer {
                simulation,
                source,
                destination,
                bytes,
            } => value(
                self.get_mut(&simulation)?
                    .transfer(&source, &destination, bytes)?,
            ),
            Command::AllReduce {
                simulation,
                ranks,
                inputs,
                reduction,
                execution,
            } => value(self.get(&simulation)?.collective(
                &ranks,
                &Collective::AllReduce { inputs, reduction },
                &execution,
            )?),
            Command::Broadcast {
                simulation,
                ranks,
                root,
                input,
                execution,
            } => value(self.get(&simulation)?.collective(
                &ranks,
                &Collective::Broadcast { root, input },
                &execution,
            )?),
            Command::AllGather {
                simulation,
                ranks,
                inputs,
                execution,
            } => value(self.get(&simulation)?.collective(
                &ranks,
                &Collective::AllGather { inputs },
                &execution,
            )?),
            Command::ReduceScatter {
                simulation,
                ranks,
                inputs,
                reduction,
                execution,
            } => value(self.get(&simulation)?.collective(
                &ranks,
                &Collective::ReduceScatter { inputs, reduction },
                &execution,
            )?),
        }
    }
    pub fn respond(&mut self, line: &str) -> Value {
        let result = serde_json::from_str::<Command>(line)
            .map_err(Error::from)
            .and_then(|c| self.execute(c));
        match result {
            Ok(result) => json!({"ok":true,"result":result}),
            Err(error) => json!({"ok":false,"error":error.to_string()}),
        }
    }
}
