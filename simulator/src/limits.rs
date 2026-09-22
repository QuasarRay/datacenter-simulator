// SPDX-License-Identifier: RPL-1.5
//! Per-simulation retention budgets. Byte counts are serialized sizes, not RSS.
use crate::{Error, Result, Simulation};
use serde::Serialize;
use std::io::{self, Write};

#[derive(Clone, Debug, Serialize)]
pub struct Limits {
    pub interfaces: usize,
    pub links: usize,
    pub services: usize,
    pub instructions: usize,
    pub checkpoints: usize,
    pub data_bytes: usize,
    pub checkpoint_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            interfaces: 32768,
            links: 16384,
            services: 4096,
            instructions: 4096,
            checkpoints: 16,
            data_bytes: 32 * 1024 * 1024,
            checkpoint_bytes: 64 * 1024 * 1024,
        }
    }
}
#[derive(Debug, Serialize)]
pub struct Usage {
    pub interfaces: usize,
    pub links: usize,
    pub services: usize,
    pub instructions: usize,
    pub checkpoints: usize,
    pub data_bytes: usize,
    pub checkpoint_bytes: usize,
}
pub(crate) fn size(value: &impl Serialize) -> Result<usize> {
    struct Count(usize);
    impl Write for Count {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0 = self
                .0
                .checked_add(bytes.len())
                .ok_or_else(|| io::Error::other("size overflow"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut count = Count(0);
    serde_json::to_writer(&mut count, value)?;
    Ok(count.0)
}
pub(crate) fn capacity(label: &str, used: usize, added: usize, limit: usize) -> Result<()> {
    if used.checked_add(added).is_none_or(|n| n > limit) {
        return Err(Error::Conflict(format!(
            "{label} capacity exceeded ({limit})"
        )));
    }
    Ok(())
}
impl Simulation {
    pub fn limits(&self) -> &Limits {
        &self.limits
    }
    pub fn usage(&self) -> Result<Usage> {
        Ok(Usage {
            interfaces: self.interfaces.len(),
            links: self.links.len(),
            services: self.services.len(),
            instructions: self.instructions.len(),
            checkpoints: self.checkpoints.len(),
            data_bytes: self.data_bytes()?,
            checkpoint_bytes: self.checkpoints.values().map(|c| c.retained_bytes).sum(),
        })
    }
    pub fn set_limits(&mut self, limits: Limits) -> Result<()> {
        self.validate_limits(&limits)?;
        self.limits = limits;
        Ok(())
    }
    pub(crate) fn validate_limits(&self, limits: &Limits) -> Result<()> {
        let u = self.usage()?;
        for (name, used, max) in [
            ("interfaces", u.interfaces, limits.interfaces),
            ("links", u.links, limits.links),
            ("services", u.services, limits.services),
            ("instructions", u.instructions, limits.instructions),
            ("checkpoints", u.checkpoints, limits.checkpoints),
            ("retained data bytes", u.data_bytes, limits.data_bytes),
            (
                "checkpoint bytes",
                u.checkpoint_bytes,
                limits.checkpoint_bytes,
            ),
        ] {
            capacity(name, used, 0, max)?;
        }
        Ok(())
    }
    fn data_bytes(&self) -> Result<usize> {
        // Fixed-width interfaces/links are bounded by counts. Include every
        // variable-size payload, including files left by deleted instructions.
        size(&(
            &self.name,
            &self.metadata,
            &self.nodes,
            &self.services,
            &self.instructions,
            &self.runtime,
            &self.ztp_script,
        ))
    }
    pub(crate) fn data_change(&self, old: &impl Serialize, new: &impl Serialize) -> Result<()> {
        capacity(
            "retained data bytes",
            self.data_bytes()?.saturating_sub(size(old)?),
            size(new)?,
            self.limits.data_bytes,
        )
    }
}
