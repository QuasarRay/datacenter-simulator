# Evidence and reproduction

Release status: **STATIC_AND_FIXTURE_VALIDATED_RUNTIME_GATED**, recorded 2026-09-13. The checks below actually ran in the local Python/Ansible environment. Docker, Containerlab, a RHEL VM and an AAP Controller were unavailable here.

| Evidence | Observed result | Practical limit |
|---|---|---|
| [Validation report](validation.json) and [component lock](../component-lock.json) | All 23 available check groups passed | Parsing and synthetic execution do not prove live interoperability |
| [Unit transcript](logs/predicate_unit_tests.log) | 38 tests passed | Includes peer sets, interface placement, BGP/Zebra fields, numeric packet loss and Ansible integer compatibility |
| [Actual module argument checks](logs/actual_module_arguments.log) | 45 rendered cases passed; unknown-argument mutations rejected | Uses installed collection argument specs; not Controller server-side schema validation |
| [Ansible dataflow report](logs/ansible_synthetic_dataflow.log) | Five scenarios passed: good state accepted; four targeted faults rejected | Actual shipped verifier tasks, with device I/O replaced by synthetic output |
| Ansible syntax logs | 13 playbook entry points parsed | Includes the root EE entry point and Controller configuration/launch |
| Navigator / Builder logs | Host inventory, settings and EE definition accepted | No EE image was built, pushed or pulled |
| [Preparation smoke test](preparation-smoke.json) | Repeated preparation preserves four host identities and renders four links | Test keys were temporary and deleted; no SSH connection |
| [Input provenance](input-provenance.json) | Model bytes match the pinned upstream Git blob | Identifies inputs; does not certify their behavior |
| [Public blueprint map](../00_blueprint_map.md) | 23/23 objectives across eight families have instruction, exercises and acceptance criteria | Content coverage of the dated public blueprint, not proof of exam mastery |
| [Source matrix](content-to-sources.md) and [review record](source-review.md) | 38 section claims mapped across 40 official documentation/upstream sources | Manual semantic review; adaptations and community support scope are explicit |
| [Audit dispositions](audit-closure.md) | All 48 supplied findings accounted for | Runtime-dependent findings remain gated; none claims runtime closure |

The archived JSON reports retain original command paths and `.state` log paths as provenance. Their `archived_log` fields point to included copies. Dataflow scenario logs are also included in `evidence/logs/` with their original basenames. Logs contain synthetic device state, not captured live router sessions.

From the pack root, after installing the pinned Python requirements and collections:

```bash
export ANSIBLE_COLLECTIONS_PATH="$PWD/collections"
python tools/validate.py --full --manifest
```

The validator reruns checks and writes new local logs under `.state/`; it does not rewrite the archived release evidence. The unsigned SHA-256 manifest checks byte integrity only. It excludes itself, runtime state, secrets, downloaded dependencies and generated build context. A clean extraction can run basic checks and verify the manifest with `python tools/validate.py --manifest`.

Live acceptance requires all rows in [chapter 20](../20_runtime_release_gate.md), including a rejected wrong SSH host key, exact route/forwarding behavior, restart without reconfiguration, usable restore, EE distribution, Controller node outcomes, durable backups and RHEL profile persistence. Record actual runtime versions, schemas and outputs before changing any live status to passed.
