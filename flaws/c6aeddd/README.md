# Datacenter simulator audit — c6aeddd

Source: [`c6aedddfbb9ac569b5e0eb68acc616452bf66136`](https://github.com/QuasarRay/datacenter-simulator/commit/c6aedddfbb9ac569b5e0eb68acc616452bf66136), audited 2026-09-23. Main already contained PR #12's remediation; this collection does not present those 20 prior fixes as new open bugs.

The goal is accurate NVIDIA GPU datacenter infrastructure behavior at a cost compatible with consumer hardware. The current project supplies useful bounded control/analytical APIs and several real execution paths. It does not yet establish calibrated whole-datacenter behavior, and there are additional reproducible resource and validation problems.

## Collection and review order

There are **28 recorded items** across six focused documentation PRs: confirmed defects/behaviors, existing model gaps and explicitly unverified risks. They are not 28 newly reproduced bugs. Each slice is based independently on the audited main commit and changes only its own directory. The links below work before the sibling PRs merge; each directory can be reviewed/merged independently. This index is a separate small PR.

| Area | Items | Pull request | Read the exact report revision |
|---|---:|---|---|
| 01-fidelity | 6 | [PR #13](https://github.com/QuasarRay/datacenter-simulator/pull/13) | [Pinned reports](https://github.com/QuasarRay/datacenter-simulator/blob/fe1055de71cced3efac9a25f5beb4c69ac7a0078/flaws/c6aeddd/01-fidelity/README.md) |
| 02-resources | 6 | [PR #14](https://github.com/QuasarRay/datacenter-simulator/pull/14) | [Pinned reports](https://github.com/QuasarRay/datacenter-simulator/blob/8b30bea7c517d2f769b12ca09633d6e8cd827900/flaws/c6aeddd/02-resources/README.md) |
| 03-native-runtime | 4 | [PR #15](https://github.com/QuasarRay/datacenter-simulator/pull/15) | [Pinned reports](https://github.com/QuasarRay/datacenter-simulator/blob/1dd89f120a43fb5b4a40e2ec5821f302c694d8c9/flaws/c6aeddd/03-native-runtime/README.md) |
| 04-integrations | 5 | [PR #16](https://github.com/QuasarRay/datacenter-simulator/pull/16) | [Pinned reports](https://github.com/QuasarRay/datacenter-simulator/blob/aa1e16037416b0171eff4b0c3ca5fa943ed1fbce/flaws/c6aeddd/04-integrations/README.md) |
| 05-setup | 3 | [PR #17](https://github.com/QuasarRay/datacenter-simulator/pull/17) | [Pinned reports](https://github.com/QuasarRay/datacenter-simulator/blob/ba8d019f1e21b775d4e814da739bf81051e5d61b/flaws/c6aeddd/05-setup/README.md) |
| 06-validation | 4 | [PR #18](https://github.com/QuasarRay/datacenter-simulator/pull/18) | [Pinned reports](https://github.com/QuasarRay/datacenter-simulator/blob/52b4ebbabf0810054ce2a2195e05019e31cabde3/flaws/c6aeddd/06-validation/README.md) |

Each item supplies source evidence, impact, a proposed correction or modeling decision, and acceptance criteria. Priority is relative to the stated simulator goal, not CVSS: 13 High, 15 Medium. An architectural gap can be high priority without being a regression or a misleading current implementation claim.

## Most actionable observations

1. `shutdown(false)` can be refused at a full accepted data budget, leaving the model active; custom limits also revert to defaults on clone.
2. Software sends copy complete QP queues, and event retention shifts a full vector. A 512-send probe grew from 681 µs at receive depth 1 to 593,854 µs at depth 32,768 on this runner.
3. A one-node lifecycle edit clones retained checkpoint payloads. Per-simulation serialized-byte bounds are not a whole-host memory/CPU/disk admission policy.
4. The real Mokka CLI accepted duplicate `GPU-DUPLICATE` identities labeled `Not-a-T4` for a two-T4 plan in an isolated fake-tool reproduction.
5. Doctor rejects the documented pre-bootstrap unprivileged namespace path. Blocking input/helper paths and outer-future cancellation leave deadline/cleanup gaps.
6. Current-main datacenter CI failed its post-partition recovery ping. Hardware workflows were waiting at the recorded observation; their outcomes must be refreshed before promotion.

Fix these bounded implementation/validation problems first. In parallel, define typed host/GPU/NIC/rail placement and explicit accuracy contracts for each mode before extending transport or workload claims. Do not make setup require a full VM fleet for scenarios the portable or namespace modes can cover.

## Evidence and limits

The audit ran 80 top-level Rust tests with `vm,ibsim,nccl-check,netbox,mokka`, and 91 v6 Python tests with one capability-dependent skip. It executed focused public-API probes and isolated real-CLI/predicate counterexamples, checked upstream NCCL test semantics, and reviewed exact-SHA GitHub run/job/log evidence. Successful remote namespace, CPU-VM, NetBox/IB and Kind jobs are credited explicitly.

No new local GPU, physical RNIC, VFIO, VM fleet or Kubernetes success is claimed. Static cancellation/host-cleanup risks are not represented as observed leaks. Performance numbers are single-run illustrations, not calibrated hardware predictions or a laptop capacity guarantee. No finite audit can establish that nothing remains undiscovered; [coverage](coverage.md) records the actual surfaces, exclusions and required next evidence.

See [primary sources](sources.md) for the technical references. All files are documentation; the collection neither implements the proposed fixes nor changes branch protection or deployment state.
