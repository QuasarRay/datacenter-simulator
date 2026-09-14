# Design and validation

## Topology and runtime ownership

Simulator owns local Simulation objects. All resource identifiers resolve within that simulation. Mutation methods preflight names, endpoints, resource values, relationships and state; nested resources expose immutable getters. JSON manifests import into a temporary Simulator and commit only after all validation and optional startup succeed.

petgraph stores a separate edge per physical link and direction. Data routes exclude PCIe and OOB links. Host nodes can originate and receive traffic but cannot forward a path. A directed-link timestamp tracks serialization occupancy; propagation does not occupy the transmitter. Routes use unloaded cost and are recomputed on each operation; there is no dynamic routing protocol or ECMP distribution model.

A collective stages its trace on a cloned simulation, performs the complete reduction schedule, then commits. Numeric overflow or a missing route leaves the source simulation unchanged. Software verbs validate memory ranges, generation, protection domain, access key, queue state and receiver capacity before copying bytes or advancing time. QPs and memory belong to one simulation and one node generation.

## Kernel topology

LinuxFabric owns a patchbay lab plus a snapshot of the portable simulation. Every modeled node becomes a Device namespace. Every link receives its own bridge behind a patchbay router. The routers' IX uplinks are disabled before the lab becomes available, and automatic device default routes are removed. There is no implicit exchange path between the physical links.

The backend installs /32 loopback routes with a dedicated route protocol number. Next hops are selected from the same petgraph paths as the model. Forwarding is enabled only on modeled switches. Live link changes toggle both kernel endpoints and recompute owned routes. A kernel reconfiguration error is returned to the caller; discard the lab on such an error because kernel operations are not a database transaction.

The pinned patchbay allocator assigns gateway host octets within /24 networks, and its IX pool bounds the Linux adapter to 240 links. Each link receives a distinct /24 with exactly its two endpoint interfaces. Each directed egress receives a tc netem condition. The link helper bridge forwards at layer 2, adding no modeled IP router hop. Namespace networking shares the host kernel and filesystem, so it does not execute arbitrary guest shell instructions or claim to boot node OS images. Application-owned Device clones keep their namespace handles alive and must be dropped during cleanup.

## Native execution boundaries

The rust-ibverbs adapter executes an actual RC loopback with a registered memory region, posted receive, signaled send and both work completions. Send and receive regions are disjoint. A deadline prevents infinite polling, and the QP is dropped before its registered memory on failure. The returned payload is checked byte-for-byte.

NcclRank validates rank placement against a running model, then creates an actual native communicator. Native all-reduce and broadcast use typed CUDA pointers with an explicit unsafe contract. It does not claim host buffers are CUDA buffers or that model bandwidth controls a physical NCCL transport. NCCL's no-link feature is confined to compile-check use.

The portable ring algorithm follows the reduce-scatter/all-gather phases of [the pinned NCCL runRing implementation](https://github.com/QuasarRay/nccl/blob/16b55c792f902080c38820c9c2bd766337909da3/src/device/all_reduce.h). It uses f64 values and mathematical reduction semantics; this is not a reproduction of NCCL CUDA kernel scheduling, GPU topology tuning, floating-point accumulation order, LL/LL128 packet layout, PFC/ECN or wire-level InfiniBand.

## Tests

| Risk | Check |
|---|---|
| Partial resource mutation or leaked import | lifecycle, bulk-validation and transactional import regression tests |
| Cross-node endpoints and duplicate links | ownership, endpoint reuse, breakout and cascade tests |
| Unexpected forwarding | host-transit rejection, OOB partition and parallel-edge reroute tests |
| Wrong bandwidth/time units or duplex behavior | exact serialization and shared-link contention assertions |
| Wrong collective values, uneven chunks, single rank, empty buffer | 96 generated ring cases, each checking five reductions against an independent scalar oracle |
| Failed collective advancing time | disconnected-rank rollback test |
| Guest configuration state lost or shared by clone | checkpoint/rebuild/clone assertions with fresh IDs |
| Memory corruption, stale registrations or unauthorized remote writes | verbs range, key, domain, queue exhaustion and generation tests; 96 generated invalid ranges |
| Real Linux data path or hidden bypass | TCP payload, partition failure and link recovery in linux_fabric |
| Native API drift | compile linux + rdma + nccl-check against pinned source revisions |
| Actual RDMA work submission | Manual rdma-runtime workflow on a configured RDMA runner; not executed successfully in this environment |
| Existing project regression | separate Dagger/Containerlab/EX457 CI on the pull request |

No finite test suite guarantees complete correctness. The hosted Azure kernel omits SoftRoCE even in its matching extra module package. Linux networking and native adapter compilation are required hosted CI gates; actual RDMA execution is a separate explicit hardware gate, with no unavailable-device success path.

Native CUDA runtime execution, actual RDMA/SoftRoCE execution, real RNIC hardware, large-scale performance and timing calibration need hardware validation beyond these checks.
