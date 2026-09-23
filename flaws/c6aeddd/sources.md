# Primary technical sources and revision discipline

Retrieved/checked on 2026-09-23. NVIDIA references are used to identify behavior that the implementation does not model; recommendations and impact statements are the audit's analysis. Product documentation is not a performance calibration dataset.

| Source | What it supports |
|---|---|
| [NVIDIA DGX H100/H200 system guide](https://docs.nvidia.com/dgx/dgxh100-user-guide/introduction-to-dgxh100.html) | Multi-GPU server and NVSwitch topology differ from independent flat hosts |
| [NVIDIA DGX SuperPOD network fabrics](https://docs.nvidia.com/dgx-superpod/reference-architecture-scalable-infrastructure-h100/latest/network-fabrics.html) | Rail alignment and distinct compute, storage, in-band and OOB planes |
| [NVIDIA NCCL environment reference](https://docs.nvidia.com/deeplearning/nccl/user-guide/docs/env.html) | Explicit transport restrictions, GPU P2P and NIC/rail controls |
| [NVIDIA Cumulus ECMP](https://docs.nvidia.com/networking-ethernet-software/cumulus-linux-59/Layer-3/Routing/Equal-Cost-Multipath-Load-Sharing/) | Multiple equal-cost next hops differ from a single static path |
| [NVIDIA RDMA programming manual](https://docs.nvidia.com/rdma-aware-networks-programming-user-manual-1-7.pdf) | Verbs/QP bring-up, retry and completion/error semantics; stable conceptual reference, not a current hardware support matrix |
| [Linux user_namespaces(7)](https://man7.org/linux/man-pages/man7/user_namespaces.7.html) | Parent capabilities differ from capabilities acquired in a new user namespace |
| [Tokio timeout](https://docs.rs/tokio/latest/tokio/time/fn.timeout.html) | Cooperative timeout behavior for work that does not yield |
| [Tokio Child](https://docs.rs/tokio/latest/tokio/process/struct.Child.html) | Kill request versus waited/reaped child completion |

The `latest` documentation URLs may evolve; code evidence is pinned separately. Exact version-dependent NCCL report details were checked against the repository's pinned source, not inferred from a current web page:

| Dependency | Pinned revision | Inspected scope |
|---|---|---|
| [NVIDIA nccl-tests](https://github.com/NVIDIA/nccl-tests/tree/b4d5beebca8a76cf01335f724d154b9b9d394d96) | `b4d5beebca8a76cf01335f724d154b9b9d394d96` | Count/size/root/reduction handling, in-place validation and report output |
| [QuasarRay patchbay](https://github.com/QuasarRay/patchbay/tree/3d3c577c4b49ba23cb6b14f6b48aacbec933a93e) | `3d3c577c4b49ba23cb6b14f6b48aacbec933a93e` | User-namespace bootstrap, selected IB graph/log/process ownership |
| [NVIDIA Mokka](https://github.com/NVIDIA/k8s-test-infra/tree/4a44f73b41abc041f93d9b745677fa3210f65a84) | `4a44f73b41abc041f93d9b745677fa3210f65a84` | Consumed chart/profiles and host-facing deployment boundaries |
| [NVIDIA DeepOps](https://github.com/NVIDIA/deepops/tree/dc80499ea34b3f36563ed039421076de15071517) | `dc80499ea34b3f36563ed039421076de15071517` | Adapter contracts and observed upstream/setup/VM CI scope |

The remaining pinned Rust forks are recorded in `.gitmodules`/`simulator/upstreams.json`. No full downstream dependency audit is claimed. Exact GitHub Actions runs and branch-policy observations are linked in the validation slice rather than replaced by undated green-check claims.
