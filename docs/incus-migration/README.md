> Implementation authorized by the project owner on 2026-09-24. The former implementation prohibition is withdrawn. See integrations/incus for implemented behavior and validation boundaries.

# Full replacement with Incus, patchbay, and petgraph

This collection specifies how to replace the project's Docker/Containerlab infrastructure with **CachyOS Incus system containers**, with **patchbay controlling the emulated fabric** and **petgraph representing topology and calculating model routes**. It also specifies an Ansible-based curriculum covering every public objective enumerated for EX200, EX294, EX457, and EX342 on 2026-09-23.

**Status: instructions only.** No Incus backend, new playbook, image, CI job, service, or troubleshooting exercise is implemented by this change. Examples inside Markdown are designs or illustrative tasks; new names are explicitly proposed. Do not copy a proposed CLI invocation into the existing binary and expect it to work.

Repository baseline: [`c6aedddfbb9ac569b5e0eb68acc616452bf66136`](https://github.com/QuasarRay/datacenter-simulator/tree/c6aedddfbb9ac569b5e0eb68acc616452bf66136). Re-inventory changes after this commit before implementation. Existing code already uses petgraph and patchbay; this is a lifecycle/network integration replacement, not a proposal to introduce those libraries for the first time.

## Read in this order

| Document | Decision or procedure |
|---|---|
| [01 Repository architecture](01-repository-architecture.md) | Every Rust module, lab, integration, and workflow; preserve/change/remove decisions |
| [02 Target architecture](02-target-architecture.md) | Ownership, graph model, routing, control boundaries, persistence |
| [03 CachyOS and images](03-cachyos-host-and-images.md) | Host preparation, reproducible CachyOS rootfs, image qualification |
| [04 Incus lifecycle](04-incus-lifecycle.md) | API, operations, state, admission, recovery, snapshots, teardown |
| [05 Patchbay integration](05-patchbay-petgraph-networking.md) | Required fork work, external namespace ownership, wiring and failure semantics |
| [06 Ansible contracts](06-ansible-orchestration.md) | Inventories, bootstrap, SSH, diagnosis, repair, idempotence |
| [07 NetBox and services](07-netbox-and-services.md) | Replace Compose; preserve importer, database, identity, logs and backups |
| [08 GPU and InfiniBand](08-gpu-nccl-rdma-infiniband.md) | CUDA/NCCL, verbs, ibsim, topology fidelity and hardware boundaries |
| [09 DeepOps and Kubernetes](09-deepops-kubernetes-mokka.md) | Replace QEMU provisioning/Kind where specified; port OS roles; retain real workload semantics |
| [10 Curriculum contract](10-exam-coverage-contract.md) | Objective IDs, execution classes, non-equivalences and grading |
| [11 EX200](11-ex200-system-administration.md) | 62 objective rows with CachyOS/Ansible procedures and checks |
| [12 EX294](12-ex294-ansible-administration.md) | 57 objective rows, including inherited administration and development tools |
| [13 EX457](13-ex457-network-automation.md) | 23 objective rows, network drills and Controller workflows |
| [14 EX342](14-ex342-diagnostics.md) | 30 objective rows with diagnosis, repair and proof |
| [15 Recovery runbooks](15-ansible-recovery-runbooks.md) | Broken SSH, boot, storage, packages, authentication, applications and kernels |
| [16 CI and acceptance](16-ci-and-acceptance.md) | Docker-free replacement gates, property testing and runtime evidence |
| [17 Cutover and removal](17-cutover-and-removal.md) | Ordered replacement, retirement, rollback and completion criteria |
| [18 Sources](18-sources-and-version-policy.md) | Official blueprint URLs, technical sources and pinning rules |
| [19 File coverage](19-file-coverage-ledger.md) | Baseline tracked-file disposition, including historical fixtures |

## Required end state

1. Incus is the only infrastructure-node runtime. A node has a private CachyOS filesystem, PID 1/systemd, identity, cgroup budget and lifecycle. No Docker, dockerd, Compose, Containerlab, Kind, Dagger engine, or QEMU node-provisioning dependency remains in the supported operational path.
2. Existing portable simulation remains available without Incus. It is a model, not a second infrastructure runtime. Namespace-only unit/integration tests remain useful internal tests; they must not be advertised as a competing deployment backend.
3. The root connectivity example and the active EX457 lab use the same new lifecycle implementation. There is no `backend=docker` fallback, dual deployment mode, or permanent translation shim.
4. patchbay owns fabric wiring and impairments; Incus owns container namespaces and its NIC devices; FRR or the static-route compiler owns a routing table, never both. petgraph does not forward real packets.
5. CachyOS is the host and node OS. DeepOps and package/service roles must be ported and tested. Renaming an Ubuntu image does not meet this requirement.
6. Kubernetes pods still require a CRI runtime such as containerd **inside** Incus nodes. OCI image formats are not Incus system images. This preserves Kubernetes semantics while replacing Docker/Kind infrastructure. Navigator EEs and VS Code development-container requirements are explicitly addressed in [10](10-exam-coverage-contract.md); an Incus shell is not falsely described as an OCI development container.

## What full coverage means

Every public blueprint item has an original practice specification, an execution location, and a success criterion. This is **complete mapped scope**, not a claim that all labs already work, that proprietary products run on CachyOS, or that these instructions certify exam readiness. Exam-specific RPM/CDN, SELinux, AAP, and development-container behavior is distinguished from CachyOS adaptations. Kernel/boot exercises use a disposable CachyOS host or rescue environment, never an invented container kernel. Physical hardware failures require physical evidence.

Each future lab must fail visibly if a prerequisite is missing. `BLOCKED`, `NOT_RUN`, and `ADAPTED` must never become `PASS` automatically. The objective ledgers and the file ledger make omissions reviewable; no static document can guarantee the absence of every future compatibility issue.

Sources and source-order rules are in [18](18-sources-and-version-policy.md). The guide contains original migration/lab designs and brief topic labels, rather than copying vendor objective text.
