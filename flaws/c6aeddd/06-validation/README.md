# CI evidence, fault tests and calibration

Audited revision: `c6aedddfbb9ac569b5e0eb68acc616452bf66136` (2026-09-23). Documentation only.

Priority is relative to accurate NVIDIA infrastructure simulation on consumer hardware, not a security score. **Confirmed** means a code path, local reproduction or CI observation establishes the behavior. **Gap** is a known model boundary. **Risk** identifies an unverified failure scenario. Existing limitations are not counted as newly introduced defects.

| ID | Priority / classification | Finding | Report |
|---|---|---|---|
| VAL-01 | High / observed CI failure | Current main fails the legacy fabric recovery ping | [CI snapshot](ci-snapshot.md) |
| VAL-02 | High / observed evidence/policy gap | Hardware runs have not passed; promotion gate is not automatically enforced | [CI snapshot](ci-snapshot.md) |
| VAL-03 | Medium / reproduced validator weakness and causal risk | Nonzero fault-test exits do not prove the intended data-plane failure | [Fault tests](fault-tests.md) |
| VAL-04 | High / existing validation gap | No calibrated NVIDIA accuracy or consumer-scale envelope is established | [Calibration](calibration.md) |

CI states are timestamped observations, not permanent claims. These documentation changes do not disable checks, change branch policy, re-run hardware workloads or mark pending evidence as passed.
