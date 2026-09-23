# Intended state, fault propagation and ownership

## INT-03 — The backends do not form one causal datacenter

**High · Gap · Existing G004.** [simulator/src/netbox.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/netbox.rs#L261) emits a manifest. [simulator/src/infiniband.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/infiniband.rs#L90) and [simulator/src/linux.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/linux.rs#L26) own separate snapshots. Mokka binds model names to an external Kubernetes cluster whose Pod networking does not traverse either snapshot. The physical RDMA loopback selects a host RNIC directly.

Consequently, cutting an `IbSimulation` link does not interrupt a Mokka Pod or a separate NCCL Socket collective. `bandwidth_latency_enforced=false`, `rdma_payload_tested=false` and `nccl_tested=false` in the IB plan correctly disclose this. Native ibsim/OpenSM manages discovery/MADs; it does not enforce the model's rate/latency or transport RDMA payloads.

**Remedy/acceptance:** define shared stable device/port identifiers and a single owner for topology generations and fault events. Each backend should acknowledge which effects it implements and reject unsupported cross-plane claims. A joint scenario must show a selected fault affecting the intended workload while independent planes remain operational; no unrelated success should certify the missing dependency.

## INT-04 — NetBox records are narrowed to selected data interfaces

**Medium · Scope gap.** [simulator/src/netbox.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/netbox.rs#L217) excludes management ports and chooses one medium by interface type string. `compile` iterates those interfaces to create nodes. Devices with no selected ports disappear from the resulting graph; generic config CPU/memory/storage and one configured latency stand in for device-specific quantities. Rack placement, modules/GPUs, PCIe/NIC affinity, power feeds and cable-dependent propagation are not acquired.

Unsupported split/passive/LAG/bridge structures fail explicitly, which is safer than inventing connectivity. The usability gap is that the output is not a complete NetBox hardware inventory and does not enumerate every dropped object/reason. A source-device count cannot be assumed to equal a modeled-node count.

**Remedy/acceptance:** emit a transformation report listing included/excluded IDs, reasons, configured defaults and unsupported structures. Add device/port-specific overrides and explicit unit/provenance fields where modeled. Test mixed management/Ethernet/IB devices and a device with no selected ports; require the exclusion to be visible rather than silently interpreted as nonexistent infrastructure.

## INT-05 — Rollback evidence stops at owned Helm releases

**Medium · Risk, no residual host files reproduced.** [simulator/src/mokka.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/mokka.rs#L579) correctly filters releases by a unique ownership token before uninstalling them, and reports partial cleanup when uninstall fails. The pinned [Mokka DaemonSet](https://github.com/NVIDIA/k8s-test-infra/blob/4a44f73b41abc041f93d9b745677fa3210f65a84/deployments/nvml-mock/helm/nvml-mock/templates/daemonset.yaml) nevertheless has privileged host mounts, including `/var/lib/nvml-mock`, CDI and driver-related paths. A successful Helm uninstall is not, by itself, an adapter assertion that node files/devices are restored.

The adapter does not inspect those host paths after rollback or coordinate node-level ownership across different namespaces/runs. The no-advertised-GPU preflight is useful but does not establish that no other mock installation owns host artifacts. This report does not assert that upstream cleanup is absent; it identifies the missing adapter-level proof and cross-run ownership boundary.

**Remedy/acceptance:** keep disposable-node/cluster guidance; add a node-scoped ownership lease and an upstream-aware cleanup verification contract. Test partial apply, interrupted termination, a competing run in another namespace and an existing unowned artifact. Preserve unowned host state and report unresolved cleanup explicitly instead of interpreting `rolled-back` as full host restoration.
