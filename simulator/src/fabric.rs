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

//! Software reliable-connected verbs model with checked memory regions and completions.
//! The hardware backend in rdma.rs executes the corresponding rust-ibverbs operations.
use crate::{id, model::*, topology::Transmission};
use serde::Serialize;
use std::{
    collections::{BTreeMap, VecDeque},
    ops::Range,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum QpState {
    Reset,
    Rts,
    Error,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Opcode {
    Send,
    Receive,
    RdmaWrite,
}
#[derive(Debug, Clone, Serialize)]
pub struct Completion {
    pub wr_id: u64,
    pub opcode: Opcode,
    pub bytes: usize,
    pub completed_ns: u64,
}
#[derive(Debug, Clone)]
struct Memory {
    node: String,
    generation: u64,
    bytes: Vec<u8>,
    remote_write: bool,
    rkey: String,
}
#[derive(Debug, Clone)]
struct Receive {
    mr: String,
    range: Range<usize>,
    wr_id: u64,
}
#[derive(Debug, Clone)]
struct Qp {
    node: String,
    generation: u64,
    state: QpState,
    peer: Option<String>,
    receives: VecDeque<Receive>,
    completions: VecDeque<Completion>,
}
/// Aggregate per-fabric budgets, separate from the simulation's serialized state.
#[derive(Debug, Clone)]
pub struct FabricLimits {
    pub queue_pairs: usize,
    pub memory_regions: usize,
    pub queued_entries: usize,
    pub payload_bytes: usize,
}
impl Default for FabricLimits {
    fn default() -> Self {
        Self {
            queue_pairs: 4096,
            memory_regions: 4096,
            queued_entries: 65536,
            payload_bytes: 64 * 1024 * 1024,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FabricUsage {
    pub queue_pairs: usize,
    pub memory_regions: usize,
    pub queued_entries: usize,
    pub payload_bytes: usize,
}
#[derive(Debug, Clone)]
pub struct Fabric {
    simulation: String,
    memory: BTreeMap<String, Memory>,
    qps: BTreeMap<String, Qp>,
    capacity: usize,
    limits: FabricLimits,
    queued_entries: usize,
    payload_bytes: usize,
}
impl Fabric {
    pub fn new(sim: &Simulation, capacity: usize) -> Result<Self> {
        Self::with_limits(sim, capacity, FabricLimits::default())
    }
    pub fn with_limits(sim: &Simulation, capacity: usize, limits: FabricLimits) -> Result<Self> {
        if !(1..=65536).contains(&capacity) {
            return Err(Error::Invalid("queue capacity must be 1..65536".into()));
        }
        Ok(Self {
            simulation: sim.id().into(),
            memory: BTreeMap::new(),
            qps: BTreeMap::new(),
            capacity,
            limits,
            queued_entries: 0,
            payload_bytes: 0,
        })
    }
    pub fn usage(&self) -> FabricUsage {
        FabricUsage {
            queue_pairs: self.qps.len(),
            memory_regions: self.memory.len(),
            queued_entries: self.queued_entries,
            payload_bytes: self.payload_bytes,
        }
    }
    fn queue_capacity(&self, additional: usize) -> Result<()> {
        crate::limits::capacity(
            "aggregate fabric queue entries",
            self.queued_entries,
            additional,
            self.limits.queued_entries,
        )
    }
    fn check_sim(&self, sim: &Simulation) -> Result<()> {
        if sim.id() != self.simulation {
            return Err(Error::Conflict(
                "fabric belongs to another simulation".into(),
            ));
        }
        Ok(())
    }
    fn qp<'a>(&'a self, sim: &Simulation, key: &str) -> Result<&'a Qp> {
        self.check_sim(sim)?;
        let qp = self
            .qps
            .get(key)
            .ok_or_else(|| Error::NotFound(key.into()))?;
        if sim.node(&qp.node)?.generation != qp.generation {
            return Err(Error::State("stale queue pair after reset/shutdown".into()));
        }
        Ok(qp)
    }
    fn mr<'a>(&'a self, sim: &Simulation, key: &str) -> Result<&'a Memory> {
        self.check_sim(sim)?;
        let mr = self
            .memory
            .get(key)
            .ok_or_else(|| Error::NotFound(key.into()))?;
        if sim.node(&mr.node)?.generation != mr.generation {
            return Err(Error::State("stale memory registration".into()));
        }
        Ok(mr)
    }
    pub fn register(
        &mut self,
        sim: &Simulation,
        node: &str,
        bytes: Vec<u8>,
        remote_write: bool,
    ) -> Result<String> {
        self.check_sim(sim)?;
        let n = sim.node(node)?;
        if bytes.is_empty() || bytes.len() > 16 * 1024 * 1024 {
            return Err(Error::Invalid(
                "memory region must be 1 byte..16 MiB".into(),
            ));
        }
        crate::limits::capacity(
            "fabric memory regions",
            self.memory.len(),
            1,
            self.limits.memory_regions,
        )?;
        crate::limits::capacity(
            "fabric payload bytes",
            self.payload_bytes,
            bytes.len(),
            self.limits.payload_bytes,
        )?;
        self.payload_bytes += bytes.len();
        let key = id();
        self.memory.insert(
            key.clone(),
            Memory {
                node: node.into(),
                generation: n.generation,
                bytes,
                remote_write,
                rkey: id(),
            },
        );
        Ok(key)
    }
    pub fn bytes(&self, sim: &Simulation, mr: &str) -> Result<&[u8]> {
        Ok(&self.mr(sim, mr)?.bytes)
    }
    pub fn remote_key(&self, sim: &Simulation, mr: &str) -> Result<&str> {
        Ok(&self.mr(sim, mr)?.rkey)
    }
    pub fn deregister(&mut self, mr: &str) -> Result<()> {
        if self
            .qps
            .values()
            .any(|q| q.receives.iter().any(|r| r.mr == mr))
        {
            return Err(Error::Conflict("memory has a posted receive".into()));
        }
        let removed = self
            .memory
            .remove(mr)
            .ok_or_else(|| Error::NotFound(mr.into()))?;
        self.payload_bytes -= removed.bytes.len();
        Ok(())
    }
    pub fn create_qp(&mut self, sim: &Simulation, node: &str) -> Result<String> {
        self.check_sim(sim)?;
        crate::limits::capacity(
            "fabric queue pairs",
            self.qps.len(),
            1,
            self.limits.queue_pairs,
        )?;
        let n = sim.node(node)?;
        let key = id();
        self.qps.insert(
            key.clone(),
            Qp {
                node: node.into(),
                generation: n.generation,
                state: QpState::Reset,
                peer: None,
                receives: VecDeque::new(),
                completions: VecDeque::new(),
            },
        );
        Ok(key)
    }
    pub fn state(&self, sim: &Simulation, qp: &str) -> Result<QpState> {
        Ok(self.qp(sim, qp)?.state)
    }
    pub fn connect(&mut self, sim: &Simulation, first: &str, second: &str) -> Result<()> {
        let a = self.qp(sim, first)?;
        let b = self.qp(sim, second)?;
        if first == second || a.state != QpState::Reset || b.state != QpState::Reset {
            return Err(Error::State(
                "connect requires two distinct reset QPs".into(),
            ));
        }
        for (x, y) in [(first, second), (second, first)] {
            let q = self.qps.get_mut(x).unwrap();
            q.peer = Some(y.into());
            q.state = QpState::Rts;
        }
        Ok(())
    }
    pub fn destroy_qp(&mut self, qp: &str) -> Result<()> {
        let q = self
            .qps
            .remove(qp)
            .ok_or_else(|| Error::NotFound(qp.into()))?;
        self.queued_entries -= q.receives.len() + q.completions.len();
        if let Some(peer) = q.peer
            && let Some(peer) = self.qps.get_mut(&peer)
        {
            peer.state = QpState::Error;
            peer.peer = None;
        }
        Ok(())
    }
    pub fn post_receive(
        &mut self,
        sim: &Simulation,
        qp: &str,
        mr: &str,
        range: Range<usize>,
        wr_id: u64,
    ) -> Result<()> {
        let q = self.qp(sim, qp)?;
        let m = self.mr(sim, mr)?;
        check_range(&range, m.bytes.len())?;
        if q.node != m.node {
            return Err(Error::Conflict(
                "memory is not in QP protection domain".into(),
            ));
        }
        if q.state != QpState::Rts || q.receives.len() >= self.capacity {
            return Err(Error::State("QP not ready or receive queue full".into()));
        }
        self.queue_capacity(1)?;
        self.queued_entries += 1;
        self.qps.get_mut(qp).unwrap().receives.push_back(Receive {
            mr: mr.into(),
            range,
            wr_id,
        });
        Ok(())
    }
    pub fn send(
        &mut self,
        sim: &mut Simulation,
        qp: &str,
        mr: &str,
        range: Range<usize>,
        wr_id: u64,
    ) -> Result<Transmission> {
        let sender = self.qp(sim, qp)?.clone();
        let peer_id = sender
            .peer
            .clone()
            .ok_or_else(|| Error::State("QP is not connected".into()))?;
        let receiver = self.qp(sim, &peer_id)?.clone();
        let source = self.mr(sim, mr)?;
        check_range(&range, source.bytes.len())?;
        if source.node != sender.node {
            return Err(Error::Conflict(
                "send memory has wrong protection domain".into(),
            ));
        }
        if sender.state != QpState::Rts || receiver.state != QpState::Rts {
            return Err(Error::State("QP not ready".into()));
        }
        if sender.completions.len() >= self.capacity || receiver.completions.len() >= self.capacity
        {
            return Err(Error::Conflict("completion queue full".into()));
        }
        let receive = receiver
            .receives
            .front()
            .ok_or_else(|| Error::State("receiver not ready (RNR)".into()))?
            .clone();
        let target = self.mr(sim, &receive.mr)?;
        check_range(&receive.range, target.bytes.len())?;
        if range.len() > receive.range.len() {
            return Err(Error::Invalid("receive buffer too small".into()));
        }
        self.queue_capacity(1)?; // one receive is consumed, two completions are added
        let bytes = source.bytes[range.clone()].to_vec();
        let transfer = sim.transfer(&sender.node, &receiver.node, bytes.len())?;
        self.queued_entries += 1;
        let destination = &mut self.memory.get_mut(&receive.mr).unwrap().bytes;
        destination[receive.range.start..receive.range.start + bytes.len()].copy_from_slice(&bytes);
        self.qps
            .get_mut(qp)
            .unwrap()
            .completions
            .push_back(Completion {
                wr_id,
                opcode: Opcode::Send,
                bytes: bytes.len(),
                completed_ns: transfer.completed_ns,
            });
        let receiver = self.qps.get_mut(&peer_id).unwrap();
        receiver.receives.pop_front();
        receiver.completions.push_back(Completion {
            wr_id: receive.wr_id,
            opcode: Opcode::Receive,
            bytes: bytes.len(),
            completed_ns: transfer.completed_ns,
        });
        Ok(transfer)
    }
    #[allow(clippy::too_many_arguments)]
    pub fn write(
        &mut self,
        sim: &mut Simulation,
        qp: &str,
        source_mr: &str,
        source_range: Range<usize>,
        target_mr: &str,
        target_offset: usize,
        rkey: &str,
        wr_id: u64,
    ) -> Result<Transmission> {
        let q = self.qp(sim, qp)?.clone();
        let peer = self
            .qp(
                sim,
                q.peer
                    .as_deref()
                    .ok_or_else(|| Error::State("QP not connected".into()))?,
            )?
            .clone();
        let source = self.mr(sim, source_mr)?;
        let target = self.mr(sim, target_mr)?;
        check_range(&source_range, source.bytes.len())?;
        let end = target_offset
            .checked_add(source_range.len())
            .ok_or_else(|| Error::Invalid("offset overflow".into()))?;
        check_range(&(target_offset..end), target.bytes.len())?;
        if q.state != QpState::Rts || peer.state != QpState::Rts {
            return Err(Error::State("QP not ready".into()));
        }
        if source.node != q.node
            || target.node != peer.node
            || !target.remote_write
            || target.rkey != rkey
        {
            return Err(Error::Conflict("RDMA protection error".into()));
        }
        if q.completions.len() >= self.capacity {
            return Err(Error::Conflict("completion queue full".into()));
        }
        self.queue_capacity(1)?;
        let bytes = source.bytes[source_range].to_vec();
        let transfer = sim.transfer(&q.node, &peer.node, bytes.len())?;
        self.queued_entries += 1;
        self.memory.get_mut(target_mr).unwrap().bytes[target_offset..end].copy_from_slice(&bytes);
        self.qps
            .get_mut(qp)
            .unwrap()
            .completions
            .push_back(Completion {
                wr_id,
                opcode: Opcode::RdmaWrite,
                bytes: bytes.len(),
                completed_ns: transfer.completed_ns,
            });
        Ok(transfer)
    }
    pub fn poll(&mut self, sim: &Simulation, qp: &str) -> Result<Option<Completion>> {
        self.qp(sim, qp)?;
        let result = self.qps.get_mut(qp).unwrap().completions.pop_front();
        self.queued_entries -= usize::from(result.is_some());
        Ok(result)
    }
}
fn check_range(range: &Range<usize>, len: usize) -> Result<()> {
    if range.start > range.end || range.end > len {
        return Err(Error::Invalid("memory range is out of bounds".into()));
    }
    Ok(())
}
