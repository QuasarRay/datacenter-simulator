# VAL-04 — Accuracy and efficiency need independent measured envelopes

**High · Existing validation gap.** The checked contracts prove selected invariants and real positive paths; they do not establish a calibrated NVIDIA datacenter model or a consumer-hardware capacity claim. [simulator/examples/model_scale.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/examples/model_scale.rs) measures construction, not native topology, sustained traffic, full memory use or a training workload. Current fixtures cannot resolve the fidelity gaps simply by adding more assertions against the same implementation.

## What was actually checked in this audit

| Validation | Outcome / limit |
|---|---|
| `cargo test --locked --manifest-path simulator/Cargo.toml --all-targets --features vm,ibsim,nccl-check,netbox,mokka` | 80 top-level tests passed. An internal child reruns one filtered test; do not count it twice. `nccl-check` cannot execute real collectives |
| v6 `pytest tests -q` in its own directory with the configured Ansible environment | 91 passed, 1 skipped because this container lacks the OpenSSH privilege-separation capability |
| Public-API release probe | Queue/history trends, checkpoint-copy cost, shutdown refusal, clone policy and RNR behavior observed |
| Real CLI with isolated fake tools | Duplicate/wrong-model Mokka inventory accepted; doctor capability predicate isolated |
| Timed identity-read child | FIFO blocked before regular-file rejection; terminated after one second |
| Isolated current traffic-test method | Missing-executable negative probe accepted |
| GitHub run/job/log review | See the exact-SHA snapshot; credit successful native namespace, CPU VM, NetBox/IB and Kind jobs |
| Local GPU, RNIC, VFIO, QEMU fleet or cluster execution | Not performed for these counterexamples; no new native success claimed |

## Closure matrix

| Target claim | Required independent evidence |
|---|---|
| NVIDIA network timing | Match path, rate, MTU, traffic distribution and topology to a hardware reference; publish error versus message size and load |
| Collective performance | Record GPU/NIC/NUMA placement, NCCL version/transport, algorithm/protocol, data type, warmup, variance and timing scope |
| Realistic failure recovery | Timed faults with independent control connectivity, transport-specific diagnostics, positive recovery and verified cleanup |
| Consumer-host efficiency | Peak RSS, CPU time, disk allocation, process/FD/namespace counts and wall time for increasing logical/native scale |
| 100–1,000 logical devices | Demonstrate admission, create/change/delete cycles and retained history/checkpoints on an explicitly sized host; do not substitute the 4,096-node constant |

Use deterministic seeds and small analytical reference cases for CPU regressions, then retain sampled hardware traces for calibration. Test multiple traffic patterns and long-lived runs. Report uncertainty/error bars and the applicability range. A validated small native slice plus efficient abstract large-scale model is a feasible architecture; claiming identical fidelity from all modes is not supported by the current evidence.
