// SPDX-License-Identifier: RPL-1.5
//! Scoped deployment of NVIDIA Mokka for Kubernetes GPU software contracts.
//! Real NCCL and RDMA execution are never delegated to this backend.
use crate::{Simulator, manifest::Manifest, model::Role};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
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
    pub image_pinned: bool,
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
        let mut config: Self =
            serde_json::from_slice(&crate::input::read(path, crate::input::CONFIG_LIMIT)?)?;
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
        crate::provenance::verify(
            &self.upstream,
            REVISION,
            "deployments/nvml-mock/helm/nvml-mock",
        )?;
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
        let (reference, digest) = self
            .image
            .split_once('@')
            .map_or((self.image.as_str(), None), |(r, d)| (r, Some(d)));
        if let Some(digest) = digest {
            ensure!(valid_digest(digest), "image must use a SHA-256 OCI digest");
        }
        let (repository, source_tag) = reference
            .rsplit_once(':')
            .context("Mokka image needs an explicit source revision tag")?;
        ensure!(
            !repository.is_empty() && source_tag == format!("simulator-{}", &REVISION[..12]),
            "Mokka image tag must identify the pinned source revision"
        );
        let tag = digest.map_or_else(|| source_tag.to_string(), |d| format!("{source_tag}@{d}"));
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
            // The agent reads GPU_COUNT, but exec'd NVML consumers read the
            // ConfigMap directly. Set the upstream engine's count in both.
            ensure!(
                profile["system"].is_object(),
                "upstream profile has no system configuration"
            );
            profile["system"]["num_devices"] = json!(binding.gpu_count);
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
            image_pinned: digest.is_some(),
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

