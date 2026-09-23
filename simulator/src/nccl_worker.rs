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

//! Private worker protocol. Each process creates and runs one real NCCL rank.
use crate::{
    collective::{Collective, Reduction},
    cuda::Cuda,
};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Read, Write};

pub(crate) const MESSAGE_LIMIT: u64 = 512 * 1024 * 1024;
#[derive(Serialize, Deserialize)]
pub(crate) struct Ready {
    pub nonce: String,
    pub device_identity: serde_json::Value,
    pub unique_id: Option<Vec<u8>>,
    pub nccl_version: i32,
}
#[derive(Serialize, Deserialize)]
pub(crate) struct Start {
    pub unique_id: Vec<u8>,
    pub operation: Operation,
    pub input: Vec<f64>,
    pub output_count: usize,
}
#[derive(Clone, Copy, Serialize, Deserialize)]
pub(crate) enum Operation {
    AllReduce(Reduction),
    Broadcast(usize),
    AllGather,
    ReduceScatter(Reduction),
}
#[derive(Serialize, Deserialize)]
pub(crate) struct Finished {
    pub output: Vec<f64>,
    pub elapsed_ns: u64,
}
impl Start {
    pub fn for_rank(request: &Collective, nranks: usize, rank: usize, id: &[u8]) -> Result<Self> {
        let (_, output_count) = request.shape(nranks)?;
        let (operation, input) = match request {
            Collective::AllReduce { inputs, reduction } => {
                (Operation::AllReduce(*reduction), inputs[rank].clone())
            }
            Collective::Broadcast { root, input } => (
                Operation::Broadcast(*root),
                if *root == rank {
                    input.clone()
                } else {
                    vec![0.0; input.len()]
                },
            ),
            Collective::AllGather { inputs } => (Operation::AllGather, inputs[rank].clone()),
            Collective::ReduceScatter { inputs, reduction } => {
                (Operation::ReduceScatter(*reduction), inputs[rank].clone())
            }
        };
        Ok(Self {
            unique_id: id.to_vec(),
            operation,
            input,
            output_count,
        })
    }
    fn validate(&self, nranks: usize) -> Result<()> {
        ensure!(
            self.input.len() <= 16 * 1024 * 1024 / 8 && self.input.iter().all(|v| v.is_finite()),
            "invalid worker input"
        );
        ensure!(
            self.output_count <= 64 * 1024 * 1024 / 8 / nranks,
            "worker output too large"
        );
        let expected = match self.operation {
            Operation::AllReduce(_) => self.input.len(),
            Operation::Broadcast(root) => {
                ensure!(root < nranks, "invalid root");
                self.input.len()
            }
            Operation::AllGather => self
                .input
                .len()
                .checked_mul(nranks)
                .context("count overflow")?,
            Operation::ReduceScatter(_) => {
                ensure!(
                    self.input.len().is_multiple_of(nranks),
                    "uneven reduce-scatter"
                );
                self.input.len() / nranks
            }
        };
        ensure!(
            self.output_count == expected,
            "worker output shape mismatch"
        );
        Ok(())
    }
}
fn reduction(value: Reduction) -> nccl::Reduction {
    match value {
        Reduction::Sum => nccl::Reduction::Sum,
        Reduction::Product => nccl::Reduction::Product,
        Reduction::Min => nccl::Reduction::Min,
        Reduction::Max => nccl::Reduction::Max,
        Reduction::Average => nccl::Reduction::Average,
    }
}
fn emit(value: &impl Serialize) -> Result<()> {
    let mut out = std::io::stdout().lock();
    serde_json::to_writer(&mut out, value)?;
    writeln!(out)?;
    out.flush()?;
    Ok(())
}
/// Called only by the executable's private worker entry point, before any runtime.
pub fn run(args: &[String]) -> Result<()> {
    ensure!(
        !cfg!(feature = "nccl-check"),
        "nccl-check cannot execute NCCL"
    );
    ensure!(
        args.len() == 5,
        "worker requires rank, rank count, CUDA ordinal, session and pinned libraries"
    );
    let rank: usize = args[0].parse()?;
    let nranks: usize = args[1].parse()?;
    let device: i32 = args[2].parse()?;
    ensure!(
        (1..=4096).contains(&nranks) && rank < nranks,
        "invalid NCCL ranks"
    );
    crate::evidence::reject_loader_injection()?;
    let libraries: crate::evidence::NcclLibraries = serde_json::from_str(&args[4])?;
    libraries.verify()?;
    let cuda = Cuda::open(device, &libraries.cuda.path)?;
    libraries.verify_mapped(std::process::id(), true)?;
    // Generate the bootstrap listener INSIDE rank zero's network namespace.
    let unique_id = if rank == 0 {
        Some(nccl::UniqueId::generate()?.to_bytes().to_vec())
    } else {
        None
    };
    emit(&Ready {
        nonce: args[3].clone(),
        device_identity: cuda.identity.clone(),
        unique_id,
        nccl_version: nccl::version()?,
    })?;
    let mut line = String::new();
    std::io::stdin()
        .lock()
        .take(MESSAGE_LIMIT + 1)
        .read_line(&mut line)?;
    ensure!(
        line.ends_with('\n') && line.len() as u64 <= MESSAGE_LIMIT,
        "invalid worker message"
    );
    let start: Start = serde_json::from_str(&line)?;
    start.validate(nranks)?;
    let bytes = start
        .unique_id
        .as_slice()
        .try_into()
        .context("invalid NCCL unique ID length")?;
    let mut send = cuda.allocate(start.input.len())?;
    let recv = cuda.allocate(start.output_count)?;
    send.upload(&start.input)?;
    let stream = cuda.stream()?;
    // Native NCCL chooses and executes the algorithm. No Rust reduction or ring exists.
    let communicator = nccl::Communicator::init_rank(
        nranks as i32,
        nccl::UniqueId::from_bytes(bytes),
        rank as i32,
    )?;
    let began = std::time::Instant::now();
    let result = (|| -> Result<Finished> {
        // SAFETY: disjoint CUDA allocations sized by the validated operation; a
        // single worker owns the communicator, context and stream through completion.
        unsafe {
            match start.operation {
                Operation::AllReduce(op) => communicator.all_reduce(
                    send.pointer(),
                    recv.pointer(),
                    start.input.len(),
                    reduction(op).op(),
                    stream.nccl(),
                )?,
                Operation::Broadcast(root) => communicator.broadcast(
                    send.pointer(),
                    recv.pointer(),
                    start.input.len(),
                    root as i32,
                    stream.nccl(),
                )?,
                Operation::AllGather => communicator.all_gather(
                    send.pointer(),
                    recv.pointer(),
                    start.input.len(),
                    stream.nccl(),
                )?,
                Operation::ReduceScatter(op) => communicator.reduce_scatter(
                    send.pointer(),
                    recv.pointer(),
                    start.output_count,
                    reduction(op).op(),
                    stream.nccl(),
                )?,
            }
        }
        // The supervisor deadline also bounds a hung CUDA synchronization/init call.
        stream.synchronize()?;
        ensure!(
            communicator.async_error()?.as_raw() == nccl::sys::ncclSuccess,
            "NCCL asynchronous failure: {}",
            communicator.last_error()
        );
        let elapsed_ns = began
            .elapsed()
            .as_nanos()
            .try_into()
            .context("elapsed time overflow")?;
        let output = recv.download()?;
        ensure!(
            output.iter().all(|v| v.is_finite()),
            "NCCL returned non-finite output"
        );
        Ok(Finished { output, elapsed_ns })
    })();
    match result {
        Ok(finished) => {
            communicator.destroy()?;
            emit(&finished)
        }
        Err(error) => {
            // If abort itself fails, exit without releasing memory still used by CUDA.
            // The OS/driver tears down this dedicated process's resources.
            if let Err(abort) = communicator.abort() {
                eprintln!("NCCL abort failed: {abort}; original error: {error:#}");
                std::process::exit(1);
            }
            bail!("NCCL rank {rank}: {error:#}")
        }
    }
}
