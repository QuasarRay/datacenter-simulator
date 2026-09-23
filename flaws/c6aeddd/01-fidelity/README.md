# NVIDIA hardware and datacenter fidelity

Audited revision: `c6aedddfbb9ac569b5e0eb68acc616452bf66136` (2026-09-23). Documentation only.

Priority is relative to accurate NVIDIA infrastructure simulation on consumer hardware, not a security score. **Confirmed** means a code path, local reproduction or CI observation establishes the behavior. **Gap** is a known model boundary. **Risk** identifies an unverified failure scenario. Existing limitations are not counted as newly introduced defects.

| ID | Priority / classification | Finding | Report |
|---|---|---|---|
| FID-01 | High / gap | Host/GPU/NIC/NUMA/rail hierarchy is absent | [Placement and transport](placement-and-transport.md) |
| FID-02 | High / gap | Native NCCL is deliberately restricted to Socket | [Placement and transport](placement-and-transport.md) |
| FID-03 | High / gap | Message timing and static routes cannot predict packet fabrics | [Network and verbs](network-and-verbs.md) |
| FID-04 | Medium / gap, local probe | Software RC lacks hardware retry/completion semantics | [Network and verbs](network-and-verbs.md) |
| FID-05 | High / gap | Storage, power, thermal and BMC behavior is metadata only or absent | [Datacenter scope](datacenter-scope.md) |
| FID-06 | Medium / gap | Small fixed collective tests do not cover training workloads | [Datacenter scope](datacenter-scope.md) |

These extend the existing G001–G006 architectural follow-ups in `docs/audit-6311-remediation.md`; they do not reopen the 20 implementation fixes merged in PR #12. Preserve genuine NCCL execution and explicitly bounded analytical modes when addressing them. A CPU substitute must not be reported as native CUDA/NCCL evidence.
