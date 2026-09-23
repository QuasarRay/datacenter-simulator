// SPDX-License-Identifier: RPL-1.5
//! Read-only NetBox REST snapshots compiled into the simulator's validated graph.
use crate::{
    Simulator,
    manifest::{Manifest, ManifestInterface, ManifestLink, ManifestNode, Topology},
    model::{LinkSpec, Resources, Role},
};
use anyhow::{Context, Result, bail, ensure};
use reqwest::{
    Url,
    blocking::Client,
    header::{AUTHORIZATION, HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::Path,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Medium {
    Ethernet,
    Infiniband,
}
fn token_env() -> String {
    "NETBOX_TOKEN".into()
}
fn yes() -> bool {
    true
}
fn latency() -> u64 {
    1000
}
fn image() -> String {
    "generic/cachyos".into()
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NetBoxConfig {
    pub api_url: String,
    #[serde(default = "token_env")]
    pub token_env: String,
    #[serde(default)]
    pub allow_http: bool,
    #[serde(default)]
    pub device_filter: BTreeMap<String, String>,
    /// Explicit role slug -> simulator role mapping; unknown roles are errors.
    pub roles: BTreeMap<String, Role>,
    pub medium: Medium,
    #[serde(default = "yes")]
    pub planned_links_up: bool,
    #[serde(default)]
    pub default_bandwidth_bps: Option<u64>,
    #[serde(default = "latency")]
    pub latency_ns: u64,
    #[serde(default = "image")]
    pub image: String,
    #[serde(default)]
    pub resources: Resources,
    #[serde(default)]
    pub acquisition: AcquisitionLimits,
    /// Explicit offline relocation; the override is recorded on emitted nodes.
    #[serde(default)]
    pub replay_allow_origin_change: bool,
    #[serde(default)]
    pub snapshot_max_age_secs: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct AcquisitionLimits {
    pub records: usize,
    pub bytes: usize,
    pub requests: usize,
    pub duration_secs: u64,
}
impl Default for AcquisitionLimits {
    fn default() -> Self {
        Self {
            records: 100_000,
            bytes: 64 * 1024 * 1024,
            requests: 256,
            duration_secs: 120,
        }
    }
}
struct Budget {
    limits: AcquisitionLimits,
    records: usize,
    bytes: usize,
    requests: usize,
    began: Instant,
}
impl Budget {
    fn new(limits: &AcquisitionLimits) -> Result<Self> {
        ensure!(
            limits.records > 0
                && limits.bytes > 0
                && limits.requests > 0
                && (1..=3600).contains(&limits.duration_secs),
            "invalid NetBox acquisition budget"
        );
        Ok(Self {
            limits: limits.clone(),
            records: 0,
            bytes: 0,
            requests: 0,
            began: Instant::now(),
        })
    }
    fn request(&mut self) -> Result<Duration> {
        ensure!(
            self.requests < self.limits.requests,
            "NetBox aggregate request budget exceeded"
        );
        let remaining = Duration::from_secs(self.limits.duration_secs)
            .checked_sub(self.began.elapsed())
            .context("NetBox aggregate duration exceeded")?;
        ensure!(!remaining.is_zero(), "NetBox aggregate duration exceeded");
        self.requests += 1;
        Ok(remaining.min(Duration::from_secs(30)))
    }
    fn records(&mut self, n: usize) -> Result<()> {
        self.records = self
            .records
            .checked_add(n)
            .context("record count overflow")?;
        ensure!(
            self.records <= self.limits.records,
            "NetBox aggregate record budget exceeded"
        );
        Ok(())
    }
}
fn api_root(value: &str, allow_http: bool) -> Result<Url> {
    let root = Url::parse(value)?;
    ensure!(
        (root.scheme() == "https" || (root.scheme() == "http" && allow_http))
            && root.username().is_empty()
            && root.password().is_none()
            && root.query().is_none()
            && root.fragment().is_none()
            && root.path().ends_with("/api/"),
        "api_url must end in /api/, without credentials/query; HTTP requires allow_http"
    );
    Ok(root)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Choice {
    pub value: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Reference {
    pub id: u64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeviceRole {
    pub slug: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Device {
    pub id: u64,
    pub name: String,
    pub role: DeviceRole,
    pub status: Choice,
    #[serde(default)]
    pub custom_fields: BTreeMap<String, Value>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Interface {
    pub id: u64,
    pub name: String,
    pub device: Reference,
    #[serde(rename = "type")]
    pub kind: Choice,
    pub enabled: bool,
    pub mgmt_only: bool,
    /// NetBox documents interface speed in kilobits per second.
    pub speed: Option<u64>,
    pub cable: Option<Reference>,
    #[serde(default)]
    pub lag: Option<Reference>,
    #[serde(default)]
    pub parent: Option<Reference>,
    #[serde(default)]
    pub bridge: Option<Reference>,
    #[serde(default)]
    pub duplex: Option<Choice>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Termination {
    pub object_type: String,
    pub object_id: u64,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Cable {
    pub id: u64,
    pub status: Choice,
    pub a_terminations: Vec<Termination>,
    pub b_terminations: Vec<Termination>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Snapshot {
    pub api_url: String,
    pub fetched_at: String,
    pub devices: Vec<Device>,
    pub interfaces: Vec<Interface>,
    pub cables: Vec<Cable>,
}

fn selected(i: &Interface, medium: Medium) -> bool {
    if i.mgmt_only {
        return false;
    }
    match medium {
        Medium::Infiniband => i.kind.value.starts_with("infiniband-"),
        Medium::Ethernet => i.kind.value.contains("base-"),
    }
}
// IDs, rather than display names, survive NetBox renames. Base36 fits every
// u64 interface ID inside Linux's 15-byte IFNAMSIZ limit with a letter prefix.
fn interface_name(mut id: u64) -> String {
    let mut digits = Vec::new();
    loop {
        digits.push(b"0123456789abcdefghijklmnopqrstuvwxyz"[(id % 36) as usize] as char);
        id /= 36;
        if id == 0 {
            break;
        }
    }
    format!("i{}", digits.iter().rev().collect::<String>())
}
fn operating(status: &str, planned: bool) -> Result<bool> {
    match status {
        "active" => Ok(true),
        "planned" | "staged" => Ok(planned),
        "offline" | "failed" | "inventory" | "decommissioning" => Ok(false),
        _ => bail!("unsupported NetBox device status {status}"),
    }
}
fn unique<T>(items: &[T], id: impl Fn(&T) -> u64) -> Result<BTreeMap<u64, &T>> {
    let mut map = BTreeMap::new();
    for item in items {
        let key = id(item);
        ensure!(
            key > 0 && map.insert(key, item).is_none(),
            "duplicate or zero NetBox ID {key}"
        );
    }
    Ok(map)
}

/// Compile a bounded network plane. Unsupported cable paths fail instead of
/// being replaced by direct links. This never changes a running model in place.
pub fn compile(config: &NetBoxConfig, snapshot: &Snapshot) -> Result<Manifest> {
    config.resources.validate()?;
    let expected = api_root(&config.api_url, config.allow_http)?;
    let origin = api_root(&snapshot.api_url, config.allow_http)?;
    ensure!(
        expected == origin || config.replay_allow_origin_change,
        "snapshot origin differs from configured NetBox; explicit replay_allow_origin_change required"
    );
    let fetched = chrono::DateTime::parse_from_rfc3339(&snapshot.fetched_at)
        .context("invalid snapshot fetched_at")?;
    let age = chrono::Utc::now().signed_duration_since(fetched);
    ensure!(
        age.num_seconds() >= -300,
        "snapshot fetched_at is in the future"
    );
    if let Some(max) = config.snapshot_max_age_secs {
        ensure!(
            age.num_seconds().max(0) as u64 <= max,
            "snapshot exceeds maximum age"
        );
    }
    let total = snapshot
        .devices
        .len()
        .checked_add(snapshot.interfaces.len())
        .and_then(|n| n.checked_add(snapshot.cables.len()))
        .context("snapshot count overflow")?;
    ensure!(
        total <= config.acquisition.records,
        "snapshot record budget exceeded"
    );
    ensure!(
        crate::limits::size(snapshot)? <= config.acquisition.bytes,
        "snapshot byte budget exceeded"
    );
    let devices = unique(&snapshot.devices, |d| d.id)?;
    let interfaces = unique(&snapshot.interfaces, |i| i.id)?;
    let cables = unique(&snapshot.cables, |c| c.id)?;
    let chosen: BTreeMap<_, _> = interfaces
        .iter()
        .filter(|(_, i)| selected(i, config.medium))
        .map(|(k, v)| (*k, *v))
        .collect();
    ensure!(
        !chosen.is_empty(),
        "NetBox scope contains no selected data interfaces"
    );
    let mut content = Topology::default();
    let mut endpoints = BTreeMap::new();
    let mut bandwidth = BTreeMap::new();
    let mut active = BTreeMap::new();
    for i in chosen.values() {
        let d = devices
            .get(&i.device.id)
            .context("interface references a device outside the snapshot")?;
        let role = *config
            .roles
            .get(&d.role.slug)
            .with_context(|| format!("unmapped NetBox device role {}", d.role.slug))?;
        ensure!(
            i.lag.is_none() && i.parent.is_none() && i.bridge.is_none(),
            "LAG, bridge and parent interfaces require a dedicated model: {}:{}",
            d.name,
            i.name
        );
        ensure!(
            i.duplex
                .as_ref()
                .is_none_or(|d| d.value == "full" || d.value.is_empty() || d.value == "auto"),
            "half-duplex links are not modeled"
        );
        let rate = match i.speed {
            Some(kbps) if kbps > 0 => kbps.checked_mul(1000).context("NetBox speed overflow")?,
            _ => config.default_bandwidth_bps.context(
                "NetBox interface speed is unset; provide an explicit default_bandwidth_bps",
            )?,
        };
        ensure!(rate > 0, "zero interface bandwidth");
        bandwidth.insert(i.id, rate);
        active.insert(
            i.id,
            i.enabled && operating(&d.status.value, config.planned_links_up)?,
        );
        let node_name = format!("nb{}", d.id);
        let iface_name = interface_name(i.id);
        let n = content
            .nodes
            .entry(node_name.clone())
            .or_insert_with(|| ManifestNode {
                os: config.image.clone(),
                cpu: config.resources.cpu,
                memory: config.resources.memory,
                storage: config.resources.storage,
                role,
                interfaces: BTreeMap::new(),
                labels: BTreeMap::from([
                    ("netbox.snapshot".into(), serde_json::json!({"api_url":snapshot.api_url, "configured_api_url":config.api_url, "fetched_at":snapshot.fetched_at, "origin_override":expected != origin})),
                    ("netbox.device_id".into(), Value::from(d.id)),
                    ("netbox.device_name".into(), Value::from(d.name.clone())),
                    ("netbox.interfaces".into(), serde_json::json!({})),
                    (
                        "simulator.medium".into(),
                        Value::from(match config.medium {
                            Medium::Ethernet => "ethernet",
                            Medium::Infiniband => "infiniband",
                        }),
                    ),
                ]),
                user_data: None,
                meta_data: None,
                image_metadata: BTreeMap::new(),
            });
        for (field, label) in [
            ("simulator_gpu_profile", "mokka.profile"),
            ("simulator_gpu_count", "mokka.gpu_count"),
            ("simulator_kubernetes_node", "kubernetes.node"),
        ] {
            if let Some(value) = d.custom_fields.get(field).filter(|v| !v.is_null()) {
                n.labels.insert(label.into(), value.clone());
            }
        }
        n.labels
            .get_mut("netbox.interfaces")
            .expect("mapping initialized")[&iface_name] =
            serde_json::json!({"id":i.id,"name":i.name});
        ensure!(
            n.interfaces
                .insert(iface_name.clone(), ManifestInterface::default())
                .is_none(),
            "duplicate interface ID"
        );
        endpoints.insert(i.id, format!("{node_name}:{iface_name}"));
    }
    let needed: BTreeSet<_> = chosen
        .values()
        .filter_map(|i| i.cable.as_ref().map(|c| c.id))
        .collect();
    let mut used = BTreeSet::new();
    for id in needed {
        let c = cables
            .get(&id)
            .with_context(|| format!("cable {id} missing from snapshot"))?;
        ensure!(
            c.a_terminations.len() == 1 && c.b_terminations.len() == 1,
            "split or unterminated cable {id} is unsupported"
        );
        let a = &c.a_terminations[0];
        let b = &c.b_terminations[0];
        ensure!(
            a.object_type == "dcim.interface" && b.object_type == "dcim.interface",
            "cable {id} includes a passive panel or non-interface termination; trace modeling is required"
        );
        let mut ends = Vec::new();
        for termination in [a, b] {
            let i = chosen.get(&termination.object_id).context(
                "cable crosses the selected device/medium boundary; include both endpoint devices",
            )?;
            ensure!(
                i.cable.as_ref().is_some_and(|r| r.id == id),
                "inconsistent interface/cable snapshot"
            );
            ensure!(used.insert(i.id), "interface appears on multiple cables");
            ends.push(endpoints[&i.id].clone());
        }
        let connected = match c.status.value.as_str() {
            "connected" => true,
            "planned" => config.planned_links_up,
            "decommissioning" => false,
            s => bail!("unsupported NetBox cable status {s}"),
        };
        content.links.push(ManifestLink {
            endpoints: [ends.remove(0), ends.remove(0)],
            condition: LinkSpec {
                up: connected && active[&a.object_id] && active[&b.object_id],
                bandwidth_bps: bandwidth[&a.object_id].min(bandwidth[&b.object_id]),
                latency_ns: config.latency_ns,
            },
        });
    }
    ensure!(
        chosen
            .values()
            .filter(|i| i.cable.is_some())
            .all(|i| used.contains(&i.id)),
        "a cabled interface is absent from its cable terminations; retry the snapshot"
    );
    let manifest = Manifest {
        format: "JSON".into(),
        name: "NetBox intended topology".into(),
        ztp: None,
        content,
    };
    // Reuse the simulator validators before exposing any plan or starting a backend.
    Simulator::new().import(manifest.clone(), false)?;
    Ok(manifest)
}

#[derive(Deserialize)]
struct Page {
    count: usize,
    next: Option<String>,
    results: Vec<Value>,
}

pub struct NetBoxClient {
    client: Client,
    root: Url,
}
impl NetBoxClient {
    pub fn new(config: &NetBoxConfig) -> Result<Self> {
        let root = api_root(&config.api_url, config.allow_http)?;
        let token = std::env::var(&config.token_env)
            .with_context(|| format!("set {} for read-only NetBox access", config.token_env))?;
        ensure!(!token.is_empty(), "empty NetBox token");
        // NetBox >=4.5 uses v2 Bearer tokens; legacy v1 tokens use Token.
        let scheme = if token.starts_with("nbt_") {
            "Bearer"
        } else {
            "Token"
        };
        let mut authorization = HeaderValue::from_str(&format!("{scheme} {token}"))?;
        authorization.set_sensitive(true);
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::none())
            .default_headers(HeaderMap::from_iter([(AUTHORIZATION, authorization)]))
            .build()?;
        Ok(Self { client, root })
    }
    fn collection<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        query: &[(String, String)],
        budget: &mut Budget,
    ) -> Result<Vec<T>> {
        let mut url = self.root.join(endpoint)?;
        url.query_pairs_mut()
            .extend_pairs(query)
            .append_pair("limit", "1000");
        let path = url.path().to_owned();
        let mut seen = BTreeSet::new();
        let mut records = Vec::new();
        let mut expected = None;
        loop {
            ensure!(
                url.origin() == self.root.origin()
                    && url.path() == path
                    && url.username().is_empty()
                    && url.password().is_none()
                    && url.fragment().is_none(),
                "NetBox pagination escaped its authenticated endpoint"
            );
            ensure!(
                seen.len() < 1000 && seen.insert(url.as_str().to_owned()),
                "NetBox pagination cycle or page limit"
            );
            let duration = budget.request()?;
            let response = self.client.get(url.clone()).timeout(duration).send()?;
            ensure!(
                response.status().is_success(),
                "NetBox GET failed: {}",
                response.status()
            );
            let mut bytes = Vec::new();
            let limit = (8 * 1024 * 1024).min(budget.limits.bytes.saturating_sub(budget.bytes));
            response.take(limit as u64 + 1).read_to_end(&mut bytes)?;
            ensure!(
                bytes.len() <= limit,
                "NetBox page or aggregate byte budget exceeded"
            );
            budget.bytes += bytes.len();
            let page: Page = serde_json::from_slice(&bytes)?;
            ensure!(
                page.count <= 100_000 && expected.is_none_or(|n| n == page.count),
                "NetBox count changed or exceeds scope limit"
            );
            expected = Some(page.count);
            budget.records(page.results.len())?;
            for value in page.results {
                records.push(serde_json::from_value(value)?);
            }
            ensure!(
                records.len() <= page.count,
                "NetBox pagination exceeds count"
            );
            match page.next {
                Some(next) => url = url.join(&next)?,
                None => break,
            }
        }
        ensure!(
            Some(records.len()) == expected,
            "incomplete NetBox pagination"
        );
        Ok(records)
    }
    pub fn snapshot(&self, config: &NetBoxConfig) -> Result<Snapshot> {
        ensure!(
            self.root == api_root(&config.api_url, config.allow_http)?,
            "client/config NetBox origin mismatch"
        );
        let mut budget = Budget::new(&config.acquisition)?;
        let first = self.read_snapshot(config, &mut budget)?;
        let second = self.read_snapshot(config, &mut budget)?;
        ensure!(
            same_intent(&first, &second)?,
            "NetBox intent changed during acquisition; retry in a change window"
        );
        ensure!(
            budget.began.elapsed() <= Duration::from_secs(budget.limits.duration_secs),
            "NetBox aggregate duration exceeded"
        );
        Ok(second)
    }
    fn read_snapshot(&self, config: &NetBoxConfig, budget: &mut Budget) -> Result<Snapshot> {
        ensure!(
            !config.device_filter.keys().any(|k| [
                "limit", "offset", "brief", "fields", "exclude", "omit"
            ]
            .contains(&k.as_str())),
            "device_filter must not alter pagination or response fields"
        );
        let devices: Vec<Device> = self.collection(
            "dcim/devices/",
            &config
                .device_filter
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<Vec<_>>(),
            budget,
        )?;
        ensure!(!devices.is_empty(), "empty NetBox device scope");
        let mut interfaces = Vec::<Interface>::new();
        for chunk in devices.chunks(50) {
            interfaces.extend(
                self.collection(
                    "dcim/interfaces/",
                    &chunk
                        .iter()
                        .map(|d| ("device_id".into(), d.id.to_string()))
                        .collect::<Vec<_>>(),
                    budget,
                )?,
            );
        }
        let ids: BTreeSet<_> = interfaces
            .iter()
            .filter(|i| selected(i, config.medium))
            .filter_map(|i| i.cable.as_ref().map(|c| c.id))
            .collect();
        let ids: Vec<_> = ids.into_iter().collect();
        let mut cables = Vec::new();
        for chunk in ids.chunks(50) {
            cables.extend(
                self.collection(
                    "dcim/cables/",
                    &chunk
                        .iter()
                        .map(|id| ("id".into(), id.to_string()))
                        .collect::<Vec<_>>(),
                    budget,
                )?,
            );
        }
        Ok(Snapshot {
            api_url: self.root.to_string(),
            fetched_at: chrono::Utc::now().to_rfc3339(),
            devices,
            interfaces,
            cables,
        })
    }
}

/// Two equal observations detect concurrent edits, but are not database isolation.
fn same_intent(a: &Snapshot, b: &Snapshot) -> Result<bool> {
    let normalize = |s: &Snapshot| -> Result<Value> {
        let mut devices = s.devices.clone();
        devices.sort_by_key(|d| d.id);
        let mut interfaces = s.interfaces.clone();
        interfaces.sort_by_key(|i| i.id);
        let mut cables = s.cables.clone();
        cables.sort_by_key(|c| c.id);
        Ok(serde_json::json!({"devices":devices,"interfaces":interfaces,"cables":cables}))
    };
    Ok(normalize(a)? == normalize(b)?)
}

pub fn import(config: &NetBoxConfig, directory: &Path) -> Result<()> {
    let snapshot = NetBoxClient::new(config)?.snapshot(config)?;
    let manifest = compile(config, &snapshot)?;
    std::fs::create_dir(directory).context("NetBox output directory must be new")?;
    std::fs::write(
        directory.join("snapshot.json"),
        serde_json::to_vec_pretty(&snapshot)?,
    )?;
    std::fs::write(
        directory.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(())
}

#[cfg(test)]
mod snapshot_tests {
    use super::*;
    #[test]
    fn acquisition_comparison_detects_edits_but_ignores_listing_order() {
        let first: Snapshot = serde_json::from_str(include_str!(
            "../../integrations/netbox/snapshot.example.json"
        ))
        .unwrap();
        let mut second = first.clone();
        second.devices.reverse();
        second.interfaces.reverse();
        second.cables.reverse();
        second.fetched_at = "another instant".into();
        assert!(same_intent(&first, &second).unwrap());
        second.interfaces[0].enabled = !second.interfaces[0].enabled;
        assert!(!same_intent(&first, &second).unwrap());
    }
}

#[cfg(test)]
mod budget_tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
    };
    fn fixture() -> (NetBoxConfig, Snapshot) {
        (
            serde_json::from_str(include_str!(
                "../../integrations/netbox/config.example.json"
            ))
            .unwrap(),
            serde_json::from_str(include_str!(
                "../../integrations/netbox/snapshot.example.json"
            ))
            .unwrap(),
        )
    }
    fn serve(pages: Vec<Value>) -> (NetBoxClient, std::thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let root = Url::parse(&format!("http://{}/api/", listener.local_addr().unwrap())).unwrap();
        let handle = std::thread::spawn(move || {
            for page in pages {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut request = Vec::new();
                while !request.ends_with(b"\r\n\r\n") {
                    let mut b = [0];
                    stream.read_exact(&mut b).unwrap();
                    request.push(b[0]);
                }
                let body = page.to_string();
                write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
            }
        });
        (
            NetBoxClient {
                client: Client::builder().no_proxy().build().unwrap(),
                root,
            },
            handle,
        )
    }
    fn page(records: impl Serialize) -> Value {
        let records = serde_json::to_value(records).unwrap();
        serde_json::json!({"count":records.as_array().unwrap().len(),"next":null,"results":records})
    }
    #[test]
    fn one_record_budget_spans_both_observations() {
        let (mut config, snapshot) = fixture();
        let pages = vec![
            page(&snapshot.devices),
            page(&snapshot.interfaces),
            page(&snapshot.cables),
            page(&snapshot.devices),
        ];
        let (client, server) = serve(pages);
        config.api_url = client.root.to_string();
        config.allow_http = true;
        config.acquisition.records =
            snapshot.devices.len() + snapshot.interfaces.len() + snapshot.cables.len();
        assert!(
            client
                .snapshot(&config)
                .unwrap_err()
                .to_string()
                .contains("aggregate record")
        );
        server.join().unwrap();
    }
    #[test]
    fn request_budget_spans_device_chunks_and_byte_budget_spans_pages() {
        let (mut config, snapshot) = fixture();
        let devices: Vec<_> = (1..=51)
            .map(|id| {
                let mut d = snapshot.devices[0].clone();
                d.id = id;
                d
            })
            .collect();
        let (client, server) = serve(vec![page(devices), page(Vec::<Interface>::new())]);
        config.api_url = client.root.to_string();
        config.allow_http = true;
        config.acquisition.requests = 2;
        assert!(
            client
                .snapshot(&config)
                .unwrap_err()
                .to_string()
                .contains("aggregate request")
        );
        server.join().unwrap();
        let device_page = page(&snapshot.devices);
        let interface_page = page(&snapshot.interfaces);
        config.acquisition.bytes =
            device_page.to_string().len() + interface_page.to_string().len() - 1;
        let (client, server) = serve(vec![device_page, interface_page]);
        config.api_url = client.root.to_string();
        config.acquisition.requests = 20;
        assert!(
            client
                .snapshot(&config)
                .unwrap_err()
                .to_string()
                .contains("aggregate byte")
        );
        server.join().unwrap();
    }
    #[test]
    fn acquisition_deadline_and_offline_provenance_are_enforced() {
        let mut budget = Budget::new(&AcquisitionLimits::default()).unwrap();
        budget.began = Instant::now() - Duration::from_secs(121);
        assert!(budget.request().is_err());
        let (mut config, mut snapshot) = fixture();
        snapshot.api_url = "https://other.invalid/api/".into();
        assert!(compile(&config, &snapshot).is_err());
        config.replay_allow_origin_change = true;
        let manifest = compile(&config, &snapshot).unwrap();
        assert_eq!(
            manifest.content.nodes["nb1"].labels["netbox.snapshot"]["origin_override"],
            true
        );
        snapshot.fetched_at = "not a timestamp".into();
        assert!(compile(&config, &snapshot).is_err());
        snapshot.fetched_at = "2000-01-01T00:00:00Z".into();
        config.snapshot_max_age_secs = Some(60);
        assert!(compile(&config, &snapshot).is_err());
    }
}
