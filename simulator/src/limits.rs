// SPDX-License-Identifier: RPL-1.5
//! Per-simulation retention budgets. Byte counts are serialized sizes, not RSS.
use crate::{Error, Result, Simulation};
use serde::Serialize;
use std::io::{self, Write};

#[derive(Debug, Default)]
pub(crate) struct DataBytesCache(std::sync::atomic::AtomicUsize);
impl Clone for DataBytesCache {
    fn clone(&self) -> Self {
        let copy = Self::default();
        copy.set(self.get());
        copy
    }
}
impl DataBytesCache {
    pub(crate) fn get(&self) -> Option<usize> {
        self.0
            .load(std::sync::atomic::Ordering::Relaxed)
            .checked_sub(1)
    }
    pub(crate) fn set(&self, bytes: Option<usize>) {
        self.0.store(
            bytes.and_then(|n| n.checked_add(1)).unwrap_or(0),
            std::sync::atomic::Ordering::Relaxed,
        );
    }
}

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
    pub(crate) fn retained_transaction<T>(
        &mut self,
        change: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        let mut candidate = self.clone();
        let result = change(&mut candidate)?;
        candidate.data_bytes_cache.set(None);
        candidate.validate_limits(&candidate.limits)?;
        *self = candidate;
        Ok(result)
    }
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
    pub(crate) fn data_bytes(&self) -> Result<usize> {
        if let Some(bytes) = self.data_bytes_cache.get() {
            return Ok(bytes);
        }
        let bytes = self.recompute_data_bytes()?;
        self.data_bytes_cache.set(Some(bytes));
        Ok(bytes)
    }
    pub(crate) fn recompute_data_bytes(&self) -> Result<usize> {
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

#[cfg(test)]
mod tests {
    use crate::{Simulator, model::*};
    #[test]
    fn incremental_accounting_and_indexes_match_full_reference_after_edits() {
        let mut api = Simulator::new();
        let id = api.create("accounting").unwrap().id().to_owned();
        let sim = api.get_mut(&id).unwrap();
        sim.set_auto_oob(true, true).unwrap();
        for i in 0..128 {
            let mut spec = NodeSpec::host(&format!("host-{i}"));
            spec.labels
                .insert("escaped".into(), format!("a\"b\n{i}").into());
            let node = sim.create_node(spec).unwrap();
            sim.create_interface(&node, "eth0", InterfaceType::Data)
                .unwrap();
            assert_eq!(
                sim.usage().unwrap().data_bytes,
                sim.recompute_data_bytes().unwrap()
            );
            assert_eq!(sim.node_named(&format!("host-{i}")).unwrap().id, node);
            if i % 5 == 0 {
                sim.delete_node(&node).unwrap();
                assert!(sim.node_named(&format!("host-{i}")).is_err());
                assert_eq!(
                    sim.usage().unwrap().data_bytes,
                    sim.recompute_data_bytes().unwrap()
                );
            }
        }
        sim.start(None).unwrap();
        sim.shutdown(false).unwrap();
        sim.rebuild(None).unwrap();
        assert_eq!(
            sim.usage().unwrap().data_bytes,
            sim.recompute_data_bytes().unwrap()
        );
        let leases: std::collections::BTreeSet<_> = sim
            .nodes()
            .filter_map(|n| n.management_ip.clone())
            .collect();
        assert_eq!(leases.len(), sim.nodes().count());
    }
}
