# Audit investigation: 675d35d

The supplied archive contains 47 findings against commit
`675d35db1a6a9ce280e4296cf01800ce0f51380f`, which was still main when this
investigation began. The cited behaviors are present in that snapshot. The
severity labels combine software defects, test coverage gaps and deliberate
capability boundaries; they are not all equivalent implementation bugs.

This change fixes the confirmed software defects, adds regression coverage, and
makes evidence boundaries explicit. It does **not** claim to implement an
InfiniBand verbs payload provider or demonstrate unrun GPU/RNIC tests. No NCCL
substitute or Rust/Python bridge has been added.

## Findings and disposition

“Implemented” describes the change, not a claim that unavailable hardware was
exercised. “Boundary” means the observation is correct but cannot be closed by
claiming broader fidelity. “Open” identifies missing capability/evidence.

| ID | Investigation and remediation |
| --- | --- |
| DS-001 | Confirmed; implemented a monotonic submission frontier. Earlier timestamps fail atomically; reset clears the frontier and reservations. Equal timestamps retain submission-order FIFO semantics. |
| DS-002 | Confirmed; rank admission builds one graph and computes switch components, with direct host links checked separately. No all-pairs path reconstruction. The 1,024-rank regression has a ten-second admission budget. |
| DS-003 | Confirmed; checkpoint cloning rejects changed configuration and missing runtime before creating a destination. |
| DS-004 | Confirmed; checkpoints retain instruction states and cloning maps them to fresh IDs. Pending instructions remain pending until executed. Post-checkpoint instruction additions cause a conflict. |
| DS-005 | Confirmed; compatibility compares exact serialized configuration, including interfaces, links, resources, services, instructions, OOB and guest config, rather than node keys alone. |
| DS-006 | Confirmed; nonempty node resets cancel all analytical reservations. |
| DS-007 | Confirmed; initial allocation uses names, existing leases survive insertion, and clones copy leases. |
| DS-008 | Confirmed; history retains 10,000 events by default; callers can change/disable retention. |
| DS-009 | Confirmed; a checked monotonic 40-bit allocator gives unique locally administered MACs within each simulation. |
| DS-010 | Confirmed; manifests must declare endpoint interfaces. Checked-in example manifests have been migrated. |
| DS-011 | Confirmed; only the documented Air image-metadata keys are accepted outside named node fields. Custom data belongs in labels. |
| DS-012 | Confirmed; wire and Rust-buffer limits are now explicitly distinct. Oversized/invalid UTF-8 lines produce errors and drain safely without ending the session. |
| DS-013 | Confirmed; namespace setup occurs at native NCCL execution, before its runtime, rather than for every NCCL-enabled CLI invocation. |
| DS-014 | Confirmed; local manifest reads are capped at 16 MiB before deserialization. |
| DS-015 | Confirmed; native queues derive from two one-way BDPs plus 1,024 packets at explicit MTU 1500. Unsupported capacity fails before namespace construction. The formula is public/documented and tested at high BDP. |
| DS-016 | Confirmed; native host/guest routes share one graph per update and one shortest-path search per source, with memoized first hops. Route installation remains proportional to the actual routing-table size. |
| DS-017 | Confirmed; analytical/native routing both use static minimum latency, then hop count. Payload serialization changes time, not route choice. |
| DS-018 | Confirmed; endpoint and route updates form a rollback transaction, model commit follows success, all restoration steps are attempted, and failed restoration poisons the fabric. Failure-injection unit tests cover every stage. |
| DS-019 | Confirmed; routing compilation propagates validation errors; only absent reachable destinations omit routes. |
| DS-020 | Boundary; docs explicitly limit native links to rate/delay/link faults and the stated queue/MTU policy. PFC/ECN, jitter and programmable loss are not implemented or claimed. |
| DS-021 | Confirmed; worker failure cleanup has a two-second grace period, reports unreaped PIDs, and never says all workers terminated when that cannot be established. The kernel may retain a stuck process/GPU. |
| DS-022 | Confirmed; results explicitly identify aggregate interface-counter accounting, including possible concurrent traffic. These counters are not per-collective byte attribution. |
| DS-023 | **Open capability:** NCCL remains real Socket execution on the modeled Ethernet fabric. Removing the transport restriction without an attached RDMA backend would permit an unmodeled hardware shortcut. NCCL-IB/GPUDirect is not implemented or claimed. |
| DS-024 | **Open coverage:** the real verbs test remains local RC loopback. Results now explicitly say `modeled_fabric_tested: false`; this does not establish multi-node payload routing. |
| DS-025 | Confirmed; RDMA now requires explicit device name, port and GID index and emits that selection. No enumeration-order fallback. |
| DS-026 | **Open capability:** upstream ibsim emulates management MADs, not RNIC verbs payloads. Joining an independent SoftRoCE test to the report would not make it InfiniBand. A genuine payload backend and joint fault test are still needed. |
| DS-027 | Boundary; IB plans explicitly record `bandwidth_latency_enforced: false`, `rdma_payload_tested: false` and `nccl_tested: false`. |
| DS-028 | Confirmed; a VM partition guard restores every changed link and guest routes on ordinary errors and cancellation, including failed SSH. Restoration failure poisons the fabric. |
| DS-029 | Confirmed; DeepOps/Kubespray staging uses the exact committed archives and freshly installs declared Galaxy dependencies. Untracked local modules/roles are not copied. Galaxy versions themselves are not content-hash attestations. |
| DS-030 | Confirmed; subprocess, QEMU and serial output is continuously drained but retained files stop at 16 MiB with truncation markers. Collector tests verify the bound. |
| DS-031 | Implemented, **hardware verification outstanding:** the DeepOps GPU run waits for an actual nccl-tests correctness row, cuts modeled links while repeated cycles are running, requires failure, restores/cancels, then runs the normal suite for recovery. |
| DS-032 | Confirmed; Mokka rejects tracked, untracked and ignored changes in consumed chart/profile paths. Tests cover all three mutation types. |
| DS-033 | Confirmed; source verification precedes profile reads and plan/render output creation. |
| DS-034 | Confirmed; apply requires a source tag plus OCI digest. The chart receives the digest-qualified reference, deployed pod references are checked, and running image IDs are recorded. An explicit pod readiness wait handles Helm's early return for OnDelete DaemonSets. Tag-only plans explicitly report `image_pinned: false`. |
| DS-035 | Confirmed; new-release batches roll back their owned releases on any install/verification failure. A unique Helm label prevents deleting a competing/pre-existing release. Partial cleanup is recorded in `batch-state.json`. A process-level regression injects failure in the second release and checks cleanup of the first without removing an unowned release. |
| DS-036 | Partially mitigated; two complete, sorted intended-state observations must agree. Tests detect changes and tolerate listing order changes. REST acquisition is still not a database transaction and cannot detect changes reverted between observations. |
| DS-037 | Boundary; mandatory Mokka scope flags remain false for NCCL/RDMA, with existing regression assertions. No generic GPU-execution success is synthesized. |
| DS-038 | Partially mitigated; trusted-main pushes and weekly runs now trigger hardware workflows. A reusable promotion gate requires exact-commit successes within seven days. Runner provisioning/configuration, actual successful execution, and requiring the gate in repository policy remain operator prerequisites. |
| DS-039 | Confirmed; Helm unittest is built from immutable commit `6f82a998e0b5461762ca959f87f5dd344af5e4eb`; the unverified tag installer is removed. Go dependencies remain checked by its go.sum. |
| DS-040 | Confirmed; the complete Ubuntu/CPython 3.12 Molecule dependency resolution has exact versions and SHA-256 hashes and is installed with `--require-hashes`. |
| DS-041 | Confirmed; NetBox, Postgres, Valkey and the new ephemeral test registry use verified registry manifest digests. |
| DS-042 | Boundary clarified; root Containerlab is explicitly a connectivity lab with ASN labels. It does not claim BGP sessions; routed BGP exercises are separately located under EX457. |
| DS-043 | Confirmed; legacy tests compare the exact lab-labelled container set and reject extra data interfaces, allowing only lo/eth0 extras. |
| DS-044 | Confirmed; legacy tests now require real direct-link ping, partition failure and recovery, with restoration in finally. This is connectivity scope, not an assertion of unconfigured BGP. |
| DS-045 | Confirmed; deployment refuses pre-existing/partial labs rather than silently replacing them. New deployment and verification failures enter cleanup/rescue and report cleanup failure. |
| DS-046 | Confirmed; removed the unused server profile. |
| DS-047 | Confirmed; sessions default to 128 simulations with an explicit configurable quota, enforced for create/import/clone. |

