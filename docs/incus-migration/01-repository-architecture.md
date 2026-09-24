> Implementation authorized by the project owner on 2026-09-24. The former implementation prohibition is withdrawn. See integrations/incus for implemented behavior and validation boundaries.

# Repository architecture and replacement map

Baseline: `c6aedddfbb9ac569b5e0eb68acc616452bf66136`. Paths in this chapter are repository-relative. [The complete file ledger](19-file-coverage-ledger.md) complements this module-level map.

## Trace the current paths before replacing them

The root Ansible example renders `templates/datacenter.clab.yml.j2`, deploys FRR application containers, and tests direct IP links. Its ASNs are labels; this is not the routed EX457 lab. `cheatsheets/ex457/v6` adds SSH-to-vtysh adaptation, BGP intent, validation, saved configuration, backups, and a Controller compatibility harness. `v5` is historical regression input.

The Rust portable API is independent of both labs. `LinuxFabric` turns its model into patchbay device namespaces and a helper router/bridge per wire. It suppresses patchbay IX shortcuts and installs petgraph-derived static routes. Native NCCL uses workers in those namespaces. DeepOps adds QEMU guests behind TAP/NAT boundaries. NetBox imports intent, ibsim executes management traffic, and Mokka configures an existing Kubernetes cluster. Sharing a manifest currently does not couple all these data planes.

## Rust module decisions

| Existing module in `simulator/src/` | Required future work and invariant |
|---|---|
| `lib.rs` | Export the proposed Incus orchestration layer; retire VM exports/features only when replacements pass. Do not hide native feature failures. |
| `model.rs` | Keep stable IDs and lifecycle definitions. Separate analytical resources/image labels from Incus image fingerprints, observed process state, and measured limits. |
| `api.rs` | Preserve Air-inspired portable operations. Make runtime operations explicit; a model `start` alone cannot assert a live container. |
| `command.rs` | Version new JSON commands and structured errors. Distinguish plan, apply, observe, stop, destroy and repair. Preserve existing portable contracts. |
| `main.rs` | Replace VM operational commands/help with Incus lifecycle entry points after migration; reject obsolete flags with actionable errors. No automatic Docker fallback. |
| `manifest.rs` | Compile root YAML, EX457 intent and NetBox snapshots into one validated desired representation; do not silently lose interface/role/medium metadata. |
| `topology.rs` | Retain `DiGraph`, stable external IDs, directed edge costs, parallel-link identity and host-transit rejection. Add separate intended/observed graph views. |
| `linux.rs` | Refactor `LinuxFabric` attachment to externally owned Incus nodes; retain partition, route, queue, MAC, MTU and poison-on-rollback-failure contracts. |
| `fabric.rs` | Preserve software QP/MR/key/state checks. Never rename them to native RDMA merely because they run in a container. |
| `infiniband.rs` | Preserve native ibsim/OpenSM management lifecycle, topology validation and explicit shutdown. Map Incus node IDs to management endpoints without claiming payload coupling. |
| `collective.rs` | Preserve validation and native-only collective result policy; missing native execution must fail even for empty/single-rank jobs. |
| `cuda.rs` | Preserve native driver/context/buffer ownership. Container GPU injection changes visibility, not CUDA semantics. |
| `nccl_backend.rs` | Replace namespace worker launching with Incus execution or verified borrowed-netns execution as appropriate; validate GPU identities, data interfaces and transport environment. |
| `nccl_worker.rs` | Run the same real GPU worker against matching native libraries; preserve deadlines, abort behavior, output framing and correctness checks. |
| `rdma.rs` | Keep actual verbs completion tests separate from ibsim and software verbs. Qualify device/GID/port visibility inside Incus. |
| `deepops.rs` | Replace VM image/VFIO/guest plans with CachyOS Incus node plans and explicit host-driver ownership. Preserve manifest validation and compute count constraints. |
| `deepops_runtime.rs` | Port orchestration, inventory validation, Slurm/NCCL stages and cancellation to Incus. Replace Ubuntu-only role inputs rather than bypassing OS assertions. |
| `vm.rs` | Retire QEMU, qcow2, cloud-init disk, TAP descriptor, user-network and VFIO lifecycle code after Incus replacement acceptance. Extract reusable bounded process/partition logic first. |
| `mokka.rs` | Keep upstream revision, image digest, opt-in, node-capacity, namespace, chart-integrity and Helm rollback checks; change cluster provisioning assumptions and image-building instructions. |
| `netbox.rs` | Keep bounded read-only REST import, pagination/origin controls, double observation, source IDs and unsupported-cable rejection; service provisioning changes elsewhere. |
| `doctor.rs` | Add read-only Incus/API/image/cgroup/storage/namespace/resource checks; remove QEMU/Docker requirements from current paths. Distinguish discovered tools from verified capabilities. |
| `input.rs` | Preserve bounded command/config reads and malformed-input handling across new API clients. |
| `limits.rs` | Keep retained-model byte/count budgets; add separate host resource admission, not a relabel of serialized bytes as RSS. |
| `evidence.rs` | Record Incus/image/kernel/collection/fork versions and actual outcomes; preserve executable identity checks. |
| `provenance.rs` | Retain verification of consumed sources, including ignored/untracked chart contamination. Extend to image-build inputs. |
| `process_group.rs` | Reuse bounded child-process supervision; Incus-managed services require Incus stop/delete, not host PID-only cleanup. |
| `bounded_log.rs` | Retain bounded diagnostics; add journal/Incus operation logs without exporting secrets. |

