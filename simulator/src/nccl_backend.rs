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

//! Native NCCL submission for application-owned CUDA buffers.
//! The caller supplies rank/device placement and CUDA stream synchronization.
//! This module contains Rust-to-C NCCL calls only, never Python interoperation.
use crate::model::{Error, Result, Role, Simulation};
pub use nccl::{CudaStream, NcclType, Reduction, UniqueId};

pub struct NcclRank {
    communicator: nccl::Communicator,
    node: String,
    simulation: String,
    generation: u64,
}
impl NcclRank {
    /// Invoke concurrently for each rank after selecting the corresponding CUDA device.
    pub fn connect(
        sim: &Simulation,
        ranks: &[String],
        rank: usize,
        unique_id: UniqueId,
    ) -> Result<Self> {
        sim.active()?;
        if ranks.is_empty() || ranks.len() > i32::MAX as usize || rank >= ranks.len() {
            return Err(Error::Invalid("invalid NCCL ranks".into()));
        }
        if ranks
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != ranks.len()
        {
            return Err(Error::Invalid("duplicate NCCL nodes".into()));
        }
        for node in ranks {
            if sim.node(node)?.spec.role != Role::Host {
                return Err(Error::Invalid("NCCL ranks must be hosts".into()));
            }
            sim.route(&ranks[rank], node, 0)?;
        }
        let communicator =
            nccl::Communicator::init_rank(ranks.len() as i32, unique_id, rank as i32)
                .map_err(native_error)?;
        let node = sim.node(&ranks[rank])?;
        Ok(Self {
            communicator,
            node: node.id.clone(),
            simulation: sim.id().into(),
            generation: node.generation,
        })
    }
    fn validate(&self, sim: &Simulation) -> Result<()> {
        sim.active()?;
        if self.simulation != sim.id() || sim.node(&self.node)?.generation != self.generation {
            return Err(Error::State(
                "NCCL rank is stale or belongs to another simulation".into(),
            ));
        }
        Ok(())
    }
    /// # Safety
    /// Select this rank's CUDA device. Pointers must address at least count values
    /// of CUDA-accessible memory and satisfy NCCL aliasing rules. Buffers, stream
    /// and rank must remain alive until the caller synchronizes the stream.
    pub unsafe fn all_reduce<T: NcclType>(
        &self,
        sim: &Simulation,
        send: *const T,
        recv: *mut T,
        count: usize,
        reduction: Reduction,
        stream: CudaStream,
    ) -> Result<()> {
        self.validate(sim)?;
        // SAFETY: delegated to the explicit caller contract above.
        unsafe {
            self.communicator
                .all_reduce(send, recv, count, reduction.op(), stream)
        }
        .map_err(native_error)
    }
    /// # Safety
    /// Same pointer, device, stream, and lifetime requirements as all_reduce.
    pub unsafe fn broadcast<T: NcclType>(
        &self,
        sim: &Simulation,
        send: *const T,
        recv: *mut T,
        count: usize,
        root: i32,
        stream: CudaStream,
    ) -> Result<()> {
        self.validate(sim)?;
        // SAFETY: caller upholds CUDA memory and asynchronous lifetime requirements.
        unsafe { self.communicator.broadcast(send, recv, count, root, stream) }
            .map_err(native_error)
    }
    pub fn abort(self) -> Result<()> {
        self.communicator.abort().map_err(native_error)
    }
}
fn native_error(error: nccl::Error) -> Error {
    Error::State(format!("NCCL: {error}"))
}
