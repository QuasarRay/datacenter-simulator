// SPDX-License-Identifier: RPL-1.5
//! Incus system-container lifecycle over its Unix-socket REST API.
//! Intent comes from the existing validated petgraph-backed simulator model.
use crate::{
    Simulator,
    manifest::Manifest,
    model::{InterfaceType, Role},
};
use anyhow::{Context, Result, bail, ensure};
use patchbay::host_cable::{Endpoint, HostCable};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

/// Explicit local resources. Images must already be imported into Incus.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub project: String,
    pub socket: PathBuf,
    pub storage_pool: String,
    pub image_fingerprint: String,
    pub manifest: PathBuf,
    pub max_memory_mib: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodePlan {
    pub name: String,
    pub spec: Value,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CablePlan {
    pub name: String,
    pub peers: [String; 2],
    pub latency_ns: u64,
    pub bandwidth_bps: u64,
    pub queue_packets: u32,
    pub up: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plan {
    pub owner: String,
    pub nodes: Vec<NodePlan>,
    pub cables: Vec<CablePlan>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Journal {
    pub config: Config,
    pub plan: Plan,
    pub cables: Vec<HostCable>,
    pub phase: String,
}
fn name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 40
        && value.as_bytes()[0].is_ascii_lowercase()
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
impl Config {
    pub fn read(path: &Path) -> Result<Self> {
        let path = fs::canonicalize(path)?;
        let mut c: Self =
            serde_json::from_slice(&crate::input::read(&path, crate::input::CONFIG_LIMIT)?)?;
        if c.manifest.is_relative() {
            c.manifest = path.parent().context("config directory")?.join(&c.manifest);
        }
        Ok(c)
    }
    pub fn plan(&self) -> Result<Plan> {
        ensure!(
            name(&self.project) && self.project.starts_with("ncp-") && name(&self.storage_pool),
            "use an ncp- project and valid pool name"
        );
        ensure!(self.socket.is_absolute(), "socket must be absolute");
        ensure!(
            self.image_fingerprint.len() == 64
                && self
                    .image_fingerprint
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit()),
            "an exact local image fingerprint is required"
        );
        let mut api = Simulator::new();
        let id = api.import(Manifest::read(&self.manifest)?, false)?;
        let sim = api.get(&id)?;
        let owner = uuid::Uuid::new_v4().to_string();
        let prefix = owner.replace('-', "")[..8].to_string();
        let mut nodes: Vec<_> = sim.nodes().collect();
        nodes.sort_by_key(|n| &n.spec.name);
        ensure!(
            !nodes.is_empty() && nodes.len() <= 64,
            "Incus profile supports 1..64 real system containers; use KWOK for worker scale"
        );
        let total = nodes
            .iter()
            .try_fold(0u64, |sum, n| sum.checked_add(n.spec.resources.memory))
            .context("memory overflow")?;
        ensure!(
            total <= self.max_memory_mib,
            "declared memory exceeds local budget"
        );
        let mut peers = BTreeMap::new();
        let mut plans = Vec::new();
        for (index, node) in nodes.iter().enumerate() {
            ensure!(name(&node.spec.name), "invalid instance name");
            ensure!(
                node.spec.image == "cachyos",
                "Incus teaching profile requires os=cachyos and a user-supplied CachyOS image fingerprint"
            );
            ensure!(
                node.spec
                    .labels
                    .get("simulator.medium")
                    .and_then(Value::as_str)
                    != Some("infiniband"),
                "InfiniBand requires the existing ibsim backend; Ethernet host cables cannot impersonate it"
            );
            ensure!(
                node.user_data.is_none(),
                "implicit user_data execution is unsupported; use reviewed Ansible code"
            );
            let mut devices = serde_json::Map::new();
            devices.insert("root".into(), json!({"type":"disk","path":"/","pool":self.storage_pool,"size":format!("{}GiB",node.spec.resources.storage)}));
            let mut interfaces: Vec<_> = sim.interfaces().filter(|i| i.node == node.id).collect();
            interfaces.sort_by_key(|i| &i.name);
            ensure!(interfaces.len() <= 99, "too many ports");
            for (port, interface) in interfaces.iter().enumerate() {
                ensure!(
                    interface.interface_type == InterfaceType::Data
                        && interface.split_parent.is_none()
                        && interface.split_children.is_empty(),
                    "only explicit unsplit data ports are supported"
                );
                ensure!(
                    name(&interface.name) && interface.name.len() <= 15,
                    "invalid guest interface name"
                );
                let peer = format!("pi{prefix}{index:02}{port:02}");
                peers.insert(interface.id.clone(), peer.clone());
                devices.insert(interface.name.clone(), json!({"type":"nic","nictype":"p2p","name":interface.name,"host_name":peer,"hwaddr":interface.mac_address}));
            }
            plans.push(NodePlan { name: node.spec.name.clone(), spec: json!({
                "name":node.spec.name,"type":"container","profiles":[],
                "source":{"type":"image","fingerprint":self.image_fingerprint},
                "config":{"user.ncp.owner":owner,"user.ncp.role":if node.spec.role == Role::Switch {"switch"} else {"host"},
                          "security.privileged":"false","security.nesting":"false","limits.cpu":node.spec.resources.cpu.to_string(),"limits.memory":format!("{}MiB",node.spec.resources.memory)},
                "devices":devices
            }) });
        }
        let mut links: Vec<_> = sim.links().collect();
        links.sort_by_key(|l| l.interfaces.clone());
        ensure!(links.len() <= 999, "too many cables");
        let mut cables = Vec::new();
        for (index, link) in links.iter().enumerate() {
            cables.push(CablePlan {
                name: format!("pc{prefix}{index:03}"),
                peers: [
                    peers[&link.interfaces[0]].clone(),
                    peers[&link.interfaces[1]].clone(),
                ],
                latency_ns: link.spec.latency_ns,
                bandwidth_bps: link.spec.bandwidth_bps,
                queue_packets: link.spec.native_queue_packets()?,
                up: link.spec.up,
            });
        }
        Ok(Plan {
            owner,
            nodes: plans,
            cables,
        })
    }
}

/// Incus calls use argument vectors and stdin JSON. HTTP and operation failures
/// propagate. Async acceptance is never interpreted as successful completion.
pub struct Client<'a>(pub &'a Config);
impl Client<'_> {
    pub fn request(&self, method: &str, path: &str, body: Option<&Value>) -> Result<Value> {
        ensure!(
            path.starts_with("/1.0/") || path == "/1.0",
            "invalid API path"
        );
        ensure!(
            name(&self.0.project) && self.0.project.starts_with("ncp-"),
            "invalid project scope"
        );
        let sep = if path.contains('?') { '&' } else { '?' };
        let scoped = path.starts_with("/1.0/instances") || path.starts_with("/1.0/operations/");
        let url = if scoped {
            format!("http://localhost{path}{sep}project={}", self.0.project)
        } else {
            format!("http://localhost{path}")
        };
        let mut cmd = Command::new("curl");
        cmd.args([
            "--silent",
            "--show-error",
            "--fail-with-body",
            "--max-time",
            "40",
            "--max-filesize",
            "16777216",
            "--unix-socket",
        ])
        .arg(&self.0.socket)
        .args([
            "--request",
            method,
            "--header",
            "Content-Type: application/json",
            &url,
        ]);
        if body.is_some() {
            cmd.args(["--data-binary", "@-"]);
        }
        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        if let Some(body) = body {
            child
                .stdin
                .take()
                .context("curl stdin")?
                .write_all(&serde_json::to_vec(body)?)?;
        }
        let output = child.wait_with_output()?;
        ensure!(
            output.status.success(),
            "Incus HTTP request failed: {} {}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        );
        let result: Value = serde_json::from_slice(&output.stdout)?;
        ensure!(
            result["type"] != "error",
            "Incus API error: {}",
            result["error"]
        );
        Ok(result)
    }
    pub fn complete(&self, method: &str, path: &str, body: Option<&Value>) -> Result<Value> {
        let result = self.request(method, path, body)?;
        if result["type"] != "async" {
            return Ok(result["metadata"].clone());
        }
        let operation = result["operation"]
            .as_str()
            .context("missing operation")?
            .split('?')
            .next()
            .context("operation path")?;
        let id = operation
            .strip_prefix("/1.0/operations/")
            .context("unexpected operation URL")?;
        ensure!(
            uuid::Uuid::parse_str(id).is_ok(),
            "unexpected operation identifier"
        );
        for _ in 0..20 {
            let status = self.request("GET", &format!("{operation}/wait?timeout=30"), None)?;
            let code = status["metadata"]["status_code"]
                .as_u64()
                .context("missing operation status")?;
            if code == 200 {
                return Ok(status["metadata"].clone());
            }
            ensure!(
                code < 400,
                "Incus operation failed: {}",
                status["metadata"]["err"]
            );
        }
        bail!("Incus operation timed out; journal retained; reconcile before retry")
    }
}
fn save(path: &Path, journal: &Journal) -> Result<()> {
    let temp = path.with_extension("next");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)?;
    file.write_all(&serde_json::to_vec_pretty(journal)?)?;
    file.sync_all()?;
    fs::rename(temp, path)?;
    fs::File::open(path.parent().context("journal parent")?)?.sync_all()?;
    Ok(())
}
struct Lock(PathBuf);
impl Lock {
    fn acquire() -> Result<Self> {
        let path = PathBuf::from("/run/lock/ncp-incus-network.lock");
        let mut file = OpenOptions::new().write(true).create_new(true).open(&path)
            .context("another lifecycle operation or stale crash lock exists; inspect ownership before removing the lock")?;
        writeln!(file, "{}", std::process::id())?;
        Ok(Self(path))
    }
}
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

