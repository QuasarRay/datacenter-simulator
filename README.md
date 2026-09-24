# Datacenter simulator

GPU datacenter models and native execution adapters, with an Incus system-container
teaching environment and the independent [NCP-Metablueprint](education/ncp-metablueprint/README.md).

The [Rust simulator](simulator/README.md) retains its topology and lifecycle API,
petgraph routing, patchbay networking, CUDA/NCCL, native RDMA, InfiniBand management
simulation, NetBox intent import and evidence contracts. The new
[Incus backend](integrations/incus/README.md) creates owned system containers and
connects their explicit data ports through patchbay cables.

Use [native provisioning](integrations/incus/PROVISIONING.md) for real NVIDIA
DeepOps role execution, Rusternetes, KWOK and service boundaries. Students work
in code and Ansible. Host lifecycle privileges remain with the trainer.

The former deployment runtime has been removed. Historical EX457 text and audit
fixtures remain under `cheatsheets/` and `flaws/` as source history; they are not
current deployment instructions. The implementation prohibition in the Incus
migration proposal has been withdrawn by the project owner. Its unmet acceptance
criteria remain visible requirements.

Portable checks do not imply a validated physical GPU datacenter. Live Incus,
vendor services, proprietary control planes, optical hardware and native GPU/RDMA
execution require their corresponding evidence tiers. Read each integration's
validation status before interpreting its results.
