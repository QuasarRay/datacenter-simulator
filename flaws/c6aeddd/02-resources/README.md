# Portable resource use and lifecycle budgets

Audited revision: `c6aedddfbb9ac569b5e0eb68acc616452bf66136` (2026-09-23). Documentation only.

Priority is relative to accurate NVIDIA infrastructure simulation on consumer hardware, not a security score. **Confirmed** means a code path, local reproduction or CI observation establishes the behavior. **Gap** is a known model boundary. **Risk** identifies an unverified failure scenario. Existing limitations are not counted as newly introduced defects.

| ID | Priority / classification | Finding | Report |
|---|---|---|---|
| RES-01 | Medium / confirmed, measured | Every send/write clones complete QP queues | [Hot paths](hot-paths.md) |
| RES-02 | Medium / confirmed, measured | Full history shifts a vector on every event | [Hot paths](hot-paths.md) |
| RES-03 | High / confirmed, measured | Small lifecycle edits clone all retained checkpoints | [Budgets and lifecycle](budgets-and-lifecycle.md) |
| RES-04 | High / existing gap | Count/byte limits do not admit a whole run against host resources | [Budgets and lifecycle](budgets-and-lifecycle.md) |
| RES-05 | Medium / confirmed, reproduced | A full retained-data budget can prevent shutdown | [Budgets and lifecycle](budgets-and-lifecycle.md) |
| RES-06 | Medium / confirmed behavior, policy gap | Clone resets configured limits | [Budgets and lifecycle](budgets-and-lifecycle.md) |

[Reproduction code and measurements](reproduction.md) are CPU-only. Measurements are single-run examples on the audit container, not consumer-laptop or native-fabric capacity guarantees. PR #12's construction-accounting optimization remains in place; these are different hot paths.
