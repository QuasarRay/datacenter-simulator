# Coverage and evidence ledger

Baseline: `c6aedddfbb9ac569b5e0eb68acc616452bf66136`; content tree `d13c5386e75e8215d85888376a93c5faf34383c8`. Inventory: 349 tracked entries including seven gitlinks; 27 first-party Rust source modules, 9,497 source lines. Counts describe the tree, not an assertion that every upstream/generated/evidence line received a complete correctness proof.

## Reviewed surfaces

| Layer / files | Review focus | Evidence / remaining work |
|---|---|---|
| `api.rs`, `model.rs`, `manifest.rs`, `command.rs` | Lifecycle, identity, mutation admission, checkpoints, import/export and command dispatch | Full-budget shutdown and clone-policy probes; existing import/lifecycle regressions pass. No exhaustive command-sequence model checking |
| `topology.rs` | Canonical paths, directed-link timing, concurrent batches, frontier/reset semantics | Corrected FIFO retained; packet/ECMP/convergence limits recorded; no calibrated ASIC reference |
| `limits.rs`, `fabric.rs` | Retention accounting, transient copies, queue/memory admission and software verbs | Public-API queue/history/lifecycle/RNR probes; hardware verbs comparison remains scoped |
| `collective.rs`, `cuda.rs`, `nccl_worker.rs`, `nccl_backend.rs` | Shapes, GPU placement, buffers, worker identity/environment, timeout and cleanup | Compile-only tests plus source review; no local CUDA execution or cancellation experiment |
| `linux.rs` and selected patchbay namespace paths | Bridge isolation, route installation, link conditions, helper processes and scale | Current main's real namespace CI passed; local privileged deployment not run |
| `rdma.rs` | Device selection, MR/QP/CQ ownership, polling and deadline extent | Local source review and remote compile evidence; device runtime waiting |
| `infiniband.rs` and pinned patchbay IB topology/log/process files | Native management graph, link updates, log budgets, shutdown ownership | Current main's NetBox/ibsim/OpenSM job passed; no payload/RDMA fidelity claim |
| `deepops.rs`, `deepops_runtime.rs`, `vm.rs`, NCCL rank script | Guest planning, source pins, inventory, Slurm/report validation, faults and VM cleanup | CPU VM/Molecule CI passed; hardware workflow pending; fixed benchmark matrix checked against pinned nccl-tests sources |
| `mokka.rs`, pinned chart/profile boundaries | Plan guards, GPU inventory checks, ownership/rollback and actual validation scope | Existing regressions pass; duplicate/wrong-model CLI counterexample; no host-cleanup fault injection |
| `netbox.rs` | Acquisition budgets, same-origin pagination, two observations, selected-port translation and provenance | Existing HTTP regressions and remote real-service CI pass; cross-backend causality absent |
| `doctor.rs`, `main.rs`, `evidence.rs`, `input.rs`, `provenance.rs` | Feature discovery, prerequisite mismatches, identity and bounded input/tool behavior | Doctor predicate and blocking FIFO reproduced; native installers/driver combinations not exhaustively tested |
| `bounded_log.rs`, `process_group.rs` | Output retention, process-group ownership and completion handling | Existing fake-process tests pass; discarded QEMU drain result and outer cancellation analyzed |
| Root deploy/destroy playbooks, topology template and `tests/datacenter_state.py` | Existing-lab refusal, cleanup, traffic/partition recovery | Refusal regression covered by Python suite; exact CI failure and negative-probe counterexample recorded |
| v6 fabric filters, backup tool, lifecycle/restore automation and CI harness | Managed FRR grammar, transfer limits, state/trust/backup evidence and deployment orchestration | 91 passing Python tests, one local live-SSH skip; later full v6 runtime gates blocked by the observed legacy deployment failure |
| Toolchains, Cargo/features, submodule pins, setup docs and all workflow definitions | Minimal setup, optional native modes, evidence scope and promotion policy | Source/pin review plus GitHub branch/ruleset/run observations; no Windows/WSL/CachyOS clean-host install measurement |

## Status of earlier findings

PR #12 was merged before this review. The new reports preserve the distinction between its implemented D001–D020 fixes and the remaining architectural G001–G008, setup and R001/R002 follow-ups documented in `docs/audit-6311-remediation.md`. A passing regression only establishes its tested invariant; for example, atomic quota rejection does not guarantee lifecycle liveness (RES-05), and a complete NCCL microbenchmark matrix does not establish training-workload fidelity (FID-06).

This is a first-party integration audit with targeted inspection of pinned upstream code. It is not a full audit of NCCL, CUDA drivers, patchbay, QEMU, FRR, Kubernetes, Mokka or the full Ansible dependency tree. Historical generated evidence/log files were treated as historical claims, not as substitutes for current execution.

## Reproduction discipline

- Resource measurements use a small temporary Rust example containing only public API calls; complete source is in the resources slice. Temporary code is excluded from these PRs.
- Mokka uses the repository's isolated fake-tool CLI harness. The fault oracle invokes the actual current method with stubs. Neither reproduction connects to infrastructure.
- The FIFO identity reader is supervised and killed after one second; no indefinite process is left behind.
- No hardware results were synthesized. Queued/pending tests, disabled native modes, skipped tests and source-only risks retain those classifications.
- Reports link code at the audited SHA and upstreams at their gitlink SHAs. Remote CI/policy observations are time-dependent and require refresh before release decisions.

## Remaining evidence to obtain

Hardware calibration across GPU/NIC/NUMA/rail placements; physical RDMA error/retry traces; external-cancellation cleanup under real CUDA load; node-host restoration after failed Mokka runs; fully passing legacy-plus-v6 CI; and clean consumer-host setup/scale measurements. These are specific outstanding tasks, not implicit passes.
