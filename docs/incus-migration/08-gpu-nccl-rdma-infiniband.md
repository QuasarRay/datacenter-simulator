> Implementation authorized by the project owner on 2026-09-24. The former implementation prohibition is withdrawn. See integrations/incus for implemented behavior and validation boundaries.

# GPU, NCCL, RDMA and InfiniBand migration

## Preserve distinct fidelity claims

| Path | Meaning after migration | Evidence still required |
|---|---|---|
| Portable graph/message model | Analytical timing and lifecycle contracts | Deterministic model tests; no native-performance claim |
| `fabric.rs` software verbs | QP/MR/range/key/ordering model | Model state/completion tests; no RNIC claim |
| Real CUDA/NCCL Socket | GPU computation with TCP payload over the declared Ethernet fabric | Hardware correctness, interface counters and physical cut/recovery |
| ibsim/OpenSM | Real upstream IB management program and MAD behavior | Discovery, LIDs, routes, link changes and clean process lifecycle |
| Physical RDMA | Actual RNIC/kernel RDMA verbs | Device/port/GID selection and real completion/error evidence |
| Mokka | Kubernetes/NVML software contracts | Upstream checks and pod/device metadata; no CUDA/verbs claim |

Incus does not turn one row into another. The current ibsim integration has no general verbs payload data plane. Sharing a node name or graph does not put NCCL data through ibsim.

## Native GPU allocation

The current QEMU path dedicates devices through VFIO. System containers instead share the host kernel/driver and receive controlled device visibility. Port host NVIDIA driver installation and compatibility checks into a **host role**; install only matching user-space libraries/toolkits in compute nodes. Do not run guest kernel-driver installation or VFIO binding logic unchanged inside Incus.

Identify GPUs by stable UUID/PCI identity rather than a transient ordinal. Define exclusive assignment or deliberate sharing separately; container visibility does not imply exclusive GPU memory/compute isolation. Preserve the current one-distinct-physical-GPU-per-rank requirement for native acceptance. A MIG instance or MPS sharing mode needs its own supported NCCL/peer-communication qualification and accounting. CPU-only execution must fail native CUDA requirements, not substitute host arithmetic.

Qualify Incus GPU device access and native libraries against the pinned host driver, cgroup device policy and runtime. Record real device identities from both host and guest. Reject unexpected extra devices and duplicate rank assignment. Persist library absolute paths/hashes **as seen by the worker**; host and container paths differ. Tests must exercise missing device, wrong library, invalid input and timeout as well as success.

## NCCL worker migration

1. Preserve `collective.rs` validation, CUDA context/buffer/stream lifetimes and native-only result policy. Compile the worker for the image ABI and verify its executable identity after transfer.
2. Launch it through bounded Incus exec or an explicitly qualified process boundary inside the container. Entering only its network namespace while using host rootfs/cgroup/device visibility does not establish full container execution; label such a test accurately.
3. Put `simnccl` or the selected data interface in the **actual worker network namespace**. Bind rank-zero bootstrap and data communication there. Set the Socket transport/interface policy in child environments, retaining the bypass exclusions for IB, shared memory, P2P, NVLS, CollNet and MNNVL used by the existing gate.
4. Treat host control pipes/API streams as configuration/result transport only. They must not carry inter-rank payloads. Verify all workers report readiness before bootstrap and preserve deadlines across startup, submission, synchronization and cleanup.
5. Validate numerical outputs for every supported operation/root/reduction/shape. Preserve empty/single-rank handling and asynchronous error checks. Record actual hardware execution; `nccl-check` compilation is not execution evidence.
6. Cut the data path in the kernel while graph intent remains connected; require timeout/failure, reap workers and restore/retry. Record unreaped processes or retained GPU state as failure, with cleanup bounds.

Native timings remain host scheduling and TCP/qdisc measurements. They do not predict DGX, NVLink, PCIe, rail/NUMA or Spectrum performance without a calibrated model and hardware comparison.

## InfiniBand management

Keep the pinned ibsim/umad2sim/OpenSM native artifacts and scoped preload/environment. Either run a dedicated management service container or a bounded host process under the adapter, recording where it executes; ordinary Ethernet veth wiring is not an IB cable. Map graph HCAs/switch ports explicitly to ibsim topology. Preserve disabled links, management client attachment rules, startup deadlines, subnet-manager readiness and shutdown.

When an instructor changes a common physical-intent link, translate it to the appropriate backend operation and verify each affected plane. Do not announce cross-plane fault coupling until both acknowledgements and behavioral checks succeed. If only IB management is affected, evidence must say so. Stop OpenSM/ibsim and remove their owned sockets/preloads on cancellation; never install a global `LD_PRELOAD`.

## Physical RDMA and storage fabric

Pass only the intended RDMA devices/ports through a qualified Incus device policy; check userspace rdma-core ABI, memory locking, device permissions, GID index, link state and isolation. Keep Soft-RoCE/software tests distinct from physical NIC tests. RNIC DMA, registration lifetime, completion errors and memory protection must still be observed using the existing native adapter.

A physical RNIC may bypass patchbay's Ethernet veth/netem path. Do not report its performance or failure as graph-controlled without evidence at the actual network boundary. GPUDirect RDMA additionally needs compatible GPU/RNIC topology, drivers and peer-memory/DMA support; Incus device visibility alone proves none of these. PFC, ECN, congestion control, IB credit flow, NVLink/NVSwitch and storage performance remain explicit modeling/hardware work.

## Acceptance and cleanup

Keep separate trusted runners for CPU/namespace contracts, native CUDA/NCCL, physical RDMA and Slurm GPU tests. Record image fingerprint, host kernel/driver, native library identity, GPU/RNIC allocation, graph hash and execution commit in each report. No stale pre-migration evidence can satisfy the new runtime gate. Release devices and processes before removing container/fabric resources.

Sources: baseline `simulator/docs/{design,intended-state}.md`, [NCCL transport settings](https://docs.nvidia.com/deeplearning/nccl/user-guide/docs/env.html), [Incus GPU devices](https://linuxcontainers.org/incus/docs/main/reference/devices_gpu/), [Incus InfiniBand devices](https://linuxcontainers.org/incus/docs/main/reference/devices_infiniband/), and the pinned native dependencies in `simulator/upstreams.json`.
