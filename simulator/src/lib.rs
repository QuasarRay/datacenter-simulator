//! A Rust implementation of local datacenter behavior described by the Air SDK.
//! All mutations validate before commit; no Python runtime or Python bindings.
#![deny(unsafe_op_in_unsafe_fn)]
pub mod api;
pub mod collective;
pub mod command;
pub mod fabric;
pub mod manifest;
pub mod model;
pub mod topology;
#[cfg(feature = "linux")]
pub mod linux;
#[cfg(feature = "rdma")]
pub mod rdma;
#[cfg(feature = "nccl")]
pub mod nccl_backend;

pub use api::Simulator;
pub use model::{Error, Result, Simulation};

pub(crate) fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
