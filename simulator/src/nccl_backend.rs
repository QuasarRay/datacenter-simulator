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

//! Supervised real NCCL processes, communicating through the patchbay fabric.
use crate::{
    collective::{Collective, CollectiveResult, NcclOptions, RankTraffic},
    linux::{LinuxFabric, NCCL_INTERFACE},
    model::{Error, Simulation},
    nccl_worker::{Finished, MESSAGE_LIMIT, Ready, Start},
};
use anyhow::{Context, Result, bail, ensure};
use serde::de::DeserializeOwned;
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdout, Command},
};

/// CLI defaults to its own worker entry point. Embedders point this at the built
/// datacenter-simulator executable; no Python interpreter or shell is involved.
pub fn worker_executable() -> Result<PathBuf> {
    match std::env::var_os("DATACENTER_SIMULATOR_WORKER") {
        Some(path) => Ok(PathBuf::from(path)),
        None => Ok(std::env::current_exe()?),
    }
}
pub fn execute(
    sim: &Simulation,
    ranks: &[String],
    request: &Collective,
    options: &NcclOptions,
) -> crate::Result<CollectiveResult> {
    if cfg!(feature = "nccl-check") {
        return Err(Error::Unsupported("nccl-check is compilation only".into()));
    }
    if tokio::runtime::Handle::try_current().is_ok() {
        return Err(Error::State(
            "use LinuxFabric::collective().await inside an async runtime".into(),
        ));
    }
    let result = (|| -> Result<CollectiveResult> {
        let worker = worker_executable()?;
        patchbay::init_userns()?;
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(async {
                let fabric = LinuxFabric::build(sim).await?;
                fabric.collective(ranks, request, options, &worker).await
            })
    })();
    result.map_err(|e| Error::State(format!("native NCCL execution failed: {e:#}")))
}

