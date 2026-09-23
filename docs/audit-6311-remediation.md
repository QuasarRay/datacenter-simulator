# Audit 6311d122 remediation

This change addresses the 20 confirmed findings in the September 23 audit of
`6311d122dd32b3620393d606559aada8443e638f`. The audit's separate architecture gaps
and hardware-dependent follow-ups are recorded below; they are not claimed fixed
by passing portable tests.

## Finding map

| Finding | Change | Verification |
|---|---|---|
| D001 | Arrival-ordered batch event scheduler; finalized individual calls reject overlapping submissions | Original fan-in counterexample, reversed input order, independent generated FIFO reference, full-duplex test |
| D002 | Canonical unordered endpoint sort for shared analytical/Linux route selection | Equivalent manifests choose the same physical link |
| D003 | Transactional OOB/start/shutdown/rebuild/reset with final retention check and checked generation increments | Exact-budget audit regressions; failed candidates never replace the original |
| D004 | Configurable aggregate QP/MR/queue/payload limits and reported usage | Exhaustion is atomic; destroy/deregister/poll release capacity |
| D005 | All four convenience collective APIs accept explicit `NcclOptions` | All targets compile; default/no-link builds still reject GPU execution |
| D006 | Reject nonzero/malformed `nvidia.com/*` resources in capacity or allocatable, including MIG/shared names | Unit cases plus real CLI with fake kubectl/Helm; Helm is never invoked on rejected nodes |
| D007 | Durable atomic result and batch-state writes before disarming rollback | Injected result-path failure uninstalls owned releases and records rollback |
| D008 | Peek child exit with `WNOWAIT`, kill group while leader owns its PID, then reap | Fake kubectl descendant test and async VM pipe-holder regression; PID guard disarms after reap |
| D009 | Patchbay caps live command/service/transcript logs per file and across a fabric; OpenSM's own log goes through the capped pipe | Shared-budget and still-running-writer tests; [dependency PR](https://github.com/QuasarRay/patchbay/pull/2) |
| D010 | Native IB cleanup runs before the model transition and remains callable when already stopped; backend retains a manager until cleanup finishes | Source ordering and feature checks; native partial-failure/cancellation scenarios require integration runtime |
| D011 | Positive FRR grammar allowlist rejects unmodeled disruptive interface/global state | Shutdown, ACL, OSPF, forwarding and static-route mutations |
| D012 | Usable-unicast checks on management networks, router IDs and /31 links | Multicast, loopback, unspecified, link-local and reserved endpoint mutations |
| D013 | Streamed byte caps, aggregate capture cap, process-wide total deadline, size/hash/config validation before publishing a downloaded index | Oversized header/body, exact cap, live deadline, HTTPS round trip and incomplete capture tests |
| D014 | Existing-lab refusal precedes all topology writes; a separate candidate is validated before atomic publication | Actual Ansible playbook with fake tools preserves the deployed descriptor byte for byte |
| D015 | Regenerate checked NetBox manifest including all snapshot provenance labels | Compile/replay diff |
| D016 | SSH trust regression checks remote command execution, not human error wording | Correct/wrong/correct key sequence with fresh server-side marker; live test needs CAP_SYS_CHROOT |
| D017 | Exact invocation and complete size/operation/root/type/mode matrix; pinned upstream count rounding and explicitly unchecked modes | Complete synthetic grids for all 11 operations; every single-cell deletion and duplicate rejection |
| D018 | Cached exact retained bytes on node/interface construction, name/lease indexes, incremental Fabric payload totals | Reference byte recomputation after edits plus construction benchmark |
| D019 | Successful `--help`, compiled feature list, all integration commands, mandatory pins and `doctor` | CLI smoke and compiled feature checks |
| D020 | Matching repository-root toolchain pin | Root Rustup selects 1.98.0; root Cargo commands compile |

## API and behavior changes

Use `schedule_transfers` for overlapping traffic. The old single-call scheduler
returned immutable traces despite future arrivals being able to change them.
Concurrent work must now be submitted together. Batches permit 4,096 messages and
65,536 total hops; equal arrivals use input order as a documented tie-break.
Subsequent finalized batches cannot precede the previous completion frontier.
Lifecycle reset retains its existing cancellation/frontier-reset behavior.

The convenience collective methods take `&NcclOptions` as their last argument.
Library pins remain mandatory for genuine native execution. No CPU collective or
mock GPU result was introduced.

