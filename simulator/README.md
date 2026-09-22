# Rust datacenter simulator

A Rust datacenter control plane based on the [NVIDIA Air SDK specification snapshot](https://github.com/QuasarRay/nv-air-sdk/tree/0a9ce86a6195e26c5522640d895a1adba9449cbd), with a patchbay Linux network fabric and native NCCL execution. The Python API is a specification only: there is no Python runtime or Rust–Python bridge.

**Every collective executes NCCL.** All-reduce, broadcast, all-gather and reduce-scatter invoke the pinned project's native Rust bindings and CUDA kernels. There is no CPU collective, emulated ring, software-verbs substitute, or automatic fallback. A build without native NCCL returns `Unsupported`, including for empty buffers and single-rank requests.

## Control plane and CPU-only validation

```sh
git submodule update --init --recursive
cd simulator
cargo test --locked --all-targets
cargo run --locked -- json
```

The JSON-lines session accepts commands such as `{"operation":"create","name":"my-datacenter"}` and `{"operation":"list"}`. [Command](src/command.rs) is the full command reference. The local [Simulator](src/api.rs) and [Simulation](src/model.rs) APIs manage lifecycle, nodes, interfaces, links, configuration, services and checkpoints. Export/import preserves topology; sessions and runtime checkpoints remain in memory.

Default builds provide these control-plane operations and explicitly labeled topology/NIC models. They cannot run GPU scenarios. `cargo run -- run examples/gpu-spine-leaf.json` without `--features nccl` exits with an error and no fabricated result.

## Run actual NCCL through the fabric

Requirements: Linux namespace privileges, a CUDA toolkit compatible with the pinned NCCL source, an NVIDIA driver, one distinct visible CUDA GPU per rank, and `ip`, `tc`, `nft`, `sysctl` and `ping`. Install the Linux native build dependencies listed below. Build **the pinned NCCL fork**, then link and load that build:

```sh
make -C vendor/nccl -j2 src.build
export NCCL_LIB_DIR="$PWD/vendor/nccl/build/lib"
export LD_LIBRARY_PATH="$NCCL_LIB_DIR${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
cargo build --locked --features nccl --bin datacenter-simulator --example nccl_runtime
./target/debug/datacenter-simulator run examples/gpu-spine-leaf.json --devices 0,1
```

The sample has two GPU hosts joined through two leaves and a spine. Each rank runs as its own Rust process in its node's patchbay network namespace. Rank zero creates the real NCCL bootstrap ID **inside that namespace**. Workers retain their GPU's CUDA primary context, allocate device buffers, copy inputs to the GPU, initialize NCCL, submit the requested native collective, synchronize the CUDA stream, and copy results back.

NCCL's `Socket` transport sends real TCP/IP packets through the configured links. Each node has a dedicated `simnccl` /32 endpoint; that interface name is reserved. Static routes come from petgraph and per-link kernel `tc netem` applies latency and bandwidth. P2P, shared-memory, NVLS, CollNet and IB transports are disabled for this namespace execution mode so collective traffic uses the modeled fabric. NCCL chooses its own Ring/Tree algorithms. The simulator does not implement those algorithms. Worker-only environment overrides also keep inherited NCCL settings from selecting another transport; CUDA device visibility is inherited.

Results contain GPU output buffers, the NCCL version, `backend: "nccl"`, `transport: "Socket"`, measured wall times, and actual data-interface TX/RX counters. Counters include bootstrap/control traffic. They are not an NCCL packet trace. There are no synthetic collective timestamps, and execution does not advance the model's virtual clock.

The `run` command defaults to CUDA ordinals `0..nranks`; `--devices` specifies rank placement. JSON collective commands accept an optional `execution` object:

```json
{"operation":"all_reduce","simulation":"<id>","ranks":["<gpu1-id>","<gpu2-id>"],"inputs":[[1,2],[3,4]],"reduction":"sum","execution":{"devices":[0,1],"timeout_secs":60}}
```

All five reductions (`sum`, `product`, `min`, `max`, `average`) map directly to NCCL operations. Broadcast accepts a root index and its input. All-gather concatenates through NCCL; reduce-scatter invokes `ncclReduceScatter` directly and requires a divisible input count. Counts must match between ranks. Inputs must be finite, at most 16 MiB per rank, with aggregate input and output budgets of 64 MiB each. The timeout is 1–3600 seconds, default 60, covering worker startup, bootstrap, execution and shutdown. Failed or timed-out workers are killed and reaped; partial outputs are not returned. Namespace construction occurs before that deadline.

Library users call `Simulation::collective` with `Collective` and `NcclOptions`, or the four convenience methods with default options. Initialize `patchbay::init_userns()` before creating any threads. Set `DATACENTER_SIMULATOR_WORKER` to the absolute path of the native `datacenter-simulator` executable when embedding in another program. Async callers can retain a `LinuxFabric` and use `fabric.collective(..., worker_path).await`; that API executes through the existing live fabric and accepts explicit worker placement. Do not use the synchronous convenience API inside a Tokio runtime.

## Other backends

| Backend | Behavior | Requirements |
|---|---|---|
| Default control/model APIs | Resource state, petgraph route/serialization estimates, checked software NIC memory/queue model | Rust |
| `linux` | Real Linux TCP/UDP/IP, forwarding, bandwidth/latency conditions and link faults | Linux namespace support and networking tools |
| `rdma` | Real rust-ibverbs RC send/receive, memory registration and CQ polling | libibverbs and RNIC or SoftRoCE |
| `nccl` (includes `linux`) | Actual CUDA/NCCL collectives over patchbay TCP/IP | CUDA GPUs, driver and matching libnccl |
| `nccl-check` | Compile all native paths without linking NCCL; collective execution is rejected | Native build dependencies, no CUDA required |

```sh
sudo apt-get install nftables iproute2 iputils-ping clang libclang-dev cmake \
  pkg-config libibverbs-dev librdmacm-dev libnl-3-dev libnl-route-3-dev libudev-dev
cargo check --locked --all-targets --features linux,rdma,nccl-check
cargo build --locked --examples --features linux,rdma
sudo ./target/debug/examples/linux_fabric
./target/debug/examples/rdma_loopback
```

The software `Fabric` and `Simulation::transfer` APIs are analytical NIC/topology models, separate from native protocol execution. NCCL never calls them. The Linux backend supports at most 240 links and unsplit data interfaces. Hosts do not forward, OOB/PCIe cannot provide data shortcuts, and helper bridges have no shared uplink. A `LinuxFabric` owns a topology snapshot; change live links through its `set_link_up` method. Dropping its handles releases the lab.

## API boundaries and validation

See [API coverage](docs/api-coverage.md) for supported Air concepts and local extensions. This does not boot VM images, execute guest shell/ZTP/cloud-init, publish hosted Air service tunnels, implement dynamic BGP convergence, or emulate CUDA/RNIC hardware. Init/file instructions only change modeled guest state. The existing Containerlab/FRR environment remains available separately.

The simulator also includes a [Rust NetBox OpenAPI-to-protobuf generator](docs/netbox-openapi-proto.md). It converts NetBox's committed machine-readable `contrib/openapi.json` contract into app-grouped `.proto` services and component messages, with a scheduled workflow that opens a refresh PR when upstream changes.\n\nAll upstreams and registry dependencies are pinned in [upstreams.json](upstreams.json), submodules and Cargo.lock. [Rust CI](../.github/workflows/rust-simulator.yml) checks control/model contracts, the absence of CPU collective fallbacks, native compilation, and real kernel TCP delivery, partition and recovery.

The manual [NCCL runtime workflow](../.github/workflows/nccl-runtime.yml) builds the pinned NCCL source on a self-hosted Linux runner labeled `nccl` with at least two GPUs. It checks all collectives and reductions, empty and single-rank calls, actual interface traffic, missing-device failure, a kernel-only link partition and recovery after worker termination, plus the scenario CLI. Run it locally with:

```sh
./target/debug/examples/nccl_runtime "$PWD/target/debug/datacenter-simulator"
```

CPU-only compile/contract tests do not establish CUDA execution. The hardware gate fails if prerequisites are absent; it does not report skipped devices as success. Actual CUDA execution requires a configured GPU runner. The separate [RDMA runtime workflow](../.github/workflows/rdma-runtime.yml) similarly requires a real RDMA or SoftRoCE device. These runtime gates remain distinct from hosted CI. Kernel timing and physical GPU execution are not calibrated datacenter performance predictions.

## DeepOps Slurm guests and upstream NCCL tests

The optional `vm` feature boots full Linux guests on the modeled fabric and
supervises the pinned NVIDIA DeepOps deployment and native MPI nccl-tests.
See [the DeepOps integration guide](../integrations/deepops/README.md) for
hardware requirements, configuration, report validation, and the separate
CPU VM smoke and GPU deployment workflows. No Rust–Python FFI is introduced.
