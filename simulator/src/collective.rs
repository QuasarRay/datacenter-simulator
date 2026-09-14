//! Portable semantic and traffic models for NCCL collectives.
//! Ring all-reduce uses reduce-scatter followed by all-gather, grounded in
//! vendor/nccl/src/device/all_reduce.h. Timings are model estimates, not CUDA benchmarks.
use crate::{model::*, topology::Transmission};
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
#[derive(Debug, Clone, Serialize)]
pub struct CollectiveResult {
    pub outputs: Vec<Vec<f64>>,
    pub transfers: Vec<Transmission>,
    pub started_ns: u64,
    pub completed_ns: u64,
}
fn reduce(a: f64, b: f64, op: Reduction) -> f64 {
    match op {
        Reduction::Sum | Reduction::Average => a + b,
        Reduction::Product => a * b,
        Reduction::Min => a.min(b),
        Reduction::Max => a.max(b),
    }
}
fn validate(sim: &Simulation, ranks: &[String], inputs: &[Vec<f64>]) -> Result<usize> {
    sim.active()?;
    if ranks.is_empty() || ranks.len() != inputs.len() || ranks.len() > 4096 {
        return Err(Error::Invalid(
            "one buffer is required per rank, with 1..4096 ranks".into(),
        ));
    }
    if ranks.iter().collect::<BTreeSet<_>>().len() != ranks.len() {
        return Err(Error::Invalid("duplicate ranks".into()));
    }
    let size = inputs[0].len();
    if size > 2 * 1024 * 1024
        || inputs
            .iter()
            .any(|v| v.len() != size || v.iter().any(|x| !x.is_finite()))
    {
        return Err(Error::Invalid(
            "collective needs equal finite buffers <= 16 MiB per rank".into(),
        ));
    }
    if ranks
        .len()
        .checked_mul(size)
        .is_none_or(|v| v > 8 * 1024 * 1024)
    {
        return Err(Error::Invalid(
            "collective aggregate buffer exceeds 64 MiB".into(),
        ));
    }
    for rank in ranks {
        if sim.node(rank)?.spec.role != Role::Host {
            return Err(Error::Invalid("collective ranks must be hosts".into()));
        }
    }
    Ok(size)
}
impl Simulation {
    pub fn all_reduce(
        &mut self,
        ranks: &[String],
        inputs: &[Vec<f64>],
        op: Reduction,
    ) -> Result<CollectiveResult> {
        let count = validate(self, ranks, inputs)?;
        let n = ranks.len();
        let started = self.clock_ns;
        let mut staged = self.clone();
        let mut data = inputs.to_vec();
        let mut traces = Vec::new();
        let chunks: Vec<_> = (0..n)
            .map(|i| (count * i / n, count * (i + 1) / n))
            .collect();
        let mut round_time = started;
        // Each round snapshots all sends before applying receives, preserving concurrency.
        for round in 0..n - 1 {
            let previous = data.clone();
            let mut end = round_time;
            for rank in 0..n {
                let chunk = (rank + n - round - 1) % n;
                let (begin, finish) = chunks[chunk];
                let next = (rank + 1) % n;
                if begin == finish {
                    continue;
                }
                let transfer = staged.schedule_transfer(
                    &ranks[rank],
                    &ranks[next],
                    (finish - begin) * 8,
                    round_time,
                )?;
                end = end.max(transfer.completed_ns);
                traces.push(transfer);
                for i in begin..finish {
                    data[next][i] = reduce(previous[next][i], previous[rank][i], op);
                    if !data[next][i].is_finite() {
                        return Err(Error::Invalid("reduction overflow".into()));
                    }
                }
            }
            round_time = end;
        }
        if matches!(op, Reduction::Average) {
            for rank in 0..n {
                let (a, b) = chunks[rank];
                for element in &mut data[rank][a..b] {
                    *element /= n as f64;
                }
            }
        }
        for round in 0..n - 1 {
            let previous = data.clone();
            let mut end = round_time;
            for rank in 0..n {
                let chunk = (rank + n - round) % n;
                let (begin, finish) = chunks[chunk];
                let next = (rank + 1) % n;
                if begin == finish {
                    continue;
                }
                let transfer = staged.schedule_transfer(
                    &ranks[rank],
                    &ranks[next],
                    (finish - begin) * 8,
                    round_time,
                )?;
                end = end.max(transfer.completed_ns);
                traces.push(transfer);
                data[next][begin..finish].copy_from_slice(&previous[rank][begin..finish]);
            }
            round_time = end;
        }
        staged.clock_ns = round_time;
        staged.event("ring all-reduce completed");
        *self = staged;
        Ok(CollectiveResult {
            outputs: data,
            transfers: traces,
            started_ns: started,
            completed_ns: round_time,
        })
    }
    pub fn broadcast(
        &mut self,
        ranks: &[String],
        root: usize,
        input: &[f64],
    ) -> Result<CollectiveResult> {
        if root >= ranks.len() {
            return Err(Error::Invalid("root rank is out of range".into()));
        }
        if ranks.len() > 4096 || input.len() > 2 * 1024 * 1024
            || ranks.len().checked_mul(input.len()).is_none_or(|v|v>8*1024*1024) {
            return Err(Error::Invalid("broadcast exceeds rank or memory limits".into()));
        }
        let inputs = vec![input.to_vec(); ranks.len()];
        validate(self, ranks, &inputs)?;
        let mut staged = self.clone();
        let start = self.clock_ns;
        let mut end = start;
        let mut transfers = Vec::new();
        for (rank, node) in ranks.iter().enumerate() {
            if rank != root {
                let t = staged.schedule_transfer(&ranks[root], node, input.len() * 8, start)?;
                end = end.max(t.completed_ns);
                transfers.push(t);
            }
        }
        staged.clock_ns = end;
        staged.event("broadcast completed");
        *self = staged;
        Ok(CollectiveResult {
            outputs: inputs,
            transfers,
            started_ns: start,
            completed_ns: end,
        })
    }
    pub fn all_gather(
        &mut self,
        ranks: &[String],
        inputs: &[Vec<f64>],
    ) -> Result<CollectiveResult> {
        let count = validate(self, ranks, inputs)?;
        if ranks
            .len()
            .checked_mul(ranks.len())
            .and_then(|v| v.checked_mul(count))
            .is_none_or(|v| v > 8 * 1024 * 1024)
        {
            return Err(Error::Invalid("all-gather output exceeds 64 MiB".into()));
        }
        let mut staged = self.clone();
        let start = self.clock_ns;
        let mut time = start;
        let mut transfers = Vec::new();
        for _round in 0..ranks.len() - 1 {
            let mut end = time;
            for rank in 0..ranks.len() {
                let t = staged.schedule_transfer(
                    &ranks[rank],
                    &ranks[(rank + 1) % ranks.len()],
                    count * 8,
                    time,
                )?;
                end = end.max(t.completed_ns);
                transfers.push(t);
            }
            time = end;
        }
        let gathered: Vec<_> = inputs.iter().flatten().copied().collect();
        staged.clock_ns = time;
        staged.event("all-gather completed");
        *self = staged;
        Ok(CollectiveResult {
            outputs: vec![gathered; ranks.len()],
            transfers,
            started_ns: start,
            completed_ns: time,
        })
    }
    pub fn reduce_scatter(
        &mut self,
        ranks: &[String],
        inputs: &[Vec<f64>],
        op: Reduction,
    ) -> Result<CollectiveResult> {
        let count = validate(self, ranks, inputs)?;
        if count % ranks.len() != 0 {
            return Err(Error::Invalid(
                "reduce-scatter input must divide equally across ranks".into(),
            ));
        }
        // This implementation deliberately exposes the full all-reduce traffic cost:
        // a correct semantic composition, not a claim of NCCL's optimized schedule.
        let mut result = self.all_reduce(ranks, inputs, op)?;
        let chunk = count / ranks.len();
        result.outputs = result
            .outputs
            .into_iter()
            .enumerate()
            .map(|(i, v)| v[i * chunk..(i + 1) * chunk].to_vec())
            .collect();
        Ok(result)
    }
}