## Dependency and interface decisions

Keep `simulator/Cargo.toml`, `Cargo.lock`, both `rust-toolchain.toml` files and `upstreams.json` synchronized. `petgraph` and `patchbay` are already pinned path dependencies. Inspect the actual submodule commits, not crates.io API examples. Add safe Rust orchestration over Incus API/CLI as a separate boundary; do not embed Python. Ansible, NetBox, pyATS and Hypothesis remain external applications/test tools, so their Python code is not Rust–Python runtime interop.

The patchbay gitlink is `3d3c577c4b49ba23cb6b14f6b48aacbec933a93e`; petgraph is `a4d94bd2c39ac198c22682b2dcb1ff21583d0db0`. The pinned patchbay has `Lab`, `Device`, internal `NetnsManager`, `Device::run_sync`, `Device::spawn_command`, and `init_userns`; it does not expose a turnkey Incus backend. [05](05-patchbay-petgraph-networking.md) specifies the extension needed. Preserve native NCCL/rust-ibverbs/ibsim pins unless a separately validated change requires an update. Retain license and NOTICE attribution.

## Non-Rust paths

| Surface | Replacement requirement |
|---|---|
| `playbooks/{deploy,destroy}.yml`, `inventory/localhost.ini`, `vars/datacenter.yml` | Delegate lifecycle to the new Rust transaction; keep declared resources, collision refusal and rollback. |
| Root `templates/datacenter.clab.yml.j2` | Retire; compile intent into the common graph/node plan. Preserve all addresses, links and limits. |
| `tests/{datacenter_state,deploy_refusal}.py` | Preserve behavioral assertions; replace Docker labels/exec/inspect with observed Incus and kernel state. |
| `cheatsheets/ex457/v6/tools/labctl.py` | Replace lifecycle/image/exec calls with the single Rust interface; eliminate duplicated topology ownership. |
| v6 `lab/frr/*` | Replace Docker image/entrypoint with CachyOS package provisioning and systemd FRR/SSH units. Re-test forced-command, public-key, transport and saved-config behavior. |
| v6 inventory, templates, roles, playbooks, filters | Preserve exact BGP/routing/backup predicates; replace addresses and transport assumptions only where required. No static route shortcut in BGP drills. |
| v6 `controller/*`, EE definitions, Navigator configuration | Follow [13](13-ex457-network-automation.md); actual AAP remains a product gate. Pin client schemas and execution routes. |
| v6 tests, tools, evidence, component lock, packaging | Preserve historical counterexamples; regenerate evidence for the new runtime. Old Docker test output cannot certify Incus. |
| `cheatsheets/ex457/v5/**` | Archive as non-runnable historical regression fixtures, with no supported deployment instructions pointing to it. Keep source counterexamples discoverable. |
| `integrations/netbox/{compose.ci.yml,seed-ci.sh}` | Replace Compose with CachyOS system services in Incus; preserve real API seeding/read-only importer checks. |
| `integrations/k8s-test-infra/{kind.ci.yaml,config.example.json}` | Replace Kind node provisioning with Incus Kubernetes nodes; regenerate context/node names and immutable image identity. |
| `integrations/deepops/**` | Port provisioning, installation/collection locks, Slurm dependencies, benchmarks and sample topology to CachyOS/Incus. |
| `ci/**` | Replace Dagger/DinD lifecycle and Docker-specific cleanup; retain verdict freshness, redaction, exact required gates and report schema checks. |
| `simulator/examples/**`, `simulator/tests/**` | Rebase runtime harnesses; keep model, import, software verbs and native correctness coverage. Update Python Mokka CLI fixtures as needed. |
| Root/simulator/docs README material | Switch the supported getting-started path atomically at cutover; distinguish modeled, container, host and physical tests. |

## Workflow-by-workflow migration

| Workflow | Required change |
|---|---|
| `ci.yml` | Replace Dagger action with direct runner orchestration; exercise root and EX457 Incus labs; preserve always-export and strict verdict. |
| `rust-simulator.yml` | Keep portable/lint/contracts; add compile/contract coverage for Incus and patchbay attachment. |
| `intended-fabric.yml` | Replace Compose, Docker builds/registry and Kind; retain real NetBox import, native ibsim, upstream Mokka unit/race/Helm and workload gates. |
| `deepops.yml` | Replace QEMU smoke test with CachyOS Incus provisioning and fabric checks. |
| `deepops-gpu.yml` | Run ported Slurm/native tests with explicit real GPU allocation and host-driver qualification. |
| `nccl-runtime.yml` | Verify workers in the new Incus/fabric path; preserve transport exclusion and physical cut/recovery. |
| `rdma-runtime.yml` | Qualify direct RNIC assignment, port/GID and real completion evidence independently. |
| `hardware-evidence.yml` | Require exact commit/fresh trusted runs of the revised gates; retain no-forged-hardware-result policy. |

Read source-reference links at the pinned [repository tree](https://github.com/QuasarRay/datacenter-simulator/tree/c6aedddfbb9ac569b5e0eb68acc616452bf66136). This map describes changes to make later, not changes contained in this documentation PR.
