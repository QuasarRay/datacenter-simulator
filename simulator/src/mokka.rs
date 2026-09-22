// SPDX-License-Identifier: RPL-1.5
//! Scoped deployment of NVIDIA Mokka for Kubernetes GPU software contracts.
//! Real NCCL and RDMA execution are never delegated to this backend.
use crate::{Simulator, manifest::Manifest, model::Role};
use anyhow::{Context, Result, bail, ensure};
use nix::{
    sys::signal::{Signal, killpg},
    unistd::Pid,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

pub const REVISION: &str = "4a44f73b41abc041f93d9b745677fa3210f65a84";

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NodeBinding {
    pub kubernetes_node: String,
    pub profile: String,
    pub gpu_count: usize,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MokkaConfig {
    pub manifest: PathBuf,
    pub upstream: PathBuf,
    pub context: String,
    pub namespace: String,
    /// Build this tag from the pinned upstream before applying the plan.
    pub image: String,
    /// Empty uses the three intended-state labels emitted by the NetBox importer.
    #[serde(default)]
    pub nodes: BTreeMap<String, NodeBinding>,
}
#[derive(Serialize)]
pub struct Release {
    pub name: String,
    pub model_node: String,
    pub kubernetes_node: String,
    pub expected_gpus: usize,
    pub values: Value,
}
#[derive(Serialize)]
pub struct MokkaPlan {
    pub scope: &'static str,
    pub upstream_revision: &'static str,
    pub nccl_tested: bool,
    pub rdma_payload_tested: bool,
    pub context: String,
    pub namespace: String,
    pub releases: Vec<Release>,
}
fn dns(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 63
        && !name.starts_with('-')
        && !name.ends_with('-')
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
impl MokkaConfig {
    pub fn read(path: &Path) -> Result<Self> {
        let mut config: Self = serde_json::from_slice(&fs::read(path)?)?;
        let base = path.parent().unwrap_or(Path::new("."));
        if config.manifest.is_relative() {
            config.manifest = base.join(&config.manifest);
        }
        if config.upstream.is_relative() {
            config.upstream = base.join(&config.upstream);
        }
        Ok(config)
    }
    pub fn plan(&self) -> Result<MokkaPlan> {
        ensure!(
            dns(&self.namespace)
                && self.namespace != "default"
                && !self.namespace.starts_with("kube-"),
            "use a dedicated Mokka namespace"
        );
        ensure!(
            !self.context.is_empty() && !self.context.contains(['\n', '\r']),
            "explicit Kubernetes context required"
        );
        let (repository, tag) = self
            .image
            .rsplit_once(':')
            .context("Mokka image needs an explicit source revision tag")?;
        ensure!(
            !repository.is_empty() && tag == format!("simulator-{}", &REVISION[..12]),
            "build and tag the pinned Mokka source as simulator-{}",
            &REVISION[..12]
        );
        let manifest = Manifest::read(&self.manifest)?;
        Simulator::new().import(manifest.clone(), false)?;
        let mut bindings = self.nodes.clone();
        if bindings.is_empty() {
            for (name, node) in &manifest.content.nodes {
                if let Some(profile) = node.labels.get("mokka.profile") {
                    bindings.insert(
                        name.clone(),
                        NodeBinding {
                            profile: profile
                                .as_str()
                                .context("mokka.profile must be a string")?
                                .into(),
                            gpu_count: usize::try_from(
                                node.labels
                                    .get("mokka.gpu_count")
                                    .and_then(Value::as_u64)
                                    .context("mokka.gpu_count must be set in intended state")?,
                            )?,
                            kubernetes_node: node
                                .labels
                                .get("kubernetes.node")
                                .and_then(Value::as_str)
                                .context("kubernetes.node must be set in intended state")?
                                .into(),
                        },
                    );
                }
            }
        }
        ensure!(!bindings.is_empty(), "no Mokka GPU hosts selected");
        let mut kube_nodes = BTreeSet::new();
        let mut releases = Vec::new();
        for (index, (name, binding)) in bindings.iter().enumerate() {
            let node = manifest
                .content
                .nodes
                .get(name)
                .context("Mokka binding references unknown model node")?;
            ensure!(
                node.role == Role::Host,
                "Mokka cannot turn switches into GPU hosts"
            );
            ensure!(
                kube_nodes.insert(&binding.kubernetes_node),
                "two model nodes map to the same Kubernetes node"
            );
            ensure!(
                binding.kubernetes_node.len() <= 253 && binding.kubernetes_node.split('.').all(dns),
                "invalid Kubernetes node name"
            );
            ensure!(dns(&binding.profile), "invalid Mokka profile name");
            let profile_path = self
                .upstream
                .join("deployments/nvml-mock/helm/nvml-mock/profiles")
                .join(format!("{}.yaml", binding.profile));
            let mut profile: Value = serde_yaml::from_str(&fs::read_to_string(profile_path)?)?;
            let available = profile["devices"]
                .as_array()
                .context("upstream profile has no devices")?
                .len();
            ensure!(
                binding.gpu_count > 0 && binding.gpu_count <= available,
                "GPU count exceeds the upstream profile; Mokka would silently cap it"
            );
            // ibsim is the only IB management backend in this integration. Do not
            // enable Mokka's independent fake IB/verbs fabric alongside it.
            profile["infiniband"] = json!({"enabled":false});
            let values = json!({
                "image":{"repository":repository,"tag":tag,"pullPolicy":"IfNotPresent"},
                "gpu":{"profile":binding.profile,"count":binding.gpu_count,"customConfig":serde_json::to_string(&profile)?},
                "nodeSelector":{"kubernetes.io/hostname":binding.kubernetes_node,"simulator.quasarray.io/mokka":"true"},
                "tolerations":[],"infiniband":{"mockTier":"off"},"nri":{"enabled":false},
                "nodeAgent":{"kernelLog":{"enabled":false}},"allocationWatcher":{"enabled":false},
                "topology":{"enabled":false},"imex":{"mockChannels":{"enabled":false}}
            });
            releases.push(Release {
                name: format!("sim-gpu-{index}"),
                model_node: name.clone(),
                kubernetes_node: binding.kubernetes_node.clone(),
                expected_gpus: binding.gpu_count,
                values,
            });
        }
        Ok(MokkaPlan {
            scope: "kubernetes-gpu-contracts",
            upstream_revision: REVISION,
            nccl_tested: false,
            rdma_payload_tested: false,
            context: self.context.clone(),
            namespace: self.namespace.clone(),
            releases,
        })
    }
}

pub fn render(config: &MokkaConfig, directory: &Path) -> Result<MokkaPlan> {
    let plan = config.plan()?;
    fs::create_dir(directory).context("Mokka output directory must be new")?;
    for release in &plan.releases {
        fs::write(
            directory.join(format!("{}.values.json", release.name)),
            serde_json::to_vec_pretty(&release.values)?,
        )?;
    }
    fs::write(
        directory.join("plan.json"),
        serde_json::to_vec_pretty(&plan)?,
    )?;
    Ok(plan)
}

struct Process(Child);
impl Drop for Process {
    fn drop(&mut self) {
        // Reap normally completed commands before considering their process group.
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = killpg(Pid::from_raw(self.0.id() as i32), Signal::SIGKILL);
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
fn execute(mut cmd: Command, directory: &Path, label: &str, seconds: u64) -> Result<Vec<u8>> {
    let stdout = directory.join(format!("{label}.stdout.log"));
    let stderr = directory.join(format!("{label}.stderr.log"));
    cmd.stdin(Stdio::null())
        .stdout(File::create(&stdout)?)
        .stderr(File::create(&stderr)?)
        .process_group(0);
    for name in [
        "LD_PRELOAD",
        "MOCK_NVML_CONFIG",
        "MOCK_IB",
        "SIM_HOST",
        "IBSIM_SOCKNAME",
    ] {
        cmd.env_remove(name);
    }
    let mut child = Process(cmd.spawn().with_context(|| format!("spawn {label}"))?);
    let end = Instant::now() + Duration::from_secs(seconds);
    let status = loop {
        if let Some(status) = child.0.try_wait()? {
            break status;
        }
        if Instant::now() >= end {
            bail!("{label} timed out; inspect {}", stderr.display());
        }
        std::thread::sleep(Duration::from_millis(100));
    };
    ensure!(
        status.success(),
        "{label} failed ({status}); inspect {}",
        stderr.display()
    );
    ensure!(
        fs::metadata(&stdout)?.len() <= 16 * 1024 * 1024,
        "{label} output exceeds 16 MiB"
    );
    Ok(fs::read(stdout)?)
}
fn kubectl(config: &MokkaConfig) -> Command {
    let mut cmd = Command::new("kubectl");
    cmd.arg(format!("--context={}", config.context));
    cmd
}

/// Apply to explicitly selected CPU lab nodes. No cluster is inferred or created.
pub fn apply(config: &MokkaConfig, directory: &Path) -> Result<()> {
    let plan = render(config, directory)?;
    let mut git = Command::new("git");
    git.arg("-C")
        .arg(&config.upstream)
        .args(["rev-parse", "HEAD"]);
    ensure!(
        String::from_utf8(execute(git, directory, "upstream-revision", 15)?)?.trim() == REVISION,
        "Mokka checkout does not match pinned source"
    );
    let mut clean = Command::new("git");
    clean.arg("-C").arg(&config.upstream).args([
        "diff",
        "--exit-code",
        "HEAD",
        "--",
        "deployments/nvml-mock/helm/nvml-mock",
    ]);
    execute(clean, directory, "upstream-chart-clean", 15)?;
    // Validate all target nodes before the first Helm mutation.
    for (index, release) in plan.releases.iter().enumerate() {
        let mut get = kubectl(config);
        get.args(["get", "node", &release.kubernetes_node, "-o", "json"]);
        let node: Value =
            serde_json::from_slice(&execute(get, directory, &format!("node-{index}"), 30)?)?;
        ensure!(
            node["metadata"]["name"] == release.kubernetes_node,
            "unexpected Kubernetes node"
        );
        ensure!(
            node["metadata"]["labels"]["simulator.quasarray.io/mokka"] == "true",
            "target node must carry simulator.quasarray.io/mokka=true"
        );
        ensure!(
            node["metadata"]["labels"]["kubernetes.io/hostname"] == release.kubernetes_node,
            "hostname selector must identify the exact selected node"
        );
        ensure!(
            node["status"]["conditions"].as_array().is_some_and(|c| c
                .iter()
                .any(|v| v["type"] == "Ready" && v["status"] == "True")),
            "Kubernetes node is not Ready"
        );
        ensure!(
            node["status"]["capacity"]["nvidia.com/gpu"]
                .as_str()
                .is_none_or(|v| v == "0"),
            "refusing to overlay a node already advertising GPUs"
        );
    }
    let chart = config.upstream.join("deployments/nvml-mock/helm/nvml-mock");
    let mut observations = Vec::new();
    for (index, release) in plan.releases.iter().enumerate() {
        let mut helm = Command::new("helm");
        helm.args(["upgrade", "--install", &release.name])
            .arg(&chart)
            .arg(format!("--kube-context={}", config.context))
            .args([
                "--namespace",
                &config.namespace,
                "--create-namespace",
                "--wait",
                "--atomic",
                "--timeout",
                "5m",
                "-f",
            ])
            .arg(directory.join(format!("{}.values.json", release.name)));
        execute(helm, directory, &format!("helm-{index}"), 360)?;
        let mut pods = kubectl(config);
        pods.args([
            "-n",
            &config.namespace,
            "get",
            "pods",
            "-l",
            &format!("app.kubernetes.io/instance={}", release.name),
            "-o",
            "json",
        ]);
        let list: Value =
            serde_json::from_slice(&execute(pods, directory, &format!("pods-{index}"), 30)?)?;
        let items = list["items"]
            .as_array()
            .context("Kubernetes pod list is absent")?;
        ensure!(
            items.len() == 1 && items[0]["spec"]["nodeName"] == release.kubernetes_node,
            "Mokka release did not resolve to exactly its intended node"
        );
        let name = items[0]["metadata"]["name"]
            .as_str()
            .context("missing pod name")?;
        let mut probe = kubectl(config);
        probe.args([
            "-n",
            &config.namespace,
            "exec",
            name,
            "-c",
            "node-agent",
            "--",
            "nvidia-smi",
            "--query-gpu=uuid,name",
            "--format=csv,noheader",
        ]);
        let inventory =
            String::from_utf8(execute(probe, directory, &format!("nvml-{index}"), 45)?)?;
        let lines: Vec<_> = inventory.lines().filter(|l| !l.trim().is_empty()).collect();
        ensure!(
            lines.len() == release.expected_gpus && lines.iter().all(|l| l.starts_with("GPU-")),
            "observed mock GPU inventory differs from intended state"
        );
        observations.push(json!({"node":release.kubernetes_node,"gpu_inventory":lines}));
    }
    fs::write(
        directory.join("result.json"),
        serde_json::to_vec_pretty(
            &json!({"ok":true,"scope":plan.scope,"upstream_revision":REVISION,"nccl_tested":false,"rdma_payload_tested":false,"observations":observations}),
        )?,
    )?;
    Ok(())
}
