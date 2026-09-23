# Placement and transport

## FID-01 — Flat host ranks cannot describe a DGX server

**High · Gap · Existing G003.** [simulator/src/model.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/model.rs#L94) has host/switch roles, generic resource totals and free-form labels. [simulator/src/collective.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/collective.rs#L176) rejects multiple ranks on the same modeled node; `NcclOptions::devices_for` requires distinct CUDA ordinals. There is no typed GPU-to-host, PCIe-root, NUMA, NIC, NVLink/NVSwitch or rail mapping consumed by routing.

NVIDIA's [DGX H100/H200 guide](https://docs.nvidia.com/dgx/dgxh100-user-guide/introduction-to-dgxh100.html) describes eight GPUs within one server and an NVSwitch interconnect. Its [SuperPOD architecture](https://docs.nvidia.com/dgx-superpod/reference-architecture-scalable-infrastructure-h100/latest/network-fabrics.html) describes rail-aligned compute connectivity. Mapping every GPU to a separate flat host therefore loses intra-server communication and GPU/NIC affinity. Labels alone do not supply causal behavior.

**Counterexample:** a job with eight ranks on one DGX cannot be expressed as eight ranks on one host; splitting it into eight hosts changes the network graph. The physical CUDA topology remains that of the execution machine. Distinct ordinals also do not by themselves establish distinct physical parent GPUs under every partitioning/visibility arrangement.

**Remedy and acceptance:** introduce explicit host/device attachment and rank placement, including units and sharing rules. An eight-GPU host fixture must preserve local communication versus inter-host traffic, and a two-rail fixture must select the intended NICs. Keep a compact analytical representation for large logical fleets; validate native subsets against actual topology. Do not relax uniqueness checks without replacing their safety invariant.

## FID-02 — Socket execution is a narrow, intentional transport experiment

**High · Gap · Existing G001.** [simulator/src/nccl_backend.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/nccl_backend.rs#L113) and [integrations/deepops/files/nccl-rank.sh](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/integrations/deepops/files/nccl-rank.sh#L12) force Socket, disable IB/P2P/SHM/NVLS/CollNet/MNNVL and restrict algorithms to Ring/Tree. These controls prevent bypassing the namespace graph; they are useful and should remain explicit.

[NVIDIA's NCCL documentation](https://docs.nvidia.com/deeplearning/nccl/user-guide/docs/env.html) distinguishes P2P over NVLink/PCIe, shared-memory and network transports, and describes NIC/rail selection. The current results cannot stand in for GPUDirect RDMA, NVLink collectives, hardware offload or production rail performance. Setting a manifest link to 400 Gb/s does not create that hardware capability.

**Remedy and acceptance:** attach a machine-readable capability/model scope to every result and comparison. Add separately validated native transport profiles only where real hardware supports them and traffic attribution can be proved. Require NCCL logs, topology, device identities, per-plane counters and a correctly scoped fault test before claiming another transport. Keep calibration results for each profile separate.
