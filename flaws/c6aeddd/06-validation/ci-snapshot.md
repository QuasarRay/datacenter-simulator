# Exact-revision CI snapshot

Target: `c6aedddfbb9ac569b5e0eb68acc616452bf66136`. GitHub main is the merge of PR #12; its content tree is `d13c5386e75e8215d85888376a93c5faf34383c8`. Observed on 2026-09-23 during this audit (approximately 07:25–08:00 UTC). Refresh run URLs before using this as release evidence.

| Workflow / run | Observed result | What that establishes |
|---|---|---|
| [rust-simulator / 35830854022](https://github.com/QuasarRay/datacenter-simulator/actions/runs/35830854022) | Success | Portable checks, native compilation and namespace TCP partition/recovery |
| [deepops / 35830853938](https://github.com/QuasarRay/datacenter-simulator/actions/runs/35830853938) | Success | Upstream/setup/Molecule checks and real CPU VM plumbing/partition/recovery |
| [intended-fabric / 35830853929](https://github.com/QuasarRay/datacenter-simulator/actions/runs/35830853929) | Success | NetBox/ibsim/OpenSM, upstream Mokka tests and Kind agent-local NVML |
| [datacenter-ci / 35830853931](https://github.com/QuasarRay/datacenter-simulator/actions/runs/35830853931) | Failure | Legacy recovery failure prevents later required gates from completing |
| [nccl-runtime / 35830854005](https://github.com/QuasarRay/datacenter-simulator/actions/runs/35830854005) | Queued | No hardware execution pass observed |
| [rdma-runtime / 35830854018](https://github.com/QuasarRay/datacenter-simulator/actions/runs/35830854018) | Queued | No device execution pass observed |
| [deepops-gpu / 35830853941](https://github.com/QuasarRay/datacenter-simulator/actions/runs/35830853941) | Pending, no jobs returned | No GPU VM/Slurm/NCCL pass observed |

## VAL-01 — Legacy deployment fails its recovery check

**High · Observed failure; root cause not established.** [Dagger job 107082848244](https://github.com/QuasarRay/datacenter-simulator/actions/runs/35830853931/job/107082848244) reported at 07:20:02 UTC:

```text
tests/datacenter_state.py, line 289, direct_delivery_partition_and_recovery
subprocess.CalledProcessError:
['docker', 'exec', 'clab-gpu-dc-spine1', 'ping', '-n', '-c', '1',
 '-W', '2', '-I', 'eth2', '10.0.0.3'] returned non-zero exit status 1
```

[Traffic test](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/tests/datacenter_state.py#L279) performs a positive ping, cuts the peer interface, checks rejection, restores the interface, then performs the failing **recovery** ping at line 289. `ci/run_ci.py` records `legacy-deploy=FAIL` and aborts subsequent required work. The final check properly rejects that exported run. Cleanup also logged a read-only `/etc/hosts` message; that is separate from the traffic failure and not established as its cause.

**Next investigation:** retain `ip -j address/link/route/neigh`, FRR and Docker state before/after the cut, plus both ping outputs. Determine whether state restoration is incorrect or readiness is transient. If eventual readiness is the contract, use bounded, observable retries; do not turn a persistent forwarding failure into a pass. Acceptance requires the exact run to complete all required legacy and v6 gates.

## VAL-02 — Hardware evidence exists as a workflow but is not a required promotion check

**High · Observed gap · Existing A-021/A-022.** [.github/workflows/hardware-evidence.yml](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/.github/workflows/hardware-evidence.yml) is only `workflow_dispatch`/`workflow_call`. It correctly searches for recent successful runs on an exact SHA, but source inspection found no automatic caller. The branch API reported `protected=false`, required status contexts empty, and the repository rulesets endpoint returned `[]` during this audit. These are observed policy settings, not a claim about who may change them.

**Consequence:** merging a change or passing portable/native compilation does not establish that the real GPU/RDMA paths passed. A queued or pending run supplies no such evidence. This audit cannot establish whether runner availability, concurrency or other scheduling conditions caused the waiting states.

**Remedy/acceptance:** define a release/promotion process that checks the intended exact revision and preserves actual hardware artifacts. Provide the necessary runners and bounded failure behavior. Apply required checks only through an authorized policy change; account for documentation-only commits and path-filtered workflows so routine PRs do not become impossible to merge. No policy was changed by this audit.
