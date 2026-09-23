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
| Wrong CUDA/NCCL values | Trusted main/scheduled/manual nccl-runtime gate on two GPUs: every collective and reduction, uneven all-reduce count, empty and single-rank calls |
| Namespace bypass or leaked rank after failure | Hardware gate checks real interface counters, cuts only a kernel link while keeping the graph connected, requires failure and then successful recovery |
| Missing GPU accepted | Hardware gate requires an unavailable device ordinal to error |
| Actual RDMA work submission | Separate trusted main/scheduled/manual rdma-runtime gate on a configured RDMA runner |

Native compilation does not validate GPU execution. Hosted CPU runners validate contracts and Linux networking; actual NCCL and RDMA need their explicit hardware gates. Namespace wall time is affected by host scheduling and is not a calibrated GPU datacenter performance model.

## Upstream specifications

- [Pinned NCCL Rust bindings](https://github.com/QuasarRay/nccl/tree/16b55c792f902080c38820c9c2bd766337909da3/bindings/rust)
- [NCCL communicator creation](https://docs.nvidia.com/deeplearning/nccl/user-guide/docs/usage/communicators.html)
- [NCCL transport configuration](https://docs.nvidia.com/deeplearning/nccl/user-guide/docs/env.html)
- [CUDA primary contexts](https://docs.nvidia.com/cuda/cuda-driver-api/group__CUDA__PRIMARY__CTX.html)
- [CUDA memory](https://docs.nvidia.com/cuda/cuda-driver-api/group__CUDA__MEM.html) and [streams](https://docs.nvidia.com/cuda/cuda-driver-api/group__CUDA__STREAM.html)

## Audit corrections (2026-09-22)

Analytical submissions must have nondecreasing timestamps, including concurrent
submissions at equal times. An older submission is rejected atomically. Reset,
shutdown and rebuild cancel reservations and reset this submission frontier to
the completed model clock. This API returns reservations immediately; it is not
an event queue that can retroactively reorder previously returned traces.

Both analytical and Linux routing minimize total configured latency, breaking
ties by hop count, independently of transfer size. Hosts never forward transit.
A single graph is compiled for each native route update, then one shortest-path
search runs per source. Rank admission uses switch components and explicit direct
host links rather than reconstructing every pair's path.

Checkpoints contain an exact configuration snapshot and instruction states.
Restore and clone reject any changed nodes, interfaces, links, resources,
services, instructions or guest configuration. New instructions are never
fabricated as COMPLETE. Management leases persist across node insertions and
clones. Interface MACs are allocated uniquely within each simulation.

Native links use MTU 1500 and a netem queue of
`2 * ceil(bandwidth_bps * latency_ns / (8 * 1e9 * 1500)) + 1024` packets.
Unrepresentable queue capacities fail before native setup. This bounds the queue
by two one-way bandwidth-delay products plus a burst allowance. Analytical
queues are lossless and unbounded. Neither backend promises PFC/ECN, jitter,
packet-loss injection or RDMA congestion fidelity.

Native link changes restore both endpoints and routes on failure. A failed
rollback poisons the fabric, so traffic operations fail until teardown. VM
partition guards also restore links on error or cancellation. NCCL failure
cleanup gets two seconds beyond the execution deadline and reports unreaped
PIDs instead of waiting forever. Such PIDs can retain GPU resources until the
kernel releases them. Interface counters include unrelated concurrent traffic;
results explicitly label that accounting scope.

JSON commands have a separate 1 MiB wire limit, including their newline, while
Rust collective buffers retain their existing 16 MiB/rank and 64 MiB aggregate
limits. Oversized or invalid UTF-8 commands return structured errors and the
session continues. Local manifest reads stop at 16 MiB. Sessions default to 128
simulations (`Simulator::with_capacity_limit` changes this); event history retains
the newest 10,000 entries (`set_history_limit` changes this).

Hardware workflows run on trusted main pushes and weekly, as well as manual
requests. Set the explicit RDMA device/port/GID repository variables and the
runner-local `DEEPOPS_GPU_CONFIG` path. Never run untrusted fork PRs on these
privileged runners. The reusable `hardware-evidence.yml` promotion gate requires
successful runs of all three hardware workflows for the exact commit within
seven days. Maintainers must make it required in their promotion/branch policy;
this change cannot manufacture hardware runs or alter repository protection.

The audit's real InfiniBand payload gap remains: ibsim provides management MADs,
not a verbs data plane. Socket NCCL, local RNIC loopback and Mokka results do not
close that gap. A true emulated IB payload backend and an attached GPU/RNIC lab
are prerequisites for an NCCL-IB/GPUDirect claim.
