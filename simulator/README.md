# Rust datacenter simulator

A local Rust library and executable implementing datacenter behavior from the [NVIDIA Air SDK specification snapshot](https://github.com/QuasarRay/nv-air-sdk/tree/0a9ce86a6195e26c5522640d895a1adba9449cbd). It includes a portable traffic/NIC model, real Linux namespace topology, and native NCCL and RDMA adapters.

The Python SDK supplies API concepts and payload requirements. There is no Python runtime, PyO3, Cython, embedding, or Rust–Python bridge in this implementation. Native NCCL and libibverbs retain their existing C ABI behind the requested Rust libraries.

## Build and run

From the repository root:

```sh
git submodule update --init --recursive
cd simulator
cargo test --locked --all-targets
cargo run --locked -- run examples/gpu-spine-leaf.json
```

The example creates two GPU host models attached through two leaves and a spine, starts the simulation, and runs ring all-reduce. Output contains the actual reduced buffers and a timestamped transfer trace. The portable mode needs no CUDA, RNIC, root privileges, or network namespaces.

For a persistent in-process control session, run `cargo run --locked -- json`. Each input line is a command; each output is a JSON success/result or error. Use returned resource IDs in subsequent commands:

```json
{"operation":"create","name":"my-datacenter"}
{"operation":"list"}
```

The Rust [Command enum](src/command.rs) is the command reference. [Simulator](src/api.rs), [Simulation](src/model.rs), and [Fabric](src/fabric.rs) also expose typed library APIs. The session is in memory; export a manifest for persistent topology storage and import it into another session. Runtime checkpoints are in-memory snapshots.

## Execution modes

| Mode | Implementation | Executes | Requirements |
|---|---|---|---|
| Portable (default) | petgraph + Rust traffic, collective, and verbs models | Reduction arithmetic, routing, directed-link queuing, registered-buffer copies, work completions, lifecycle and configuration state | Rust |
| Linux (`linux`) | patchbay devices and isolated link bridges; petgraph routes | Kernel TCP/UDP/IP and real packet forwarding in namespaces; link faults and recovery | Linux, namespace privileges, `ip`, `tc`, `nft`, `sysctl`, `ping` |
| RDMA (`rdma`) | rust-ibverbs | Actual RC queue pair, MR registration, send/receive and CQ polling with a deadline | libibverbs, clang, cmake, RDMA device or SoftRoCE |
| NCCL (`nccl`) | NCCL Rust bindings | Native communicator setup, all-reduce and broadcast on application-owned CUDA buffers | Matching libnccl, CUDA GPUs/runtime, rank/device placement |
| NCCL compile check (`nccl-check`) | Upstream `no-link` feature | Compilation only | No CUDA library required; cannot execute native calls |

Each native adapter is explicit. A failed native operation returns an error. It does not switch to the portable model.

```sh
# Debian/Ubuntu dependencies for Linux and RDMA builds:
sudo apt-get install nftables iproute2 iputils-ping clang libclang-dev cmake \
  pkg-config libibverbs-dev librdmacm-dev libnl-3-dev libnl-route-3-dev libudev-dev

cargo build --locked --examples --features linux,rdma
sudo ./target/debug/examples/linux_fabric
./target/debug/examples/rdma_loopback

# Compile all adapters on a host without CUDA:
cargo check --locked --all-targets --features linux,rdma,nccl-check
```

The Linux adapter currently supports at most 240 links (the pinned patchbay IX address pool) and unsplit data interfaces. The Linux example verifies an actual TCP payload, partitions a modeled link, requires connectivity to fail, restores the link, and verifies connectivity again. Application code can use `LinuxFabric::device` to run Rust futures/sockets in any node namespace. Call `patchbay::init_userns()` before creating a runtime or other threads. The Linux handle owns a snapshot of the active portable topology; mutate its live link state through `set_link_up`. Dropping its handles releases the lab. Changes to a separate `Simulation` object do not reconfigure an existing Linux lab.

The native RDMA example errors when no RDMA device exists. For SoftRoCE, use an isolated test interface and `rdma link add <name> type rxe netdev <interface>`. It never registers ordinary Rust heap pointers with NCCL. NCCL accepts CUDA-accessible pointers under an explicit unsafe contract; the caller must set the device, coordinate all ranks, and synchronize streams before freeing memory. NCCL uses the actual host/CUDA network configuration, not the portable model's predicted routing.

## Model behavior

- Graph edges represent individual full-duplex physical data links, including parallel links. Hosts do not forward transit traffic. OOB and PCIe interfaces do not become data-plane shortcuts.
- Routes minimize unloaded serialization plus propagation cost. Traffic is store-and-forward and FIFO per directed link. Concurrent submissions at the same virtual time contend for that link; opposite directions are independent.
- Link bandwidth is bits/second; latency and simulation timestamps are nanoseconds. Portable serialization uses integer ceiling arithmetic. Linux uses `tc netem` with fractional microsecond delays; actual timing is bounded by kernel scheduling and machine performance.
- Ring all-reduce uses reduce-scatter and all-gather rounds, including uneven and empty chunks. Independent scalar-oracle property tests cover sum, product, min, max and average. Broadcast and all-gather are also modeled. Reduce-scatter is a semantic composition of all-reduce and slicing, and reports that full traffic cost.
- NIC memory has a per-node protection domain, generation checks, remote-write permission and keys, bounded posted-receive/completion queues, and checked ranges. Reset or shutdown invalidates registrations and queue pairs.
- Topology mutations require INACTIVE. Failed imports, invalid bulk operations, failed collective scheduling and invalid verbs requests do not commit partial state.
- Checkpoints capture modeled guest files/hostname. Init/file instructions modify this virtual state only. They never write guest paths into the host filesystem.

## Fidelity and API coverage

This is a local datacenter simulator, not a replacement for the entire NVIDIA hosted Air platform. [API coverage](docs/api-coverage.md) distinguishes implemented local operations, model extensions, and unavailable cloud/guest capabilities. In particular, this does not boot qcow2 images, implement CUDA kernels, emulate GPU compute, implement InfiniBand link-layer signaling/PFC/ECN, or simulate BGP convergence. NCCL collective schedules are not a generic routing-protocol implementation. The existing Containerlab/FRR lab remains the runnable BGP/EX457 environment.

ZTP and cloud-init content can be stored and exported, but starting with executable guest configuration returns Unsupported because neither the portable model nor network namespaces provide a guest OS/filesystem. Shell instructions also return Unsupported. Service resources are immutable descriptors; no NVIDIA Air external tunnel or worker port is fabricated. Image names and image metadata identify profiles rather than booting an operating system.

Portable operations cap node count at 4096, a transfer at 16 MiB, aggregate collective buffers at 64 MiB, and a collective trace at 65,536 transfers/hops. Requests exceeding a limit return an error. Portable resource budgets are descriptive totals, not CPU/RAM/storage enforcement. Timing is a topology/serialization model, not calibrated NVIDIA hardware performance. Linux tests establish connectivity and isolation, not nanosecond accuracy. SDK REST paths, authentication, legacy aliases, HTTP pagination, and hosted organization/publishing/training workflows are outside the local Rust contract.

## Reproducibility and validation

[upstreams.json](upstreams.json) and git submodule commits pin all four implementation projects and the SDK specification revision. Cargo.lock pins registry dependencies. The [Rust CI workflow](../.github/workflows/rust-simulator.yml) runs the portable suite, the scenario, Clippy, all-adapter compilation, and real Linux/SoftRoCE checks. The existing Dagger/EX457 workflow runs independently on the PR.

Native NCCL runtime validation still requires a CUDA runner and compatible libnccl. A successful `nccl-check` job only establishes API compilation. See [design and validation](docs/design.md) for the execution boundaries and test mapping.
