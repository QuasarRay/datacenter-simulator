"""The first DCIT delivery slice: native CPU dependency-path diagnosis.

Expand one program and skill declaration into the course, project and independent
prompts. This does not mark the other DCIT source obligations delivered.
"""
import ast
import json
from verification.materials import LearningItem, learning_graph

SKILLS = ('resolve-intended-endpoints', 'bind-observation-to-interface',
          'use-unaffected-control', 'retain-failed-attempt', 'restore-owned-fault')

PROGRAM = '''from integrations.incus.diagnostics import Endpoint, Flow, path_study

@path_study
def dataset_dependency(source="compute-a", target="compute-b"):
    return (
        Flow(Endpoint(source, "data0"), Endpoint(target, "data0")),
        Flow(Endpoint("control", "data0"), Endpoint("services", "data0")),
    )

def investigate(fabric, output):
    # fabric is already compiled from NetBox and deployed by the trainer.
    # Planning resolves names to immutable NetBox IDs before effects.
    study = dataset_dependency(fabric)
    return study.run(output)
'''

COURSE = '''# DCIT path course · Follow one sealed parcel

This is a hands-on slice of D02, covering the adapted `2.3/server-fabric-packet-path`
obligation. Complete the NVIDIA core courses and C01's NetBox practical first.
The remaining D02 routing-protocol obligations still require their own adapters.

Think of a CPU-side dataset service feeding a GPU job. The GPU cannot consume the
dataset until its CPU dependencies can communicate. We will test that dependency
with a 32-byte unpredictable parcel and its SHA-256 acknowledgement. Passing this
experiment does not mean the GPU ran, the dataset was loaded, or RDMA worked.

The parcel belongs to one attempt. Its sender binds both a specific IPv4 address
and a specific Linux interface. The receiving process binds the destination
interface and address. It acknowledges only the correct parcel from the intended
source. An answer from a previous attempt cannot satisfy the current comparison.
The response port must also match. The counters belong to that exact interface.

Use a trainer-owned, already deployed `Fabric` from the C01 NetBox practical.
The site must contain four distinct hosts named `compute-a`, `compute-b`, `control`
and `services`, each with a selected NetBox `data0` port. Change these symbolic
names in the program to match your site; do not invent addresses or Incus names.
A switch connects the two paths. Each host has Python and the qualified native
image. Set the recipe's OS and image fingerprint explicitly for your CachyOS
profile. The separate GitHub Ubuntu fixture is not a CachyOS qualification.

The native lifecycle, daemon socket and evidence directory stay with the trainer.
Run this code in the same Python environment that has the pinned Ansible dependency.
Choose a new output path for every attempt. Do not place a learner inside a host
administrator account to make the example convenient.

```python
{program}

report = investigate(fabric, "/srv/ncp/evidence/path-001.json")
print(report["complete"], report["tier"], report["gpu_execution"])
# A successful native run prints: True emulated False
```

Read the program as a sequence of ownership transfers. `Endpoint` carries a
NetBox device name and port name. `Flow` pairs source and destination intent.
`@path_study` expands two flows into one reviewed observation procedure. Calling
the decorated function resolves and checks the plan. Calling `.run()` performs
the effects. Compilation does not send packets or disable a cable.

Before reading the result, predict each transition:

1. Establish host identities through the real pinned NVIDIA DeepOps hosts role.
   Apply the generated network configuration twice. Every host must report zero
   changes on the second application. This is the AIO reconciliation condition.
2. Resolve the NetBox port into the actual owned guest, interface and address.
   Check the four live instance ownership markers. This is an AII inventory and
   installation check at the emulation tier, not a physical adapter certificate.
3. Send the subject parcel and a separate control parcel. Require receiver and
   sender agreement plus increasing source TX and RX counters. This is the AIN
   path condition; a planned route alone cannot establish delivery.
4. Disable the source's single selected patchbay cable. The subject must fail
   while the control still succeeds. If both fail, the fault is not sufficiently
   localized. If neither fails, investigate an unintended path or bad intervention.
5. Restore that exact owned cable and repeat both transfers. A failed intermediate
   check still enters restoration. Read the incomplete report before another attempt.

```mermaid
flowchart TD
  B["Dispatch board: immutable endpoint IDs"] --> P["Seal parcel with a fresh nonce"]
  P --> H["Subject and control both deliver"]
  H --> C["Close one owned loading gate"]
  C --> Q{"Subject stops; control delivers?"}
  Q -->|yes| R["Reopen the same gate"]
  Q -->|no| F["Retain failed observations"]
  F --> R
  R --> V{"Both paths deliver again?"}
  V -->|yes| E["Complete this Ethernet experiment"]
  V -->|no| I["Keep the incident incomplete"]
```

The loading gate represents a Linux bridge/cable state, not an optical switch or
ASIC queue. The independent control represents a separate dependency path, not
proof that every tenant is healthy. The acknowledgement proves a bounded UDP
exchange; it cannot prove bulk throughput, absence of packet loss under load or
application correctness. Counters are supporting observations, not unique packet
identifiers. A configured socket that fails to run is a probe error, never the
expected network-failure observation.

Change only `source` and `target`, reverse their order, choose a new report path
and run again. Predict the source interface, selected cable and affected direction.
Then request a nonexistent NetBox port. Planning must reject it before any cable
operation. Reuse an earlier report path: it must be refused without overwriting
the evidence. These are executable ownership and immutability checks.

Read `checklist.records` beside `observations`. A complete report has all nine
named checks. The shared formally checked `report_complete` predicate rejects
missing or failed checks. Its proof assumes that the observations supplied by
the Python runner are truthful. The runner, operating system and network stack
are not covered by that scalar proof. The report always disables live mastery.

If the controller is forcibly terminated, inspect the journal, cable and
`state/diagnostic.lock` before recovery; a finally block cannot run after power
loss. A stale lock requires operator investigation. Destroying the lab uses its
owned lifecycle journal. Never delete an unrelated host bridge to clear an error.
'''