Fabric limits are separate from simulation serialized-data limits. Lifecycle
transactions temporarily clone retained state to guarantee atomic failure;
their peak memory can exceed their steady-state retained byte count. Construction
caching avoids full-state scans in the reported hot path; bulk edits, deletion,
export and checkpoint validation can still scan the model.

The FRR pack deliberately rejects unsupported forwarding/policy statements. Its
parser is not a general FRR configuration interpreter. Backup transfers run on
the main thread so a process alarm can bound DNS/TLS/streaming across the complete
capture. Limits: 1 MiB index, 4 MiB per config, 64 MiB capture, 4 KiB PUT response,
120 seconds total. An incomplete download has no published manifest.

IB retained logs are capped at 16 MiB per file and 64 MiB per fabric, with at most
1,024 command log pairs. Truncated command output fails instead of becoming
successful evidence. Native caches and non-log artifacts are outside this log
budget. The dependency commit is pinned even before its PR is merged.

## Local validation

- Rust: `cargo test --locked --manifest-path simulator/Cargo.toml --all-targets --features vm,ibsim,nccl-check,netbox,mokka` passed 80 top-level tests. One test also launches its own filtered subprocess test; it is not counted twice.
- Rust Clippy with the same features and `-D warnings` passed.
- EX457: 91 passed, one skipped. The skipped live SSH test requires CAP_SYS_CHROOT, absent in this container; it remains mandatory in the existing CI runner.
- Mokka: all four isolated CLI regressions passed (MIG, shared GPU resource, result failure, surviving descendants). They use fake tools, not a real cluster.
- Patchbay: Clippy passed; 4 IB tests passed, 3 native tests explicitly ignored because their runtime prerequisites are unavailable.
- Native Linux/VM/IB/NCCL interfaces compile with `nccl-check`. This is compilation evidence, not GPU execution.
- Local RDMA compilation could not run because the environment lacks CMake/native headers and cannot install system packages with its available capabilities. The existing Rust CI installs those dependencies and checks RDMA.

No local CUDA/NCCL, hardware RDMA, OpenSM/ibsim integration, QEMU fleet or live
Kubernetes execution is claimed. The PR workflows must validate their supported
native paths. Genuine hardware results remain separate exact-commit gates.

## Construction measurement

Same release-build probe as the audit: create hosts with one interface each, no
links or native backend, in this shared container. Times are observations, not CI
thresholds or predictions for the user's machine. VmHWM is cumulative for the
benchmark process and excludes build/child processes.

| Nodes | Before (ms) | After (ms) | After VmHWM (KiB) |
|---:|---:|---:|---:|
| 128 | 4.831 | 0.566 | 1,732 |
| 256 | 17.521 | 1.245 | 2,096 |
| 512 | 68.987 | 2.268 | 2,808 |
| 1,024 | 275.126 | 3.765 | 4,232 |
| 2,048 | 1,054.679 | 8.244 | 7,112 |

Reproduce with `cargo run --locked --manifest-path simulator/Cargo.toml --release --example model_scale`.
Indexes add some steady-state memory (audit 2,048-node VmHWM was 6,304 KiB).

## Remaining goal gaps and follow-ups

G001–G008 require new models/backends and calibration, beyond the confirmed
implementation defects: GPU/NIC/NUMA/rail hierarchy, packet congestion and
multipath, alternate native transports with proven data paths, coupled execution
planes, causal storage/power/health behavior, broader workloads, larger native
fabric architecture, and whole-host resource admission. See the explicit
[backend boundaries](GETTING_STARTED.md). The native 240-link ceiling remains.

S001–S005: the root guide now selects backends, reduces the initial submodule
checkout, exposes read-only prerequisite discovery, and corrects hardware workflow
wording. CachyOS guest provisioning, Incus integration, automatic VFIO assignment
and a universal native dependency installer are not implemented. Image and
library pinning, cluster opt-in and fresh state directories remain required.

R001/R002 remain runtime follow-ups: cancellation of the outer NCCL future
requests worker termination but does not itself await a cleanup barrier, and
synchronous namespace/counter/file helpers are not preempted by an async timeout.
Callers must not infer safe GPU reuse from an externally cancelled future. Native
cancellation and blocking-helper supervision need dedicated hardware/integration
coverage. This PR's process-group fix concerns command descendants, not a claim
that these separate NCCL follow-ups are closed.

Carry-forward A-021/A-022: this PR does not fabricate exact-commit hardware success
or alter repository branch-protection settings. The existing hardware-evidence
workflow and trusted runners must provide those results before hardware claims
are accepted.