## Evidence and limits

Three identical regressions (DS-001, DS-003 and DS-004) were also run against an
independent worktree of the audited commit: all three failed there and pass with
these changes.

Local Rust validation uses 1.98.1; hosted workflows retain the project's pinned
1.98.0 toolchain. The combined `ibsim,netbox,mokka,vm,nccl-check` suite passed
52 tests and its all-target clippy run passed with warnings denied. This builds
native orchestration but deliberately cannot execute NCCL. JSON transport,
checkpoint, scheduler, provenance, rollback, log bounds and snapshot comparison
have targeted regressions. YAML, shell syntax and patch whitespace checks pass.

This workspace rejects user-namespace UID mapping and has no CUDA GPUs,
container daemon or native RDMA build prerequisites. Native Linux, ibsim, VM,
Mokka deployment and hardware execution therefore require hosted/dedicated CI;
a compile or portable test result is not substituted for those runs.

The audit's baseline CI observation was independently verified: rust-simulator,
intended-fabric and deepops succeeded, while repository-wide datacenter-ci
failed at the audited main SHA. Baseline runs:
[Rust](https://github.com/QuasarRay/datacenter-simulator/actions/runs/35728109908),
[intended fabric](https://github.com/QuasarRay/datacenter-simulator/actions/runs/35728109960),
[DeepOps](https://github.com/QuasarRay/datacenter-simulator/actions/runs/35728109931),
[legacy gate](https://github.com/QuasarRay/datacenter-simulator/actions/runs/35728109984).
New PR CI results are assessed separately. At remediation commit `8143ee4d`:

- [Rust/native](https://github.com/QuasarRay/datacenter-simulator/actions/runs/35742134720)
  passed, including native namespace traffic and partition/recovery, RDMA
  compilation and NCCL type checking.
- [DeepOps](https://github.com/QuasarRay/datacenter-simulator/actions/runs/35742134729)
  passed upstream checks, all three Molecule scenarios and real VM fabric
  partition/recovery.
- [Intended fabric](https://github.com/QuasarRay/datacenter-simulator/actions/runs/35742134755)
  passed upstream Mokka tests, Rust tests and live NetBox/ibsim/OpenSM discovery,
  partition and recovery. Its Mokka deployment exposed a container readiness race;
  the follow-up explicitly waits for ready pods before querying NVML.
- [The separate EX457 gate](https://github.com/QuasarRay/datacenter-simulator/actions/runs/35742134680)
  still fails host trust. The pinned `ansible-pylibssh` 1.2.2 `Session.connect`
  ignores `config_file` supplied by netcommon 8.1.0, so the generated alternate
  known-hosts configuration never reaches libssh. This pre-existing training-lab
  transport defect remains open; host-key checking has not been disabled.

GPU NCCL execution, the new in-flight fault case and RNIC payload execution still
need dedicated hardware evidence. Passing CPU CI does not close DS-023/024/026
or establish a successful hardware gate for DS-038.
