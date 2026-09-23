# Native deadlines, cancellation and diagnostics

Audited revision: `c6aedddfbb9ac569b5e0eb68acc616452bf66136` (2026-09-23). Documentation only.

Priority is relative to accurate NVIDIA infrastructure simulation on consumer hardware, not a security score. **Confirmed** means a code path, local reproduction or CI observation establishes the behavior. **Gap** is a known model boundary. **Risk** identifies an unverified failure scenario. Existing limitations are not counted as newly introduced defects.

| ID | Priority / classification | Finding | Report |
|---|---|---|---|
| NAT-01 | High / confirmed blocking paths; FIFO reproduced | Async deadlines do not bound blocking helpers or input opens | [Deadlines](deadlines.md) |
| NAT-02 | High / source-confirmed contract gap; GPU reuse not reproduced | External NCCL cancellation skips awaited cleanup | [Cleanup](cleanup.md) |
| NAT-03 | Medium / source-confirmed contract gap | VM shutdown has unbounded waits and discards errors | [Cleanup](cleanup.md) |
| NAT-04 | Medium / risk from discarded completion handles | QEMU log completion/errors are not observed | [Cleanup](cleanup.md) |

NAT-01/NAT-02 carry forward R002/R001 from the preceding audit, now with a concrete FIFO probe. Ordinary NCCL error/timeout cleanup, Mokka process-group handling, and retryable IB shutdown have already been improved; these reports identify the remaining paths. They do not claim an observed GPU or QEMU leak on this runner.