pub fn up(config: Config, directory: &Path) -> Result<()> {
    let plan = config.plan()?;
    let _lock = Lock::acquire()?;
    ensure!(!directory.exists(), "new state directory required");
    let client = Client(&config);
    client.request("GET", "/1.0", None)?;
    client.request(
        "GET",
        &format!("/1.0/images/{}", config.image_fingerprint),
        None,
    )?;
    fs::create_dir(directory)?;
    let path = directory.join("incus.json");
    let mut journal = Journal {
        config: config.clone(),
        plan,
        cables: Vec::new(),
        phase: "creating-project".into(),
    };
    save(&path, &journal)?;
    client.complete("POST", "/1.0/projects", Some(&json!({"name":config.project,"description":journal.plan.owner,"config":{"features.images":"false","features.profiles":"false"}})))?;
    journal.phase = "creating-instances".into();
    save(&path, &journal)?;
    for node in &journal.plan.nodes {
        client.complete("POST", "/1.0/instances", Some(&node.spec))?;
        client.complete(
            "PUT",
            &format!("/1.0/instances/{}/state", node.name),
            Some(&json!({"action":"start","timeout":120,"force":false})),
        )?;
    }
    journal.phase = "wiring".into();
    save(&path, &journal)?;
    for cable in &journal.plan.cables.clone() {
        let ends = [
            Endpoint::observe(&cable.peers[0])?,
            Endpoint::observe(&cable.peers[1])?,
        ];
        let owned = HostCable::connect(&cable.name, &journal.plan.owner, ends)?;
        journal.cables.push(owned.clone());
        save(&path, &journal)?;
        for destination in 0..2 {
            owned.shape_towards(
                destination,
                cable.latency_ns,
                cable.bandwidth_bps,
                cable.queue_packets,
            )?;
        }
        owned.set_up(cable.up)?;
    }
    journal.phase = "active".into();
    save(&path, &journal)?;
    // No implicit provisioning: a running init process is not a configured node.
    Ok(())
}

