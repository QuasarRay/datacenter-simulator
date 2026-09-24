# Primary implementation references

Inspected 2026-09-24. The source exam ledger and exact guide URLs remain in
`blueprint.json`; do not infer a changed exam objective from this supplementary list.
Pin the actual deployed product's documentation/version when qualifying a lab.

| Area | Primary reference | Use in the course |
|---|---|---|
| Real DeepOps | [Pinned repository](https://github.com/NVIDIA/deepops/tree/dc80499ea34b3f36563ed039421076de15071517) | Roles, inventory, supported OS assumptions |
| Rusternetes | [Source](https://github.com/calfonso/rusternetes) | Native control plane; kubelet omitted from the Incus/KWOK route |
| KWOK | [Architecture](https://kwok.sigs.k8s.io/docs/design/architecture/) | Synthetic workers do not execute payloads |
| Incus | [REST API](https://linuxcontainers.org/incus/docs/main/rest-api/) | Owned lifecycle and asynchronous operation completion |
| MIG | [Concepts](https://docs.nvidia.com/datacenter/tesla/mig-user-guide/latest/concepts.html) and [profiles](https://docs.nvidia.com/datacenter/tesla/mig-user-guide/supported-mig-profiles.html) | GI/CI distinction and device-specific placement |
| InfiniBand | [Partition membership](https://docs.nvidia.com/infra-controller/documentation/configuration-day-1/infini-band/infini-band-partitioning) | Full versus limited membership |
| RoCE | [Cumulus RoCE](https://docs.nvidia.com/networking-ethernet-software/cumulus-linux-510/Layer-1-and-Switch-Ports/Quality-of-Service/RDMA-over-Converged-Ethernet-RoCE/) | ECN/PFC configuration and measurement |
| GDS | [Overview](https://docs.nvidia.com/gpudirect-storage/overview-guide/index.html) | Direct and fallback data paths |
| NCCL | [Communicators](https://docs.nvidia.com/deeplearning/nccl/user-guide/docs/usage/communicators.html) | Distinct device/rank identity and actual collective execution |
| NVSwitch | [Fabric Manager](https://docs.nvidia.com/datacenter/tesla/fabric-manager-user-guide/index.html) | Supported local-fabric management behavior |
| DCGM | [Feature overview](https://docs.nvidia.com/datacenter/dcgm/latest/user-guide/feature-overview.html) | Device observation and validation boundaries |
| Functional Python | [unpythonic](https://github.com/Technologicat/unpythonic) | Pure pipelines, immutable intent |
| Python macros | [mcpyrate](https://github.com/Technologicat/mcpyrate) | Reviewable source expansion and contract syntax |
| API client | [kr8s object API](https://docs.kr8s.org/en/stable/object.html) | Explicit API objects and kubeconfig |
| Visual inspiration | [KodeKloud CKA course](https://github.com/kodekloudhub/certified-kubernetes-administrator-course) | Tangible analogy; this course's port-workshop diagrams are original |

Version-sensitive behavior, especially experimental NCCL/MIG support, must be
qualified against the pinned binaries. No lesson grants compatibility from an
unversioned product name or a diagram.
