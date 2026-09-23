# NetBox, Mokka and cross-backend integration

Audited revision: `c6aedddfbb9ac569b5e0eb68acc616452bf66136` (2026-09-23). Documentation only.

Priority is relative to accurate NVIDIA infrastructure simulation on consumer hardware, not a security score. **Confirmed** means a code path, local reproduction or CI observation establishes the behavior. **Gap** is a known model boundary. **Risk** identifies an unverified failure scenario. Existing limitations are not counted as newly introduced defects.

| ID | Priority / classification | Finding | Report |
|---|---|---|---|
| INT-01 | Medium / confirmed, reproduced | Duplicate UUIDs and wrong GPU models pass Mokka inventory validation | [GPU inventory](mokka-inventory.md) |
| INT-02 | High / scope gap | An agent-local NVML probe does not verify Kubernetes GPU allocation | [GPU inventory](mokka-inventory.md) |
| INT-03 | High / existing gap | Shared intended state does not couple live traffic or failures | [Intent and ownership](intent-and-ownership.md) |
| INT-04 | Medium / scope gap | NetBox import is a selected-port graph, not a hardware inventory | [Intent and ownership](intent-and-ownership.md) |
| INT-05 | Medium / risk | Helm rollback is not verified restoration of node host state | [Intent and ownership](intent-and-ownership.md) |

Current main CI has passing real NetBox/ibsim and Kind/NVML jobs. Those positive results are credited; they do not close the negative-input and cross-plane gaps below. The inventory reproduction uses isolated fake external tools and never contacts a cluster.