pub fn down(directory: &Path) -> Result<()> {
    let _lock = Lock::acquire()?;
    let path = directory.join("incus.json");
    let mut journal: Journal =
        serde_json::from_slice(&crate::input::read(&path, crate::input::CONFIG_LIMIT)?)?;
    let client = Client(&journal.config);
    let project = client.request(
        "GET",
        &format!("/1.0/projects/{}", journal.config.project),
        None,
    )?;
    ensure!(
        project["metadata"]["description"] == journal.plan.owner,
        "foreign project; cleanup refused"
    );
    let instances = client.request("GET", "/1.0/instances?recursion=1", None)?;
    let instances = instances["metadata"]
        .as_array()
        .context("instances array")?;
    for node in instances {
        ensure!(
            node["config"]["user.ncp.owner"] == journal.plan.owner
                && journal.plan.nodes.iter().any(|n| node["name"] == n.name),
            "foreign instance; cleanup refused"
        );
    }
    for cable in &journal.cables {
        cable.verify()?;
    }
    journal.phase = "destroying".into();
    save(&path, &journal)?;
    while let Some(cable) = journal.cables.last() {
        cable.remove()?;
        journal.cables.pop();
        save(&path, &journal)?;
    }
    for node in instances {
        let name = node["name"].as_str().context("instance name")?;
        if node["status_code"] != 102 {
            client.complete(
                "PUT",
                &format!("/1.0/instances/{name}/state"),
                Some(&json!({"action":"stop","timeout":30,"force":true})),
            )?;
        }
        client.complete("DELETE", &format!("/1.0/instances/{name}"), None)?;
    }
    client.complete(
        "DELETE",
        &format!("/1.0/projects/{}", journal.config.project),
        None,
    )?;
    journal.phase = "destroyed".into();
    save(&path, &journal)?;
    Ok(())
}