/// The worker environment is private to each child. Host process globals are not changed.
fn worker_command(executable: &Path, rank: usize, count: usize, device: i32) -> Command {
    let mut cmd = Command::new(executable);
    cmd.args([
        "__nccl_rank",
        &rank.to_string(),
        &count.to_string(),
        &device.to_string(),
    ]);
    // Prevent inherited NCCL tuning from enabling a transport outside the lab.
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("NCCL_") {
            cmd.env_remove(key);
        }
    }
    cmd.envs([
        ("NCCL_NET", "Socket"),
        ("NCCL_NET_PLUGIN", "none"),
        ("NCCL_SOCKET_IFNAME", "=simnccl"),
        ("NCCL_SOCKET_FAMILY", "AF_INET"),
        ("NCCL_P2P_DISABLE", "1"),
        ("NCCL_SHM_DISABLE", "1"),
        ("NCCL_NVLS_ENABLE", "0"),
        ("NCCL_COLLNET_ENABLE", "0"),
        ("NCCL_MNNVL_ENABLE", "0"),
        ("NCCL_IB_DISABLE", "1"),
        ("NCCL_ALGO", "Ring,Tree"),
        ("NCCL_CONF_FILE", "/dev/null"),
        ("NCCL_DEBUG", "INFO"),
        ("NCCL_DEBUG_SUBSYS", "INIT,NET"),
        ("NCCL_DEBUG_FILE", "/dev/stderr"),
    ]);
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .kill_on_drop(true);
    cmd
}
async fn message<T: DeserializeOwned>(reader: &mut BufReader<ChildStdout>) -> Result<T> {
    let mut line = String::new();
    reader.take(MESSAGE_LIMIT + 1).read_line(&mut line).await?;
    ensure!(
        line.ends_with('\n') && line.len() as u64 <= MESSAGE_LIMIT,
        "NCCL worker exited or sent an invalid response; inspect its stderr"
    );
    serde_json::from_str(&line).context("decode NCCL worker response")
}
impl LinuxFabric {
    /// Every successful output comes from ncclAllReduce/Broadcast/AllGather/ReduceScatter
    /// and a synchronized CUDA device-to-host copy. Errors never return modeled values.
    pub async fn collective(
        &self,
        ranks: &[String],
        request: &Collective,
        options: &NcclOptions,
        worker: &Path,
    ) -> Result<CollectiveResult> {
        ensure!(
            !cfg!(feature = "nccl-check"),
            "nccl-check cannot execute NCCL"
        );
        self.ensure_healthy()?;
        request.validate(&self.simulation, ranks)?;
        let devices = options.devices_for(ranks.len())?;
        let began = Instant::now();
        let mut children: Vec<Child> = Vec::new();
        let run = async {
            let mut before = Vec::new();
            let mut readers = Vec::new();
            for (rank, node) in ranks.iter().enumerate() {
                before.push(self.data_counters(node)?);
                let mut child = self.device(node)?.spawn_command(worker_command(
                    worker,
                    rank,
                    ranks.len(),
                    devices[rank],
                ))?;
                readers.push(BufReader::new(
                    child.stdout.take().context("NCCL worker stdout")?,
                ));
                children.push(child);
            }
            let mut id = None;
            let mut version = None;
            // All CUDA devices must initialize before any rank enters blocking NCCL init.
            for (rank, reader) in readers.iter_mut().enumerate() {
                let ready: Ready = message(reader)
                    .await
                    .with_context(|| format!("initialize NCCL rank {rank}"))?;
                if rank == 0 {
                    id = Some(
                        ready
                            .unique_id
                            .context("rank zero did not create a bootstrap ID")?,
                    );
                    version = Some(ready.nccl_version);
                } else {
                    ensure!(
                        ready.unique_id.is_none() && Some(ready.nccl_version) == version,
                        "NCCL worker version/protocol mismatch"
                    );
                }
            }
            let id = id.context("missing bootstrap ID")?;
            for (rank, child) in children.iter_mut().enumerate() {
                let start = Start::for_rank(request, ranks.len(), rank, &id)?;
                let mut payload = serde_json::to_vec(&start)?;
                payload.push(b'\n');
                ensure!(
                    payload.len() as u64 <= MESSAGE_LIMIT,
                    "NCCL worker request too large"
                );
                let mut input = child.stdin.take().context("NCCL worker stdin")?;
                input.write_all(&payload).await?;
                input.shutdown().await?;
            }
            let (_, output_count) = request.shape(ranks.len())?;
            let mut outputs = Vec::new();
            let mut rank_elapsed_ns = Vec::new();
            for (rank, reader) in readers.iter_mut().enumerate() {
                let finished: Finished = message(reader)
                    .await
                    .with_context(|| format!("execute NCCL rank {rank}"))?;
                ensure!(
                    finished.output.len() == output_count
                        && finished.output.iter().all(|v| v.is_finite()),
                    "invalid NCCL rank {rank} output"
                );
                outputs.push(finished.output);
                rank_elapsed_ns.push(finished.elapsed_ns);
            }
            for (rank, child) in children.iter_mut().enumerate() {
                ensure!(
                    child.wait().await?.success(),
                    "NCCL rank {rank} exited unsuccessfully"
                );
            }
            let mut network = Vec::new();
            for (rank, node) in ranks.iter().enumerate() {
                let (tx, rx) = self.data_counters(node)?;
                network.push(RankTraffic {
                    node: node.clone(),
                    tx_bytes: tx.checked_sub(before[rank].0).context("TX counter reset")?,
                    rx_bytes: rx.checked_sub(before[rank].1).context("RX counter reset")?,
                });
            }
            Ok(CollectiveResult {
                backend: "nccl".into(),
                transport: "Socket".into(),
                nccl_version: version.context("missing NCCL version")?,
                outputs,
                elapsed_ns: began.elapsed().as_nanos().try_into()?,
                rank_elapsed_ns,
                network_accounting: "interface-counters-including-concurrent-traffic".into(),
                network,
            })
        };
        let result = tokio::time::timeout(Duration::from_secs(options.timeout_secs), run).await;
        // On timeout, startup failure, broken pipes or rank failure, terminate ALL
        // remaining workers before the caller can tear down namespaces or reuse GPUs.
        if !matches!(&result, Ok(Ok(_))) {
            for child in &mut children {
                let _ = child.start_kill();
            }
            let cleanup = async {
                for child in &mut children {
                    let _ = child.wait().await;
                }
            };
            if tokio::time::timeout(Duration::from_secs(2), cleanup)
                .await
                .is_err()
            {
                let unreaped: Vec<_> = children
                    .iter_mut()
                    .filter_map(|c| {
                        if c.try_wait().ok().flatten().is_none() {
                            c.id()
                        } else {
                            None
                        }
                    })
                    .collect();
                bail!(
                    "NCCL failed; cleanup grace exceeded 2 seconds; unreaped PIDs: {unreaped:?}; GPUs must not be reused until these exit"
                );
            }
        }
        match result {
            Ok(result) => result,
            Err(_) => bail!(
                "NCCL execution exceeded {} seconds; all ranks terminated",
                options.timeout_secs
            ),
        }
    }
    fn data_counters(&self, node: &str) -> Result<(u64, u64)> {
        let names: Vec<_> = self
            .simulation
            .interfaces()
            .filter(|i| i.node == node)
            .map(|i| i.name.clone())
            .collect();
        self.device(node)?.run_sync(move || {
            let output = std::process::Command::new("ip")
                .args(["-j", "-s", "link", "show"])
                .output()?;
            ensure!(
                output.status.success(),
                "cannot read kernel interface counters"
            );
            let links: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout)?;
            let (mut tx, mut rx) = (0u64, 0u64);
            for link in links {
                let name = link["ifname"].as_str().context("missing interface name")?;
                if name == NCCL_INTERFACE || !names.iter().any(|n| n == name) {
                    continue;
                }
                let stats = &link["stats64"];
                tx = tx
                    .checked_add(
                        stats["tx"]["bytes"]
                            .as_u64()
                            .context("missing TX counter")?,
                    )
                    .context("TX overflow")?;
                rx = rx
                    .checked_add(
                        stats["rx"]["bytes"]
                            .as_u64()
                            .context("missing RX counter")?,
                    )
                    .context("RX overflow")?;
            }
            Ok((tx, rx))
        })
    }
}
