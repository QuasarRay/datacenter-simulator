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

//! Collective requests are executed exclusively by native NCCL.
//! This module validates shapes and placement; it contains no collective arithmetic.
use crate::model::{Error, Result, Role, Simulation};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reduction {
    Sum,
    Product,
    Min,
    Max,
    Average,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Collective {
    AllReduce {
        inputs: Vec<Vec<f64>>,
        reduction: Reduction,
    },
    Broadcast {
        root: usize,
        input: Vec<f64>,
    },
    AllGather {
        inputs: Vec<Vec<f64>>,
    },
    ReduceScatter {
        inputs: Vec<Vec<f64>>,
        reduction: Reduction,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NcclOptions {
    /// CUDA ordinals visible to this process, in rank order. Empty selects 0..nranks.
    pub devices: Vec<i32>,
    /// Deadline for execution. Failure cleanup has an additional two-second grace period.
    pub timeout_secs: u64,
}
impl Default for NcclOptions {
    fn default() -> Self {
        Self {
            devices: Vec::new(),
            timeout_secs: 60,
        }
    }
}
impl NcclOptions {
    pub fn devices_for(&self, ranks: usize) -> Result<Vec<i32>> {
        if ranks == 0 || ranks > 4096 || !(1..=3600).contains(&self.timeout_secs) {
            return Err(Error::Invalid(
                "NCCL requires 1..=4096 ranks and a 1..=3600 second timeout".into(),
            ));
        }
        let devices = if self.devices.is_empty() {
            (0..ranks as i32).collect()
        } else {
            self.devices.clone()
        };
        if devices.len() != ranks
            || devices.iter().any(|d| *d < 0)
            || devices.iter().collect::<BTreeSet<_>>().len() != ranks
        {
            return Err(Error::Invalid(
                "assign one distinct CUDA device per NCCL rank".into(),
            ));
        }
        Ok(devices)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RankTraffic {
    pub node: String,
    /// Kernel interface counters including NCCL bootstrap and collective traffic.
    pub tx_bytes: u64,
    pub rx_bytes: u64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CollectiveResult {
    pub backend: String,
    pub transport: String,
    pub nccl_version: i32,
    pub outputs: Vec<Vec<f64>>,
    /// Wall time including worker startup, bootstrap and teardown; never virtual time.
    pub elapsed_ns: u64,
    /// Per-rank wall time from submission through CUDA stream synchronization.
    pub rank_elapsed_ns: Vec<u64>,
    /// Aggregate fabric counters; may include other traffic, never NCCL-only byte attribution.
    pub network_accounting: String,
    pub network: Vec<RankTraffic>,
}

impl Collective {
    /// Return per-rank input and output element counts, using checked allocation bounds.
    pub fn shape(&self, nranks: usize) -> Result<(usize, usize)> {
        if nranks == 0 || nranks > 4096 {
            return Err(Error::Invalid("invalid NCCL rank count".into()));
        }
        let count = match self {
            Self::Broadcast { root, input } => {
                if *root >= nranks {
                    return Err(Error::Invalid("broadcast root is out of range".into()));
                }
                check_buffer(input)?;
                input.len()
            }
            Self::AllReduce { inputs, .. }
            | Self::AllGather { inputs }
            | Self::ReduceScatter { inputs, .. } => {
                if inputs.len() != nranks {
                    return Err(Error::Invalid(
                        "one input buffer per rank is required".into(),
                    ));
                }
                let count = inputs[0].len();
                for input in inputs {
                    check_buffer(input)?;
                    if input.len() != count {
                        return Err(Error::Invalid("NCCL requires equal input counts".into()));
                    }
                }
                count
            }
        };
        let output = match self {
            Self::AllGather { .. } => count.checked_mul(nranks).ok_or_else(memory_limit)?,
            Self::ReduceScatter { .. } => {
                if !count.is_multiple_of(nranks) {
                    return Err(Error::Invalid(
                        "reduce-scatter input count must be divisible by rank count".into(),
                    ));
                }
                count / nranks
            }
            _ => count,
        };
        for size in [count, output] {
            if size
                .checked_mul(nranks)
                .and_then(|v| v.checked_mul(8))
                .is_none_or(|v| v > 64 * 1024 * 1024)
            {
                return Err(memory_limit());
            }
        }
        Ok((count, output))
    }
    pub(crate) fn validate(&self, sim: &Simulation, ranks: &[String]) -> Result<()> {
        sim.active()?;
        self.shape(ranks.len())?;
        if ranks.iter().collect::<BTreeSet<_>>().len() != ranks.len() {
            return Err(Error::Invalid("duplicate NCCL rank nodes".into()));
        }
        for node in ranks {
            if sim.node(node)?.spec.role != Role::Host {
                return Err(Error::Invalid("NCCL ranks must be hosts".into()));
            }
        }
        sim.validate_rank_connectivity(ranks)
    }
}
fn memory_limit() -> Error {
    Error::Invalid("NCCL aggregate input/output exceeds 64 MiB".into())
}
fn check_buffer(input: &[f64]) -> Result<()> {
    if input.len() > 16 * 1024 * 1024 / 8 || input.iter().any(|v| !v.is_finite()) {
        return Err(Error::Invalid(
            "NCCL inputs must be finite and at most 16 MiB per rank".into(),
        ));
    }
    Ok(())
}

impl Simulation {
    /// Execute NCCL over a fresh Linux fabric. Initialize patchbay before starting threads.
    /// Async callers should use LinuxFabric::collective on their existing lab.
    pub fn collective(
        &self,
        ranks: &[String],
        request: &Collective,
        options: &NcclOptions,
    ) -> Result<CollectiveResult> {
        request.validate(self, ranks)?;
        options.devices_for(ranks.len())?;
        #[cfg(all(feature = "nccl", not(feature = "nccl-check")))]
        {
            crate::nccl_backend::execute(self, ranks, request, options)
        }
        #[cfg(any(not(feature = "nccl"), feature = "nccl-check"))]
        {
            Err(Error::Unsupported("collectives require a native --features nccl build, CUDA GPUs and libnccl; nccl-check cannot execute collectives".into()))
        }
    }
    pub fn all_reduce(
        &self,
        ranks: &[String],
        inputs: &[Vec<f64>],
        reduction: Reduction,
    ) -> Result<CollectiveResult> {
        self.collective(
            ranks,
            &Collective::AllReduce {
                inputs: inputs.to_vec(),
                reduction,
            },
            &NcclOptions::default(),
        )
    }
    pub fn broadcast(
        &self,
        ranks: &[String],
        root: usize,
        input: &[f64],
    ) -> Result<CollectiveResult> {
        self.collective(
            ranks,
            &Collective::Broadcast {
                root,
                input: input.to_vec(),
            },
            &NcclOptions::default(),
        )
    }
    pub fn all_gather(&self, ranks: &[String], inputs: &[Vec<f64>]) -> Result<CollectiveResult> {
        self.collective(
            ranks,
            &Collective::AllGather {
                inputs: inputs.to_vec(),
            },
            &NcclOptions::default(),
        )
    }
    pub fn reduce_scatter(
        &self,
        ranks: &[String],
        inputs: &[Vec<f64>],
        reduction: Reduction,
    ) -> Result<CollectiveResult> {
        self.collective(
            ranks,
            &Collective::ReduceScatter {
                inputs: inputs.to_vec(),
                reduction,
            },
            &NcclOptions::default(),
        )
    }
}
