# Response to the 4121cb83 audit

Reviewed `datacenter-simulator-audit-4121cb83-2026-09-23.zip` against its exact
revision `4121cb83f0de0bfb0791f18e4b1796d6e066136d`, also current main when work
began. This audit has 25 numbered findings; it is separate from the earlier
675d35d audit. Source findings were confirmed. A-024/A-025 required an explicit
lifecycle contract; the fidelity matrix mostly describes existing scope limits.

Three tests using only the old public API were also run against an isolated,
unchanged 4121cb83 checkout: non-DATA breakout import, service/instruction export
and application determinism, and ACTIVE service mutation. All three failed on
the audited version and pass with these changes.

## Findings and disposition

| ID | Verification and change |
|---|---|
| A-001 | Confirmed arbitrary worker selection. Removed both the executable argument and environment override. Workers execute `/proc/self/exe`; the parent checks inode identity, session nonce, and nonzero bidirectional counters for nonempty multi-rank work. |
| A-002 | Confirmed loader/provenance weakness. Explicit canonical NCCL/CUDA paths and SHA-256 pins are required. Reject loader preloads/auditors, restrict child library search, inspect loaded mappings from the supervisor, and record driver/GPU identities. Operator pins and a trusted host remain prerequisites; no remote attestation is claimed. |
| A-003 | Confirmed missing per-simulation limits. Added configurable counts and serialized data/checkpoint budgets, usage reporting, and atomic admission for bulk/configuration/runtime operations. |
| A-004 | Confirmed collection-local budgets. One record/byte/request/deadline budget now spans every chunk and both NetBox observations. Tests exercise real loopback HTTP responses. |
| A-005 | Confirmed armed raw PID after reaping. The process-group guard now owns an optional PID and disarms synchronously on successful wait/try-wait. SSH capture no longer consumes/reaps the child behind an armed guard. Tests verify disarming and termination of a still-owned group; they do not attempt nondeterministic OS PID reuse. |
| A-006 | Confirmed importer accepted non-DATA breakout children. Both parent and children must be DATA. Invalid import remains atomic. |
| A-007 | Confirmed public Device escape. Internal access is crate-private; explicit `raw_device` permanently invalidates the fabric. A bounded `tcp_transfer` API supports native socket validation without exposing handles. |
| A-008 | Confirmed mutable IbFabric escape. Removed `backend()`, made the plan read-only, and added wrapper lifecycle/query methods. Shutdown stops further wrapper operations. |
| A-009 | Confirmed missing native bandwidth/latency edits. `set_link_spec` updates both endpoint qdiscs, endpoint state and routes, with complete rollback; rollback failure poisons the fabric. The native example exercises changed conditions and actual TCP delivery. |
| A-010 | Confirmed UUID-ordered exports and instruction application. Sort by semantic content, and apply instructions in that same deterministic order. |
| A-011 | Confirmed UUID-dependent addressing. Native link subnets and endpoint addresses use canonical endpoint names, independent of link insertion/orientation. |
| A-012 | Confirmed unchecked replay provenance. Validate API root and RFC3339 timestamp; origin relocation requires an explicit recorded override, and optional maximum age is enforced. |
| A-013 | Confirmed non-string GPU quantities were treated as absent. Only a missing GPU key or string `"0"` is accepted. Wrong types fail before installation. |
| A-014 | Confirmed implicitly created namespaces survived Helm rollback. Removed implicit namespace creation. Apply requires an existing Active namespace; namespace lifecycle stays with the operator. Existing namespaces are never removed by rollback. |
| A-015 | Confirmed unbounded local reads. Config files are capped at 1 MiB and snapshot files at 64 MiB before deserialization. |
| A-016 | Confirmed absent external executable identity. Record canonical paths and SHA-256 hashes for Mokka, DeepOps and Linux/NCCL tools. Mokka rechecks identities before each command and executes the resolved path. These are observations, not a signature or certification of operator-installed executables. |
| A-017 | Confirmed two mutable upload-artifact references. Pin both to the same full SHA already used elsewhere. |
| A-018 | Confirmed version-only Galaxy installation. Added a reviewed SHA-256 archive lock covering five roles and seven collections including transitive dependencies. Staging and Molecule verify the complete archive set before installing with dependency resolution disabled. The setup wrapper reapplies verified dependencies after the unmodified upstream setup script. |
| A-019 | Confirmed unsigned checksum acceptance in VM CI. Verify the detached checksum signature with Ubuntu's packaged cloud-image keyring before selecting the image digest. |
| A-020 | Confirmed failing current-main CI and underlying trust defect: netcommon forwards `config_file`, but pinned pylibssh ignores that argument. A source-hash-checked dependency patch exposes the supported `knownhosts` option. Local and execution-environment setup use the patch; host-key checking stays enabled. Added a real SSH acceptance/wrong-key regression. Hosted CI must verify the full network lab. |
| A-021 | Confirmed evidence gap. NCCL, RDMA and DeepOps-GPU runs on the audited SHA were queued, not successful. No CUDA/RDMA hardware exists in this workspace. This remains open until trusted hardware runners execute the exact proposed revision. |
| A-022 | Confirmed policy gap. The branch API reports main unprotected; no connector operation can change rulesets/branch protection. The existing exact-commit hardware-evidence workflow remains available, but it is not enforceable repository policy by itself. A repository administrator must configure required checks and trusted runner availability. No policy change is claimed. |
| A-023 | Confirmed regression gaps. Added worker-selection/traffic checks, type/import regressions, aggregate HTTP budget tests, process-guard ownership tests, retention tests, deterministic export/application/address tests and real SSH trust tests. |
| A-024 | Confirmed undocumented distinction. Document and test that renaming a node changes its control-plane name while preserving the existing guest hostname until Init or rebuild. |
| A-025 | Confirmed inconsistent service mutation policy. Require INACTIVE for service create/delete, with atomic rejection tests. |

## Validation

Local validation on Rust 1.98.1 (CI remains pinned to 1.98.0):

- 67 Rust tests pass with `ibsim,netbox,mokka,vm,nccl-check`; the worker rejection
  test also launches a separate test process to isolate environment changes.
- Clippy with warnings denied passes across those features and all targets.
- Native `nccl` entry points typecheck without the compile-only feature.
- EX457 automation tests: 12 pass, one live SSH test skips locally because this
  workspace lacks `CAP_SYS_CHROOT`. That test is mandatory in the privileged
  Dagger CI environment and fails there if the capability is absent.
- Ansible Builder successfully renders the patched execution environment.
- All 12 locked Galaxy archives install successfully; a deliberately altered
  cached archive is rejected before Galaxy is invoked.
- Ubuntu's actual detached checksum signature verifies with the packaged keyring.

Hosted workflow outcomes belong to the PR's exact commit and must be checked
there. Local compile-only validation never proves CUDA, native RDMA, guest
provisioning, or namespace/cluster execution.

## Deliberate boundaries

No Rust–Python interoperability was added. Existing Ansible/Python deployment
tools remain separate executables. No NCCL implementation, CPU reduction fallback
or fake verbs transport was introduced. Mokka remains scoped to Kubernetes GPU
contracts, and ibsim/OpenSM to InfiniBand management datagrams. They do not provide
CUDA kernels or InfiniBand verbs payload execution. The native NCCL namespace
backend uses Socket transport. Its hardware example now verifies raw-access
rejection and fresh-fabric recovery; it no longer mutates the kernel behind the
wrapper to manufacture a model/kernel disagreement. Dynamic routing, continuous
NetBox reconciliation and calibrated performance prediction remain out of scope.