struct Process {
    child: Child,
    group: Option<u32>,
}
impl Process {
    fn try_wait(&mut self) -> std::io::Result<Option<std::process::ExitStatus>> {
        if let Some(pid) = self.group {
            let exited = match crate::process_group::exited(pid) {
                Ok(exited) => exited,
                Err(error) => {
                    if error.raw_os_error() == Some(nix::errno::Errno::ECHILD as i32) {
                        self.group = None;
                    }
                    return Err(error);
                }
            };
            if !exited {
                return Ok(None);
            }
            // Leader is still a zombie and owns this PGID. Kill descendants
            // before reaping, including those still holding log pipe writers.
            crate::process_group::terminate(pid);
        }
        let result = self.child.try_wait();
        if matches!(result, Ok(Some(_))) {
            self.group = None;
        }
        result
    }
}
impl Drop for Process {
    fn drop(&mut self) {
        if let Some(pid) = self.group.take() {
            crate::process_group::terminate(pid);
            let _ = self.child.kill();
            let deadline = Instant::now() + Duration::from_secs(2);
            while self.child.try_wait().ok().flatten().is_none() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(10));
            }
            if self.child.try_wait().ok().flatten().is_none() {
                eprintln!("Mokka cleanup left unreaped PID {}", self.child.id());
            }
        }
    }
}
fn execute(mut cmd: Command, directory: &Path, label: &str, seconds: u64) -> Result<Vec<u8>> {
    let program = Path::new(cmd.get_program());
    let resolved = crate::evidence::resolve(program)?;
    let identity = crate::evidence::FileIdentity::read(&resolved)?;
    let identities: BTreeMap<String, crate::evidence::FileIdentity> = serde_json::from_slice(
        &crate::input::read(directory.join("tools.json"), crate::input::CONFIG_LIMIT)?,
    )?;
    ensure!(
        identities.values().any(|expected| *expected == identity),
        "external tool identity changed"
    );
    // Rebuild using the recorded absolute executable path, preserving arguments.
    let mut pinned = Command::new(&resolved);
    pinned.args(cmd.get_args());
    cmd = pinned;
    let stdout = directory.join(format!("{label}.stdout.log"));
    let stderr = directory.join(format!("{label}.stderr.log"));
    let (out_pipe, out_done) = crate::bounded_log::capture(&stdout)?;
    let (err_pipe, err_done) = crate::bounded_log::capture(&stderr)?;
    cmd.stdin(Stdio::null())
        .stdout(out_pipe)
        .stderr(err_pipe)
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
    let child = cmd.spawn().with_context(|| format!("spawn {label}"))?;
    let mut child = Process {
        group: Some(child.id()),
        child,
    };
    drop(cmd);
    let end = Instant::now() + Duration::from_secs(seconds);
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if Instant::now() >= end {
            bail!("{label} timed out; inspect {}", stderr.display());
        }
        std::thread::sleep(Duration::from_millis(100));
    };
    out_done.finish()?;
    err_done.finish()?;
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
    ensure!(
        config
            .image
            .split_once('@')
            .is_some_and(|(_, d)| valid_digest(d)),
        "mokka-apply requires a source tag plus @sha256: OCI digest; a mutable tag is not provenance"
    );
    let plan = render(config, directory)?;
    let identities = crate::evidence::tools(&[Path::new("kubectl"), Path::new("helm")])?;
    fs::write(
        directory.join("tools.json"),
        serde_json::to_vec_pretty(&identities)?,
    )?;
    // Namespace lifecycle belongs to the operator. Never create it implicitly:
    // a Helm rollback cannot prove a namespace contains only this run's objects.
    let mut namespace = kubectl(config);
    namespace.args(["get", "namespace", &config.namespace, "-o", "json"]);
    let namespace: Value =
        serde_json::from_slice(&execute(namespace, directory, "namespace", 30)?)?;
    ensure!(
        namespace["metadata"]["name"] == config.namespace
            && namespace["status"]["phase"] == "Active",
        "Mokka requires an existing Active namespace"
    );
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
            has_no_gpu_capacity(&node),
            "refusing to overlay a node already advertising GPUs"
        );
    }
    let mut list = Command::new("helm");
    list.args([
        "list",
        "--all",
        "--output",
        "json",
        "--namespace",
        &config.namespace,
    ])
    .arg(format!("--kube-context={}", config.context));
    let existing: Vec<Value> =
        serde_json::from_slice(&execute(list, directory, "existing-releases", 30)?)?;
    ensure!(
        plan.releases
            .iter()
            .all(|r| !existing.iter().any(|e| e["name"] == r.name)),
        "Mokka batch requires new release names; existing releases are never overwritten"
    );
    let mut batch = Batch {
        config,
        directory,
        releases: Vec::new(),
        armed: true,
        owner: uuid::Uuid::new_v4().to_string(),
    };
    let chart = config.upstream.join("deployments/nvml-mock/helm/nvml-mock");
    let mut observations = Vec::new();
    for (index, release) in plan.releases.iter().enumerate() {
        let mut helm = Command::new("helm");
        batch.releases.push(release.name.clone());
        batch.record("applying", &[])?;
        helm.args(["install", &release.name])
            .args(["--labels", &format!("simulator-run={}", batch.owner)])
            .arg(&chart)
            .arg(format!("--kube-context={}", config.context))
            .args([
                "--namespace",
                &config.namespace,
                "--wait",
                "--atomic",
                "--timeout",
                "5m",
                "-f",
            ])
            .arg(directory.join(format!("{}.values.json", release.name)));
        execute(helm, directory, &format!("helm-{index}"), 360)?;
        // Helm can consider an OnDelete DaemonSet deployed before its new
        // containers exist. Wait for the actual pod before inspecting/probing it.
        let mut ready = kubectl(config);
        ready.args([
            "-n",
            &config.namespace,
            "wait",
            "--for=condition=Ready",
            "pod",
            "-l",
            &format!("app.kubernetes.io/instance={}", release.name),
            "--timeout=120s",
        ]);
        execute(ready, directory, &format!("ready-{index}"), 130)?;
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
        let agent = items[0]["spec"]["containers"]
            .as_array()
            .context("missing pod containers")?
            .iter()
            .find(|c| c["name"] == "node-agent")
            .context("missing Mokka node agent")?;
        ensure!(
            agent["image"] == config.image,
            "deployed image reference differs from the pinned plan"
        );
        let image_ids = items[0]["status"]["containerStatuses"].clone();
        ensure!(
            image_ids
                .as_array()
                .is_some_and(|statuses| statuses.iter().any(|s| {
                    s["name"] == "node-agent"
                        && s["ready"] == true
                        && s["imageID"].as_str().is_some_and(|id| !id.is_empty())
                })),
            "Mokka node agent has no ready running image identity"
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
            "observed mock GPU inventory differs from intended state: expected {}, observed {}; inspect {}",
            release.expected_gpus,
            lines.len(),
            directory.join(format!("nvml-{index}.stdout.log")).display()
        );
        observations.push(json!({"node":release.kubernetes_node,"gpu_inventory":lines,"container_images":image_ids}));
    }
    durable_json(
        directory.join("result.json"),
        serde_json::to_vec_pretty(
            &json!({"ok":true,"scope":plan.scope,"upstream_revision":REVISION,"nccl_tested":false,"rdma_payload_tested":false,"observations":observations}),
        )?,
    )?;
    batch.record("complete", &[])?;
    batch.armed = false;
    Ok(())
}

fn durable_json(path: impl AsRef<Path>, bytes: Vec<u8>) -> Result<()> {
    use std::io::Write;
    let path = path.as_ref();
    let temporary = path.with_extension(format!("json.{}.tmp", crate::id()));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    fs::rename(&temporary, path)?;
    fs::File::open(path.parent().context("report parent")?)?.sync_all()?;
    Ok(())
}

