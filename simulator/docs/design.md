# Design and validation

## Control plane and analytical models

Simulator owns local Simulation objects. Resource mutation preflights identifiers, ownership and state. Imports stage validation before committing. petgraph keeps a separate edge per link and direction and excludes OOB/PCIe and host transit from data routes. Topology serialization estimates and the checked software NIC model remain explicit analytical APIs. Neither computes or transports NCCL results.

## Real NCCL execution

The collective module only validates rank placement, finite inputs, shapes, device assignment and allocation limits. It contains no collective arithmetic or synthetic transfer scheduling. The former CPU ring, broadcast replication, all-gather concatenation and all-reduce/slicing implementation of reduce-scatter have been removed. All public collective entry points, JSON commands and the scenario CLI use one native execution path. Missing native support returns an error even for zero elements or one rank.

A LinuxFabric owns patchbay namespaces and its topology snapshot. Every physical link has an isolated bridge, disabled IX uplink and directed tc netem settings. Only switch nodes forward. Each node's simnccl interface holds a /32 endpoint, reached through petgraph-derived static routes. The same links carry real NCCL Socket traffic. There is no host-network bootstrap shortcut: rank zero obtains its NCCL unique ID in its own namespace.

One worker process owns one CUDA device and NCCL communicator. All workers report CUDA readiness before receiving the shared bootstrap ID. The CUDA Driver ABI retains the primary context shared with the CUDA runtime, allocates device memory, uploads inputs, and owns a stream. The four native calls are ncclAllReduce, ncclBroadcast, ncclAllGather and ncclReduceScatter through the pinned Rust bindings. GPU results are read only after stream synchronization and an asynchronous-error check. No host arithmetic supplies a result.

The supervisor overrides NCCL settings only in child environments, selecting Socket and the exact simnccl interface, and disables P2P/SHM/NVLS/CollNet/MNNVL/IB transport bypasses. NCCL chooses a native Ring/Tree algorithm. This implements NCCL over the Linux fabric, not NCCL over the analytical software verbs model. Physical RDMA submission remains a separate rust-ibverbs adapter.

Worker input/output pipes only carry job configuration, the bootstrap ID and host input/result arrays. Inter-rank collective communication is exclusively NCCL's TCP/IP traffic. NCCL diagnostics go to stderr so the JSON command stream stays parseable. Results expose wall times and kernel counters, including bootstrap traffic; there are no virtual collective traces or changes to model time.

The deadline bounds startup, bootstrap, submission, synchronization and teardown. On failure or timeout the controller kills and reaps remaining ranks before releasing the lab. An operation failure aborts the communicator before GPU allocations are released. If native abort fails the worker exits without destructing potentially live allocations; process teardown belongs to the CUDA driver. Cancellation also requests process termination via kill-on-drop. Successful calls destroy the communicator after synchronization and before buffers/streams/context are dropped.

## Validation

| Risk | Check |
|---|---|
| Partial control-plane mutation or leaked import | Existing lifecycle, ownership and transactional import regressions |
| Hidden forwarding path | Host-transit rejection, OOB partition, parallel-edge reroute and real Linux TCP partition/recovery |
| Wrong analytical serialization or memory protection | Existing link contention and software verbs domain/key/generation/range tests |
| CPU fallback reintroduced | Default and nccl-check builds reject every collective API, JSON command and scenario CLI; empty/single-rank paths also reject |
| Invalid shapes/unsafe allocations | Equal counts, root bounds, finite values, reduce-scatter divisibility, aggregate memory and unique device validation |
| Native API mismatch | Compile all targets with linux, rdma and nccl-check against pinned forks |
| Wrong CUDA/NCCL values | Manual nccl-runtime gate on two GPUs: every collective and reduction, uneven all-reduce count, empty and single-rank calls |
| Namespace bypass or leaked rank after failure | Hardware gate checks real interface counters, cuts only a kernel link while keeping the graph connected, requires failure and then successful recovery |
| Missing GPU accepted | Hardware gate requires an unavailable device ordinal to error |
| Actual RDMA work submission | Separate manual rdma-runtime gate on a configured RDMA runner |

Native compilation does not validate GPU execution. Hosted CPU runners validate contracts and Linux networking; actual NCCL and RDMA need their explicit hardware gates. Namespace wall time is affected by host scheduling and is not a calibrated GPU datacenter performance model.

## Upstream specifications

- [Pinned NCCL Rust bindings](https://github.com/QuasarRay/nccl/tree/16b55c792f902080c38820c9c2bd766337909da3/bindings/rust)
- [NCCL communicator creation](https://docs.nvidia.com/deeplearning/nccl/user-guide/docs/usage/communicators.html)
- [NCCL transport configuration](https://docs.nvidia.com/deeplearning/nccl/user-guide/docs/env.html)
- [CUDA primary contexts](https://docs.nvidia.com/cuda/cuda-driver-api/group__CUDA__PRIMARY__CTX.html)
- [CUDA memory](https://docs.nvidia.com/cuda/cuda-driver-api/group__CUDA__MEM.html) and [streams](https://docs.nvidia.com/cuda/cuda-driver-api/group__CUDA__STREAM.html)
