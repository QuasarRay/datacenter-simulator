# NetBox, native InfiniBand and Kubernetes GPU contracts

NetBox is the source of intended physical state. The Rust importer takes a
read-only REST snapshot, validates its devices/interfaces/cables and emits the
same Air-style manifest used by the simulator. The `ibsim` feature exports that
graph through patchbay and starts the real pinned ibsim C program, libumad2sim
and OpenSM. The `mokka` feature maps selected GPU hosts to explicitly named
Kubernetes nodes and deploys the pinned NVIDIA/k8s-test-infra chart.

| Component | Executes | Does not establish |
|---|---|---|
| NetBox REST importer | Device, interface, cable and GPU intent translation | Continuous reconciliation or an atomic database snapshot |
| patchbay + ibsim + OpenSM | SMP/SA management datagrams, discovery, LID assignment, forwarding tables, cable failure/recovery | Verbs payloads, an RNIC, IPoIB or throughput/latency fidelity |
| NVIDIA k8s-test-infra (Mokka) | Upstream GPU metadata, NVML and Kubernetes deployment contracts | CUDA kernels, NCCL collectives or RDMA payload correctness |
| Existing DeepOps/NCCL path | Real guest provisioning and real CUDA/NCCL on dedicated GPUs | GPU execution on CPU-only Mokka nodes |

No Rust–Python language bridge is used. NetBox runs as an external service;
the simulator consumes HTTP JSON. No NCCL substitute is introduced. Mokka's
IB emulators are disabled; its driver mocks are never passed to NCCL workers.
Kubernetes pod networking remains the selected cluster's networking. Sharing
NetBox intent does not connect that pod network to patchbay's IB namespace.

## Build and inspect intent

From the repository root:

```sh
git submodule update --init --recursive
cargo build --locked --manifest-path simulator/Cargo.toml --features ibsim,netbox,mokka \
  --bin datacenter-simulator --example ibsim_fabric
simulator/target/debug/datacenter-simulator netbox-compile \
  integrations/netbox/config.example.json integrations/netbox/snapshot.example.json
simulator/target/debug/datacenter-simulator ibsim-plan integrations/netbox/manifest.example.json
simulator/target/debug/datacenter-simulator mokka-plan integrations/k8s-test-infra/config.example.json
```

The checked snapshot is an example, not data retrieved from your NetBox server.
For your server, copy `integrations/netbox/config.example.json`, set its API URL,
select devices with REST filters, and explicitly map NetBox role slugs to `host`
or `switch`. Use HTTPS; plaintext HTTP requires `allow_http: true` for a local
lab. Set the configured environment variable to a **read-only** NetBox token:
v2 `nbt_<key>.<token>` uses Bearer authentication; legacy v1 uses Token.

```sh
# Set NETBOX_TOKEN in your environment without putting it in a config or log.
simulator/target/debug/datacenter-simulator netbox-import my-netbox.json new-intent
```

The new directory contains `snapshot.json` and validated `manifest.json`. Tokens
are not written. Pagination is bounded, redirects are disabled, and every page
must stay on the original origin and endpoint path. Replay with `netbox-compile`.
The import never writes to NetBox or mutates an existing running simulation.
Refresh by importing a new snapshot and rebuilding the backend.

The importer selects one data plane: `ethernet` or `infiniband`. Management-only
ports are excluded. Both ends of every selected cable must be in scope. Direct
one-to-one interface cables are supported; passive panels, breakout cables,
LAG, parent and bridge interfaces fail explicitly instead of producing shortcut
edges. Interface `speed` is converted from NetBox kbps to bps. Missing speed
requires `default_bandwidth_bps`; disabled interfaces/offline devices and
administratively down cables remain down. `planned_links_up` determines whether
planned/staged intent is brought up. ibsim does not implement the manifest's
bandwidth/latency values; the Ethernet kernel backend does.

Device identifiers are `nb<NetBox ID>`. Interface identifiers are `i<base36 ID>`
and fit Linux's 15-byte interface-name limit. `netbox.device_name` and
`netbox.interfaces` labels retain source names/IDs, including FQDNs and names
such as `IB1/1`. Renaming objects does not change their physical mapping.
Cross-reference/count inconsistencies abort a snapshot. For consistent exports
across concurrent NetBox edits, use a change window; paginated HTTP reads are
not a database transaction. The source schema revision is recorded in
`simulator/upstreams.json`; CI exercises a real NetBox 4.7 service.

## Run native InfiniBand

On a CachyOS/Arch host, install `base-devel`, `rdma-core`, `opensm`, `iproute2`
and `nftables`; ensure `ibnetdiscover`, `smpquery` and `ibtracert` are available.
On Ubuntu, install `build-essential libibmad-dev libibumad-dev opensm
infiniband-diags nftables iproute2`. Build both native artifacts from the pinned
source, then run the imported graph:

