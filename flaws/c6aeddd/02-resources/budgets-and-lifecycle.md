# Budget scope and lifecycle behavior

## RES-03 — Transactional edits duplicate unchanged retained state

**High · Confirmed and measured.** [simulator/src/limits.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/limits.rs#L90) clones the entire `Simulation`, applies one change, recomputes serialized usage, then replaces the original. Checkpoints contain cloned runtime/configuration values and are cloned again by this transaction. Resetting one node with `rebuild=false` therefore copies all checkpoint payloads and history even though it only changes generation/state bookkeeping.

With a 1-MiB runtime file, a one-node reset took 777 µs without checkpoints, 8,674 µs with eight checkpoints (16,793,408 retained checkpoint bytes), and 16,488 µs with sixteen (33,586,816 bytes). These are latency measurements; peak RSS was not instrumented. The simultaneous original/candidate ownership is established by source, not an inferred RSS number.

**Remedy/acceptance:** preflight exact deltas and stage touched objects, or share immutable checkpoint contents. Keep PR #12's atomic budget enforcement. A reset that does not modify checkpoint contents should not copy their payload; inject errors to verify no partial commits.

## RES-04 — Per-object limits leave whole-host admission unresolved

**High · Gap · Existing G007/G008.** [simulator/src/api.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/api.rs#L26) allows 128 simulations; [simulator/src/limits.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/limits.rs#L40) allows 32 MiB data plus 64 MiB checkpoints **per simulation**. That alone permits 12 GiB of accounted serialized payload across a default API instance, before Rust objects, history, transient clones or separate `Fabric` allocations. Serialized size is explicitly not RSS.

[simulator/src/linux.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/linux.rs#L72) creates an additional router/bridge per link and installs per-destination routes using subprocesses. A connected undirected graph with 240 links can have at most 241 vertices; the portable 4,096-node limit cannot describe connected native capacity. Native construction scans interfaces/links repeatedly and programs approximately N×(N−1) host routes for a fully reachable N-node topology.

VM plans validate each guest but not the host's available RAM/CPU/disk budget. The checked DeepOps example requests 20 GiB guest RAM and 12 vCPUs. Per-file 16-MiB VM/Mokka logs are not an aggregate run-disk budget. RDMA polling uses `yield_now`, not a sleep/event wait, and can consume a core while waiting. None of these proves that a run fits a consumer machine.

**Remedy/acceptance:** add an explicit run admission report with estimates/reservations for peak RAM, CPU, processes, namespace/route counts, logs and disk, plus configurable refusal thresholds. Enforce aggregate ownership budgets before creating native resources. Benchmark connected realistic topologies at the intended scale; report virtual disk capacity separately from actual allocated bytes.

## RES-05 — Budget saturation can block a resource-releasing transition

**Medium · Confirmed, reproduced.** [simulator/src/api.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/api.rs#L876) runs through the same retention transaction. Changing serialized `ACTIVE` to `INACTIVE` and incrementing generations can increase accounted size. With the current active usage accepted as the limit, `shutdown(false)` fails and leaves the model active:

```text
Err(Conflict("retained data bytes capacity exceeded (883)"))
state = Active
```

This is not a request to restore the old quota bypass. It is a lifecycle liveness problem: [simulator/src/api.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/api.rs#L216) propagates the same shutdown error when a scheduled sleep is due. Native cleanup behavior is separate; this probe establishes model behavior only.

**Remedy/acceptance:** reserve bounded transition metadata at admission, use fixed-size accounting for lifecycle fields, or define a cleanup reserve that cannot store arbitrary payload. Full-budget shutdown without a checkpoint and scheduled sleep must succeed while user data remains within its permitted budget; checkpoint creation may still fail explicitly.

## RES-06 — Clone restores default limits rather than the source's policy

**Medium · Confirmed behavior; policy gap.** [simulator/src/api.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/api.rs#L121) exports/imports a manifest, which does not contain limits. A source with `limits.interfaces = 1` produces a clone with `limits.interfaces = 32768`. `history_limit` also comes from new-simulation defaults. Custom limits therefore cannot be assumed to follow clones.

**Remedy/acceptance:** choose and document an explicit inheritance policy, or require destination budgets on clone/import. Test a constrained source and a source larger than default limits. Cloning should neither silently expand an operator's intended resource envelope nor unexpectedly fail only because custom larger limits were lost. This is not an access-control bypass: the same trusted Rust caller can already change its own limits.
