# NCP-Metablueprint implementation and qualification

The implementation is available for review. Full live mastery delivery and a
whole-application formal proof are **not complete**. A passing planning example,
coverage link or synthetic Kubernetes status does not close those gaps.

## Delivered artifacts

| Artifact | Location | Acceptance boundary |
|---|---|---|
| Source union: 113 objective identities, 203 facets | [Blueprint](education/ncp-metablueprint/README.md), [coverage ledger](education/ncp-metablueprint/coverage.md) | Includes differing AIO webpage/guide objectives; mappings are obligations |
| Progressive learning material | [Learning route](education/ncp-metablueprint/LEARNING_ROUTE.md) | C00 plus 20 courses, 20 projects, 40 independent incidents, 20 planning examples |
| Code-first topology realization | [NetBox practical](education/ncp-metablueprint/courses/C01-NETBOX.md) | Read-only NetBox → validated petgraph → generated Incus/patchbay/network/DeepOps configuration |
| Black-box learner delivery | [Assessment](education/assessment/README.md) | Rustlings plus Python controllers; private attempt-bound observations and trainer progression |
| Formal decision source | [Pinned shared kernel](education/rustlings/lab-kernel/README.md) | Seven exact production functions checked with both Verus and Kani |
| Native runtime contract | [Provisioning](integrations/incus/PROVISIONING.md) | Explicit OS/image identity, ownership checks, journaled lifecycle and separate product gates |
| Runtime capability adaptation | [Rusternetes profile](integrations/rusternetes/README.md) | No workload fabrication; runtime subresources return 501; TLS plus bearer RBAC |

The assessment compiler contains 40 station contracts and 510 facet observations
per baseline/holdout traversal, including intentional companion coverage. Every
station requires AIO, AIN, AII and a coupled outcome. Both incidents are required
for one indivisible unit; complete mastery requires all 20 units. Passing one or
two source-domain checks grants no unit.

## Evidence inspected

- [Verus and Kani run](https://github.com/QuasarRay/rustlings/actions/runs/35989371679)
  passed all seven kernel functions/harnesses. Reservation proof preceded its use
  in topology memory/address admission. Parsers, filesystem operations, observers,
  Rustlings UI, NetBox, petgraph, Incus and NVIDIA products remain outside this proof.
- [Native Incus run](https://github.com/QuasarRay/datacenter-simulator/actions/runs/35989381189)
  passed two concurrent five-node labs, real DeepOps reconciliation, second-run
  idempotency, routed packets, cable cut/restoration, isolation, foreign-member
  teardown refusal and repeated owned cleanup. This is Linux Ethernet evidence.
- The initial [native control-plane run](https://github.com/QuasarRay/datacenter-simulator/actions/runs/35989026291)
  built the three Rusternetes binaries and KWOK, and exercised API authentication,
  unavailable runtime responses and selected-node KWOK transitions. Its retained
  report exposed an incomplete checklist despite a green job. PR #32 adds the
  missing explicit CRUD checks and makes incomplete reports fail. Inspect the
  current report's `complete` field, not only the workflow color.
- [NetBox native qualification](https://github.com/QuasarRay/datacenter-simulator/pull/31)
  installs exact native NetBox/PostgreSQL/Redis/Gunicorn on a disposable host and
  tests the full API-to-Incus-to-DeepOps-to-packet path. Its generated image and
  binary identities are retained with that run, not generalized to another host.

## Remaining release gates

1. Inspect successful complete reports for the current NetBox and control-plane
   gates. Scheduler/controller binaries being built does not prove their complete
   reconciliation semantics, API conformance or NVIDIA Operator compatibility.
2. Qualify the combined deployment inside the chosen Incus image: native service
   asset installation, authentication, restart behavior, KWOK pod/lease recovery,
   Slurm, storage services and workload adapters. The Linux Ethernet lifecycle and
   separate API probe are useful evidence, not an integrated production acceptance.
3. Implement and qualify the private station collectors against actual required
   products and hardware. BCM, Mission Control/Base View, Run:ai, UFM, NetQ, Air,
   DPU/GPU/NVSwitch, real RDMA/GDS, physical service tasks and external Docker course
   evidence cannot be replaced by constant success or renamed simulation. The
   shipped collector profile stays unqualified; no live mastery is awarded.
4. Extend formal specifications beyond the seven-function kernel to the complete
   runner, bank parser and orchestration boundaries. The existing all-facet
   mutation checks are tests, not a proof of all Rustlings or the entire lab bank.
5. Have instructors verify semantic teaching sufficiency and every live practical.
   Source-facet and prerequisite-closure checks detect omissions in the registry;
   they cannot establish that a student has learned a physical procedure.

## Durable review stack

Simulator PRs [#21](https://github.com/QuasarRay/datacenter-simulator/pull/21)
through [#32](https://github.com/QuasarRay/datacenter-simulator/pull/32) are incremental
checkpoints. The aggregate PR from the current tip to `main` contains the whole
stack for one repository merge after its release gates are satisfied. No claim is
made that GitHub provides an atomic multi-repository merge.

Companion Rustlings PRs [#52](https://github.com/QuasarRay/rustlings/pull/52),
[#53](https://github.com/QuasarRay/rustlings/pull/53),
[#54](https://github.com/QuasarRay/rustlings/pull/54) add external assessment,
Python watch support and verified reservations. Patchbay PRs
[#3](https://github.com/QuasarRay/patchbay/pull/3) and
[#4](https://github.com/QuasarRay/patchbay/pull/4) supply owned native cables.
This repository pins their exact commits, so its checkout does not depend on
following a moving companion branch.

The user's full directives are already on `main` in every one of its 92 tracked
directories, including the new NCP paths. Implementation changes remain in the
review stack. External submodule source is not rewritten to duplicate root policy.