```sh
make -C simulator/vendor/patchbay/vendor/ibsim/ibsim
make -C simulator/vendor/patchbay/vendor/ibsim/umad2sim
simulator/target/debug/examples/ibsim_fabric new-intent/manifest.json new-ib-run
```

Run as the checkout owner on a host that permits unprivileged user/network
namespaces. The example creates its own user namespace; using sudo can make a
private checkout inaccessible after the UID mapping changes.

The example requires at least two connected host HCAs. It starts a dedicated
patchbay namespace, launches OpenSM, checks actual native discovery and routes,
partitions the second host, verifies loss of reachability and restores it.
It writes native reports and `result.json` with explicit validation scope.
`ibsim-plan` records simulator-to-native node/port mappings. The library API
`IbSimulation` supports arbitrary validated graphs, scoped native commands and
live `set_link_up` calls; changes commit to the model after native acknowledgement.
UMAD clients attach to the first HCA port (or switch port zero). Explicit
shutdown reaps services; Drop kills owned process groups. No global preload is
installed. Native execution requires Linux sockets and namespace privileges.

## Deploy NVIDIA k8s-test-infra (Mokka)

The upstream project is pinned as `integrations/k8s-test-infra/upstream`. Build
its image from that source; the required revision tag ties the configuration
to that build convention. Applying also requires the immutable OCI digest:

```sh
docker build -f integrations/k8s-test-infra/upstream/deployments/nvml-mock/Dockerfile \
  -t nvml-mock:simulator-4a44f73b41ab integrations/k8s-test-infra/upstream
```

Copy `config.example.json` and set explicit cluster context, dedicated namespace,
manifest and source checkout paths. Load/publish that image where the selected
**CPU lab nodes** can pull it. Set `image` to `registry/repository:simulator-4a44f73b41ab@sha256:<64 hex digest>` from the pushed build. Planning accepts a tag and reports `image_pinned: false`; apply rejects it. The CI uses a disposable local registry and records the running image IDs. Bind each model host to a Kubernetes node, GPU
profile and count using `nodes`, or omit `nodes` to use the imported labels.
NetBox custom fields `simulator_gpu_profile`, `simulator_gpu_count` and
`simulator_kubernetes_node` map to those labels. The example uses two T4 GPUs
on each of two workers. Counts exceeding the selected upstream profile fail.

```sh
kubectl --context=YOUR_CONTEXT label node YOUR_CPU_LAB_NODE simulator.quasarray.io/mokka=true
simulator/target/debug/datacenter-simulator mokka-render my-mokka.json new-mokka-plan
simulator/target/debug/datacenter-simulator mokka-apply my-mokka.json new-mokka-run
```

Apply requires an exact upstream Git revision, Ready opted-in nodes, exact
hostname selectors and no existing advertised NVIDIA GPU capacity. It uses the
local upstream Helm chart with atomic/wait, checks one deployed pod per selected
node, executes upstream `nvidia-smi`, and requires the intended GPU count.
The caller must select CPU lab nodes; Kubernetes capacity is not proof that a
machine lacks physical GPUs. Mokka mounts a simulated driver footprint on those
nodes. Ambient NRI injection, independent IB mocks, kernel logging, allocation
watching and mock topology/IMEX features are disabled by the generated values.
Uninstall the releases listed in `plan.json` with Helm against the same context
and namespace. Apply requires new release names and rolls back all releases created by this batch if any install or verification fails. `batch-state.json` records successful rollback or resources needing cleanup; existing releases are never overwritten. For complete test isolation, delete
the disposable cluster after collecting evidence.

## Validation

`intended-fabric.yml` runs Rust contracts, replays the snapshot, imports from a
real NetBox container using a read-only v2 token, and runs native OpenSM/ibsim
partition/recovery on the resulting manifest. Separate jobs run Mokka's own
`make test`, `make test-e2e-framework`, `make helm-tests`, `make helm-crds-tests`
and deploy the source-built image to a disposable Kind cluster for NVML checks.
The patchbay PR adds additional native backup-route and namespace tests.

DeepOps and real NCCL/verbs gates remain separate. Passing these CPU tests does
not imply that CUDA/NCCL or RDMA payload execution has passed; use the existing
hardware workflows for those claims.

Live NetBox acquisition now requires two matching complete observations. This
rejects changes observed between reads, including a stable but different second
view; it does not provide database isolation or detect an edit that is reverted
between observations. Continue to use a NetBox change window for strict snapshots.
Mokka verifies both tracked and untracked/ignored files in the consumed chart
before reading profiles or emitting plans. No Mokka result proves CUDA execution.
