# First-run setup and prerequisite discovery

Audited revision: `c6aedddfbb9ac569b5e0eb68acc616452bf66136` (2026-09-23). Documentation only.

Priority is relative to accurate NVIDIA infrastructure simulation on consumer hardware, not a security score. **Confirmed** means a code path, local reproduction or CI observation establishes the behavior. **Gap** is a known model boundary. **Risk** identifies an unverified failure scenario. Existing limitations are not counted as newly introduced defects.

| ID | Priority / classification | Finding | Report |
|---|---|---|---|
| SET-01 | Medium / confirmed predicate mismatch | Doctor rejects the documented unprivileged namespace path | [Doctor](doctor.md) |
| SET-02 | Medium / confirmed scope mismatch | Doctor checks do not match all actual execution prerequisites | [Doctor](doctor.md) |
| SET-03 | Medium / existing setup gaps | Build modes, native pins, platform scope and VM cost still require manual assembly | [Remaining setup work](remaining-setup.md) |

PR #12 already added successful help, root Rustup pinning and a backend/resource setup guide. Those improvements are present at this baseline. The remaining issues should be addressed without making the portable path install CUDA, VMs, Kubernetes or Python dependencies unnecessarily.
