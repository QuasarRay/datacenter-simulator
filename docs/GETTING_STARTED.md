# Start with the backend you need

Use the portable Rust model first on a consumer machine. It needs no GPU,
root privileges, Kubernetes, VMs, or Ansible. Its timings describe an explicitly
simplified message/link model. They do not predict DGX performance.

From the repository root:

```sh
# Cargo resolves optional local dependencies too. Initialize the four Rust forks;
# the DeepOps and Kubernetes integration checkouts are not needed for this step.
git submodule update --init --recursive \
  simulator/vendor/petgraph simulator/vendor/patchbay \
  simulator/vendor/nccl simulator/vendor/rust-ibverbs
cargo test --locked --manifest-path simulator/Cargo.toml --all-targets
cargo run --locked --manifest-path simulator/Cargo.toml -- --help
cargo run --locked --manifest-path simulator/Cargo.toml -- doctor portable
cargo run --locked --manifest-path simulator/Cargo.toml -- json
```

Rustup reads the root `rust-toolchain.toml`; the simulator-local copy has the same
pin. Install Rustup before these commands. A system Cargo that bypasses Rustup
does not select a toolchain from this file. `cargo +1.98.0 ...` selects it explicitly.
The toolchain file requests the formatting and lint components used in CI.

## Choose an execution scope

| Mode | What it actually does | Prerequisites and cost | Next guide |
|---|---|---|---|
| Portable model | Lifecycle, topology, lossless store-and-forward message timing, software verbs contracts | Rust; bounded retained state; no native GPU result | [Rust API](../simulator/README.md) |
| NCCL Socket | Real CUDA/NCCL over Linux namespace TCP links | Linux namespace capabilities; native libraries with explicit file identities; one distinct physical GPU per rank | [NCCL setup](../simulator/README.md#run-actual-nccl-through-the-fabric) |
| IB management | Native ibsim/OpenSM discovery, MADs and link faults | Linux namespaces; compiled ibsim/umad2sim; OpenSM/diagnostic tools | [IB setup](../simulator/docs/intended-state.md#run-native-infiniband) |
| Physical RDMA | Local hardware verbs smoke test | RNIC or configured kernel RDMA device, native build dependencies | [Rust guide](../simulator/README.md) |
| Mokka | Kubernetes GPU/NVML software contracts | Disposable CPU nodes explicitly opted in; kubectl, Helm, pinned image digest | [Kubernetes guide](../simulator/docs/intended-state.md) |
| DeepOps VM | Genuine guest provisioning, Slurm and optionally GPU benchmarks | KVM/QEMU, verified image, dedicated VFIO GPUs; example allocates 20 GiB guest RAM, 12 vCPU and 260 GiB virtual disks | [DeepOps guide](../integrations/deepops/README.md) |
| EX457 | Containerlab FRR/Ansible training exercises | Docker, Containerlab and the pack's pinned Python/Ansible environment | [EX457 v6](../cheatsheets/ex457/v6/README.md) |

`datacenter-simulator doctor MODE` prints compiled feature availability, missing
executables, namespace privileges and available host memory without creating
resources. `ibsim` discovery checks PATH; explicitly configured executable paths
must be checked against your integration config. A passing doctor is prerequisite
discovery, not proof that drivers, libraries, images or remote clusters work.
Use `doctor nccl`, `doctor ibsim`, `doctor vm`, `doctor mokka` or `doctor netbox`
with the corresponding feature-enabled binary. Missing requirements return a
nonzero exit status. The final runtime preflights still apply.

On CachyOS/Arch, the portable model uses the same Rustup commands. Native namespace
execution requires working `ip`, `tc`, `nft`, `sysctl` and `ping`; native compilation
also needs a C/C++ toolchain and the backend's headers/libraries. The IB guide has
Arch instructions. DeepOps provisions its documented Ubuntu guests; it does not
establish CachyOS guest support. There is currently no Incus backend. Choose the
portable or namespace modes when a full VM fleet would exceed your host budget.

Only initialize additional integrations when using them:

```sh
git submodule update --init --recursive integrations/k8s-test-infra/upstream
# For DeepOps instead:
git submodule update --init --recursive integrations/deepops/upstream simulator/vendor/nccl-tests
```

## Resource and fidelity boundaries

The Linux namespace backend permits at most 240 links, each with a helper bridge
and additional namespace/device state. The portable 4,096-node limit is not a
native capacity guarantee. Start with the small checked examples, measure host
memory/process/disk use, and leave room for compilation and the OS. Virtual disk
capacity is not a claim that all bytes are immediately allocated.

Native Socket traffic, IB management, physical RDMA and Kubernetes workloads are
separate execution paths. A shared NetBox manifest does not couple their payload
traffic or failures. NCCL Socket deliberately disables P2P/SHM/IB/NVLS/CollNet/MNNVL
bypasses. Do not enable them and claim traffic follows the namespace graph.

Not implemented: calibrated NVIDIA system performance, GPU/NIC/NUMA/rail placement,
ECMP/adaptive routing, packet-level congestion/ECN/PFC/IB credits, working storage
and power/thermal/BMC state machines, or whole-host native resource admission.
These require new models and hardware calibration; documentation or passing
portable tests cannot close them. See [audit remediation](audit-6311-remediation.md).
