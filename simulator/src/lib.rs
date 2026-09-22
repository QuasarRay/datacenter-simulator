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

//! A Rust implementation of local datacenter behavior described by the Air SDK.
//! All mutations validate before commit; no Python runtime or Python bindings.
#![deny(unsafe_op_in_unsafe_fn)]
pub mod api;
#[cfg(any(feature = "vm", feature = "mokka"))]
mod bounded_log;
pub mod collective;
pub mod command;
#[cfg(feature = "nccl")]
mod cuda;
pub mod deepops;
#[cfg(feature = "vm")]
pub mod deepops_runtime;
pub mod evidence;
pub mod fabric;
#[cfg(feature = "ibsim")]
pub mod infiniband;
pub mod input;
pub mod limits;
#[cfg(feature = "linux")]
pub mod linux;
pub mod manifest;
pub mod model;
#[cfg(feature = "mokka")]
pub mod mokka;
#[cfg(feature = "nccl")]
pub mod nccl_backend;
#[cfg(feature = "nccl")]
pub mod nccl_worker;
#[cfg(feature = "netbox")]
pub mod netbox;
#[cfg(feature = "mokka")]
mod provenance;
#[cfg(feature = "rdma")]
pub mod rdma;
pub mod topology;
#[cfg(feature = "vm")]
pub mod vm;

pub use api::Simulator;
pub use model::{Error, Result, Simulation};

pub(crate) fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