PROJECT = '''# DCIT path project · Recover a dependency without widening the outage

Assume the entire NVIDIA core has been studied and the path course has been
completed. This project practices every skill declared by that course. It
combines them across independent deployments; it introduces no new runtime API.

Build two small native fabrics from separate NetBox site scopes and disjoint
overlay pools using C01's `@native_fabric`. Keep their output directories and
ownership journals distinct. Each needs four host endpoints and a switch. Record
both image fingerprints and all selected NetBox IDs before deploying. Use
`Fabric.deploy()` and the course's generated `investigate` function throughout.

Your production-style deliverable is an incident bundle for each deployment:
source intent, the exact compiled plan, reconciliation evidence, attempted
intervention, subject/control observations, restoration result and the remaining
hardware/product gates. Do not call this a production-ready GPU platform merely
because both Ethernet experiments finish.

1. Predict which host is allowed to transmit the parcel in each fabric. Derive
   the endpoint names from each site's inventory; do not copy generated addresses
   from one deployment into another.
2. Run the course program in the first fabric. Bind its report to that fabric's
   `plan_identity`. Run the second using another report path. Explain why neither
   receipt can be used as the other's observation.
3. Reverse the subject direction in both fabrics. Predict the new source cable
   before executing. Compare the subject and unaffected control observations.
4. Preserve one intentional preflight rejection: an ambiguous or absent endpoint.
   Verify that the earlier receipts and active fabric still belong to their original
   attempts. Remove the bad input, not the guard.
5. Write a recovery decision that names the source-side fault boundary and the
   observations that would falsify it. Explain what further real workload and
   hardware evidence you need before resuming a GPU production job.

```mermaid
flowchart TD
  I["Two site scopes; two ownership journals"] --> A["Observe fabric A"]
  I --> B["Observe fabric B"]
  A --> J["Compare intent-bound receipts"]
  B --> J
  J --> D{"Does each direction retain its own source?"}
  D -->|no| X["Reject the incident bundle"]
  D -->|yes| R["Retain recovery evidence and open product gates"]
```

Acceptance requires the original NVIDIA reasoning in one causal argument. AII
establishes what was installed and which identities own the resources. AIN
establishes the actual data path and the intervention's scope. AIO establishes
reconciliation, observability and recovery. Omitting any of those makes the
integrated conclusion unsupported; reporting a partial checklist is diagnostic
feedback, not fractional mastery.

The private exercise delivery must provide independent fault scenarios and
collector identities. The public path program is a guided experiment; it is not
the private oracle and cannot award a Rustlings station receipt.
'''

INCIDENTS = (
    ('The intake stalled', 'A GPU batch is waiting for an input service. Deployment reports are green, but the declared dependency path is unavailable. Other users report normal service. Recover the intended dependency without opening a cross-tenant path. Preserve enough observations to separate a host identity mismatch, a forwarding failure and a misleading reconciliation report.'),
    ('The recovery widened the incident', 'A previous intervention restored one client while another dependency began failing. The two incident bundles refer to different site scopes. Determine which evidence actually describes each deployment, recover the affected paths and demonstrate that the untouched control remains healthy. Explain which conclusions still require GPU or storage-product observations.'),
)


def render(obligations):
    ast.parse(PROGRAM)
    skills = tuple('DCIT-PATH/' + skill for skill in SKILLS)
    base = ('nvidia-inventory', 'nvidia-network-evidence', 'nvidia-reconciliation')
    items = [LearningItem('DCIT-PATH-NVIDIA-core', 'foundation', 1, teaches=base),
             LearningItem('DCIT-PATH-course', 'course', 1, ('DCIT-PATH-NVIDIA-core',), teaches=skills, uses=base),
             LearningItem('DCIT-PATH-project', 'project', 1, ('DCIT-PATH-course',), uses=skills + base, practices=skills + base),
             LearningItem('DCIT-PATH-a', 'exercise', 1, ('DCIT-PATH-project',), uses=skills + base),
             LearningItem('DCIT-PATH-b', 'exercise', 2, ('DCIT-PATH-project', 'DCIT-PATH-a'), uses=skills + base)]
    learning_graph(items, obligations)
    result = {'dcit-path/course.md': COURSE.replace('{program}', PROGRAM.strip()),
              'dcit-path/project.md': PROJECT, 'dcit-path/example.py': PROGRAM}
    for suffix, (title, incident) in zip(('a', 'b'), INCIDENTS):
        result[f'dcit-path/incident-{suffix}.md'] = f'''# Independent incident {suffix} · {title}

{incident}

Use the code interfaces practiced in the course and project. The trainer supplies
the incident's NetBox scope, owned workspace and permitted observation interfaces.
Submit an executable repair, before/after evidence, an unaffected control and an
explanation that joins AIO operations, AIN paths and AII installed resources.
Do not edit desired NetBox intent merely to agree with the broken observed state.

Delivery status: this is an authored independent prompt. Private scenario seeding,
holdout collectors and Rustlings grading for this extension are not yet qualified.
The public diagnostic program does not issue a mastery receipt. Do not present a
guided cut/restore run as completion of this independent incident.
'''
    result['dcit-path/delivery.json'] = json.dumps({'schema': 1, 'source_facet': '2.3/server-fabric-packet-path',
        'core_prerequisites': [f'C{i:02}' for i in range(1,21)], 'skills': list(skills),
        'course': 'native-practical', 'project': 'guided-native-practical',
        'exercises': 'prompts-awaiting-private-scenarios', 'live_mastery_enabled': False}, indent=2) + '\n'
    return result