fn valid_digest(digest: &str) -> bool {
    digest.strip_prefix("sha256:").is_some_and(|h| {
        h.len() == 64
            && h.bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    })
}
struct Batch<'a> {
    config: &'a MokkaConfig,
    directory: &'a Path,
    releases: Vec<String>,
    armed: bool,
    owner: String,
}
impl Batch<'_> {
    fn record(&self, status: &str, errors: &[String]) -> Result<()> {
        durable_json(
            self.directory.join("batch-state.json"),
            serde_json::to_vec_pretty(&json!({
                "status":status,"owned_releases":self.releases,"owner":self.owner,"cleanup_errors":errors,
                "scope":"kubernetes-gpu-contracts","nccl_tested":false,"rdma_payload_tested":false
            }))?,
        )?;
        Ok(())
    }
}
impl Drop for Batch<'_> {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let _ = fs::remove_file(self.directory.join("result.json"));
        let mut errors = Vec::new();
        // Only uninstall releases bearing this run's ownership token, including
        // partial installs. A competing install between preflight and install is safe.
        let mut list = Command::new("helm");
        list.args([
            "list",
            "--all",
            "--output",
            "json",
            "--namespace",
            &self.config.namespace,
            "--selector",
            &format!("simulator-run={}", self.owner),
        ])
        .arg(format!("--kube-context={}", self.config.context));
        let owned = execute(list, self.directory, "rollback-owned", 30)
            .and_then(|bytes| Ok(serde_json::from_slice::<Vec<Value>>(&bytes)?));
        let owned = match owned {
            Ok(owned) => owned,
            Err(e) => {
                let _ = self.record(
                    "partial-cleanup-required",
                    &[format!("cannot verify release ownership: {e:#}")],
                );
                return;
            }
        };
        for (i, name) in self.releases.iter().enumerate().rev() {
            if !owned.iter().any(|r| r["name"] == *name) {
                continue;
            }
            let mut helm = Command::new("helm");
            helm.args([
                "uninstall",
                name,
                "--namespace",
                &self.config.namespace,
                "--ignore-not-found",
                "--wait",
                "--timeout",
                "30s",
            ])
            .arg(format!("--kube-context={}", self.config.context));
            if let Err(e) = execute(helm, self.directory, &format!("rollback-{i}"), 45) {
                errors.push(format!("{name}: {e:#}"));
            }
        }
        let status = if errors.is_empty() {
            "rolled-back"
        } else {
            "partial-cleanup-required"
        };
        if let Err(e) = self.record(status, &errors) {
            eprintln!("Mokka cleanup evidence: {e:#}");
        }
    }
}

fn has_no_gpu_capacity(node: &Value) -> bool {
    // Reject every vendor resource, including MIG and renamed time-sliced GPU
    // resources. Unknown vendor quantities fail closed. Bare CPU nodes need no
    // NVIDIA resources; the separate explicit opt-in remains mandatory.
    let safe = |field: &str| {
        node["status"][field].as_object().is_some_and(|resources| {
            resources.iter().all(|(name, value)| {
                !name.starts_with("nvidia.com/") || value.as_str() == Some("0")
            })
        })
    };
    safe("capacity") && (node["status"]["allocatable"].is_null() || safe("allocatable"))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gpu_capacity_fails_closed_on_wrong_types() {
        assert!(has_no_gpu_capacity(&json!({"status":{"capacity":{}}})));
        assert!(has_no_gpu_capacity(
            &json!({"status":{"capacity":{"nvidia.com/gpu":"0"}}})
        ));
        for value in [
            json!(8),
            json!(0),
            json!(null),
            json!(true),
            json!({}),
            json!([]),
            json!("1"),
            json!("unknown"),
        ] {
            assert!(!has_no_gpu_capacity(
                &json!({"status":{"capacity":{"nvidia.com/gpu":value}}})
            ));
        }
        assert!(!has_no_gpu_capacity(&json!({})));
    }
}

#[cfg(test)]
mod capacity_regressions {
    use super::*;
    #[test]
    fn gpu_resources_in_either_capacity_map_fail_closed() {
        for key in [
            "nvidia.com/gpu",
            "nvidia.com/mig-1g.5gb",
            "nvidia.com/gpu.shared",
            "nvidia.com/mig-1g.5gb.shared",
        ] {
            for field in ["capacity", "allocatable"] {
                let mut node = json!({"status":{"capacity":{},"allocatable":{}}});
                node["status"][field][key] = json!("1");
                assert!(!has_no_gpu_capacity(&node));
                node["status"][field][key] = json!("0");
                assert!(has_no_gpu_capacity(&node));
            }
        }
    }
}
