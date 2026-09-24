"""Render authored content and exact facet/skill links, never completion evidence."""
import json
from pathlib import Path
import sys

ROOT=Path(__file__).resolve().parents[1]

DIAGRAMS={
6: '''flowchart TD
  S["Sender: offered load"] --> Q["Reservoir: selected priority queue"]
  Q -->|forward| R["Receiver: useful workload progress"]
  Q -->|mark| E["ECN feedback loop"]
  E -->|reduce offered load| S
  Q -->|threshold reached| P["PFC: pause upstream priority"]
  P -->|temporary link backpressure| S
  R --> T{"Progress and recovery bound met?"}
  T -->|no| D["Reject policy; retain observations"]''',
8: '''flowchart TD
  H["Host control: host driver"] -->|supported control operation| N["NIC datapath"]
  A["Arm control: DOCA service"] -->|mode-dependent operation| N
  W["Workload buffers"] -->|data path| N
  N --> F["Fabric endpoint"]
  H --> V{"Compatible versions and authority?"}
  A --> V
  V -->|no| R["Reject rollout"]
  V -->|yes| O["Observe process and packet evidence"]''',
9: '''flowchart TD
  I["Intent: object specification"] --> A["Rusternetes API and state"]
  A --> C["Controllers and scheduler"]
  C -->|assigned synthetic work| K["KWOK: simulated worker status"]
  K -->|status only| A
  I -->|separate native service plan| N["Incus: actual process"]
  N --> R["Measured response and device use"]
  A --> Q{"State and execution both established?"}
  R --> Q
  Q -->|no| B["Block workload acceptance"]''',
12: '''flowchart TD
  W["Workload owners"] --> D{"All affected users drained?"}
  D -->|no| B["Block reconfiguration"]
  D -->|yes| G["GPU: model-specific MIG catalogue"]
  G --> I["GPU instance: supported profile and placement"]
  I --> C["Compute instance: compute subdivision"]
  C --> N["New device identities"]
  N --> A["Scheduler rediscovery and workload acceptance"]
  A -->|stale identity or failed result| B''',
13: '''flowchart TD
  F["Storage object: intended bytes"] --> P{"Supported direct path?"}
  P -->|yes| D["DMA to registered GPU buffer"]
  P -->|documented fallback| H["Host staging path"]
  H --> G["GPU destination buffer"]
  D --> G
  G --> C{"Required completion observed?"}
  C -->|no| W["Keep owner and registration alive"]
  C -->|yes| V["Validate and consume bytes"]
  V --> R["Release according to API lifetime rules"]''',
14: '''flowchart TD
  M["Rank map and buffer shapes"] --> V{"Valid devices and participation?"}
  V -->|no| X["Reject before allocation"]
  V -->|yes| N["Native NCCL execution"]
  N --> L["Local path: NVLink or supported fallback"]
  N --> E["Remote path: NIC and fabric"]
  L --> C["CUDA completion and result check"]
  E --> C
  C --> O["Record actual transport and timing"]''',
}

def diagram(n,d):
    if n in DIAGRAMS:return DIAGRAMS[n]
    a,b,c=d['symbols']
    return f'''flowchart TD
  I["{a}"] --> V{{"Identity, units and authority valid?"}}
  V -->|no| R["Reject before mutation"]
  V -->|yes| P["{b}"]
  P --> A["Apply bounded transition"]
  A --> O["{c}"]
  O --> C{{"Independent result matches contract?"}}
  C -->|yes| H["Repeat on holdout"]
  C -->|no| D["Drain, diagnose and recover"]
  D --> I
  H -->|changed topology or event order| V'''

def render(bp, authored):
    outputs = {}
    objects={o['id']:o for o in bp['objectives']}
    entries=[]; prior_skills=[]
    index=['# Learning route','',
      'Start with [the operator and visual language](courses/C00.md). Complete C01–C20 before P01. '
      'Projects assume the whole blueprint; exercises begin after their corresponding projects. '
      'The written route and planning examples are available. Live NVIDIA product qualification '
      'and a privileged Incus host remain required for full mastery.','',
      '| Stage | Course | Guided project | Independent incidents |', '|---|---|---|---|']
    for u in bp['units']:
        n=u['level']; d=authored[n]; cid=u['course']; pid=u['project']
        uses=list(prior_skills)
        taught=[f'{cid}.S{i:02}' for i in range(1,len(d['skills'])+1)];prior_skills+=taught
        rows=[]; faceted=[]
        for oid in u['objectives']:
            o=objects[oid]
            for facet in o['facets']:
                fid=oid+'/'+facet;faceted.append(fid)
                rows.append(f'| `{fid}` | {o["exam"]}: {o["label"]} — {facet} | A versioned action, independent observation, and rejected counterexample for this specific facet. |')
        table='\n'.join(['| Facet identity | Required skill | Course-to-assessment obligation |','|---|---|---|']+rows)
        code=d['code']
        outputs[f'examples/{cid}.py'] = code
        previous='C00' if n==1 else f'C{n-1:02}'
        course=f'''# {cid} · {u['title']}

Prerequisite: [{previous}]({previous}.md). New skills: {', '.join(d['skills'])}.
This course teaches these skills through a guided build. Model examples predict behavior;
the live steps establish behavior on the named execution tier.

## State, ownership and the reason for the rule

{d['semantics']}

## Trace the transition

The symbols name physical objects beside their actual technical referents. Solid
arrows show the labeled dependency or data path, not an implicit synchronous barrier.
The drawing describes this lesson's contract; it is not an undocumented NVIDIA
internal implementation diagram.

```mermaid
{diagram(n,d)}
```

## Build it in code

Start the qualified native lab with `Session.start("{cid}")` as described in C00.
That operation reconciles real pinned DeepOps host configuration and stores the
report. Read the journal and inventory before changing state. Keep the control,
compute, services and leaf containers inside the owned Incus project.

Run [the complete planning example](../examples/{cid}.py) through the macro-aware
session runner. Predict both its successful and rejected states first.

```python
{code.rstrip()}
```

The `require` macro evaluates its predicate once and retains the check under
optimized Python execution. It is a teaching aid; live evidence is graded by
the separate Rust decision kernel. Change one input, predict the observable
effect, then run again. Save both the prediction and the observation.

## Guided live build

'''+ '\n\n'.join(f'{i}. {step}' for i,step in enumerate(d['steps'],1))+f'''

Use typed Python APIs, generated intent and reviewed Ansible playbooks for these
operations. Narrow command adapters are appropriate when a vendor exposes no API;
record their exact arguments, exit status, output and version. Do not substitute
an illustrative calculation for a device or product observation.

## Every facet gets its own evidence

For each row, attach the concrete input/configuration, the operation performed,
the independently observed result and a counterexample that this operation rejects.
Related rows may share one raw trace only when it establishes each distinct facet.
Missing hardware or licensed services leaves that row blocked.

{table}

## Explain, perturb, recover

Submit a four-part trace: physical resource identity; authorized fabric path;
admitted workload and result; recovery after the controlled intervention. The
trainer independently removes one AIO, one AIN and one AII decision in separate
trials. Each removal must break the integrated acceptance condition; each restore
must recover it. Then repeat with a new topology, workload or event ordering.

Before moving on, explain why your planning example cannot establish the product
facets in the table. Collect those observations on the appropriate course station.
The original source references and discrepancy policy are in [the blueprint](../README.md).
'''
        if n == 1:
            course += '\nContinue with [the NetBox native compiler practical](C01-NETBOX.md).\n'
        outputs[f'courses/{cid}.md'] = course
        prerequisites=[f'C{i:02}' for i in range(1,21)]+([] if n==1 else [f'P{n-1:02}'])
        project=f'''# {pid} · {u['title']} in a production workflow

Prerequisites: **all C01–C20**, plus {('the complete course evidence portfolio' if n==1 else f'P{n-1:02}') }.
Level {n}/20. This project combines already taught skills; it does not introduce
an untracked prerequisite. Start with `Session.start("{pid}")` so real DeepOps
reconciliation accompanies this build.

## Deliver the system

{d['project']}

Use Incus system containers, patchbay, petgraph, NetBox and native services.
Rusternetes + KWOK provide API/state rehearsal; actual processes provide workload
results. Any unavoidable NVIDIA dependency exception must be qualified and
recorded before use. No such exception is preapproved by this project.

## Guided construction

1. Load the accepted course evidence and inventory revision. Express the system's
   inputs and output contract as immutable Python records or Rust structs.
   Include identity, units, capacity, timeout, ownership and rollback target.
2. Compose the earlier course transformations into an intent compiler. Generate
   the configuration and a before/after diff. Verify that a rejected input has
   produced no partial deployment. Review macro expansion when macros generate effects.
3. Apply the smallest canary through the native/DeepOps adapters. Establish the
   unit-specific observations below, then reconcile again and inspect the no-op result.
4. Add the next failure domain while retaining the same acceptance contract.
   Use independent network and device observations; compare them with scheduler/API state.
5. Exercise a partial failure, recovery and a second workload placement. Retain
   rejected runs. Produce the handover artifacts so another operator can reconstruct
   the exact decision from source, configuration and evidence.

{chr(10).join('- '+s for s in d['steps'])}

## Promotion and rollback flow

```mermaid
flowchart TD
  I["Immutable intent and accepted course evidence"] --> C["Compile canary plan"]
  C --> V{{"Identity, capacity and authority valid?"}}
  V -->|no| X["Return rejected plan"]
  V -->|yes| A["Apply through DeepOps and native APIs"]
  A --> M["Measure physical, fabric and workload results"]
  M --> H{{"Holdout and recovery pass?"}}
  H -->|no| R["Drain affected scope and restore supported state"]
  R --> M
  H -->|yes| P["Publish versioned deployment and evidence"]
```

## Unit-specific design rationale

{d['semantics']}

Use [{cid}'s executable example](../examples/{cid}.py) as a small local contract,
then compose it with the previous projects. A constant successful example is not
a production adapter. Repeat the underlying live observations after every
identity, firmware, topology or workload change that invalidates them.

## Reviewable deliverables

- Source and expanded/generated configuration, pinned dependencies and NetBox revision.
- DeepOps report and actual running service/device identities.
- Resource/path/admission invariants, negative isolation checks and a measured workload result.
- Canary, holdout and rollback traces with deadlines and failure ownership.
- Operator runbook, residual limitations and the complete facet evidence table from [{cid}](../courses/{cid}.md).

If a new solution requires a new skill, retain the solution and add that skill to
a course before marking the project taught. No production-readiness claim is
valid while its live qualification gates are blocked.
'''
        outputs[f'projects/{pid}.md'] = project
        exercise_entries=[]
        for j,eid in enumerate(u['exercises']):
            scenario=d['incidents'][j]
            exercise=f'''# {eid} · Independent incident

Difficulty: {2*n-1+j}/40. Prerequisites: all courses and P01–{pid}.
The instructor supplies a fresh lab, an incident packet and a bounded change scope.
The learner receives no fault recipe, expected configuration or grader oracle.

{scenario}

You have the incident timeline, read-only inventory, assigned service/API
credentials, relevant vendor documentation and the code interfaces taught in
the courses. Work through the Python controller in the Rustlings workspace.
The trainer performs owned Incus/patchbay lifecycle and real DeepOps operations;
the learner does not receive the trainer's socket or evidence store.

## Required outcome

Restore the stated service objective without exceeding the physical allocation,
breaking tenant/path constraints, accepting stale identity or losing the recovery
path. Demonstrate a real workload result at the requested execution tier.
Explain the relationship between your infrastructure, networking and operational
decisions, with a predicted and observed consequence for each.

Your submission is code plus a concise incident record containing observations,
hypotheses, changes and recovery evidence. The trainer repeats the task under
an undisclosed variation of topology, resource demand or failure ordering.
{'This first incident emphasizes one bounded fault domain.' if j==0 else 'This second incident adds an interacting fault or stale observation; a memorized repair is insufficient.'}

An incomplete domain, missing facet, unsupported capability, failed recovery or
incorrect execution tier earns no completed station. Docker-specific source
evidence comes from the earlier course assessor; the incident itself uses Incus.
KWOK status alone never establishes workload execution. The next station unlocks
only through the authoritative trainer assessment, not by editing local UI progress.
'''
            outputs[f'exercises/{eid}.md'] = exercise
            exercise_entries.append({'id':eid,'level':2*n-1+j,'skills':list(prior_skills),
                                     'prerequisites':[pid],'prompt':f'exercises/{eid}.md','facets':faceted})
        entries.append({'unit':u['id'],'course':{'id':cid,'level':n,'skills':taught,'uses':uses,'facets':faceted,'prerequisites':[previous],
                                               'skill_names':dict(zip(taught,d['skills'])),'document':f'courses/{cid}.md','example':f'examples/{cid}.py'},
                        'project':{'id':pid,'level':n,'skills':list(prior_skills),'facets':faceted,'prerequisites':prerequisites,
                                   'document':f'projects/{pid}.md'},'exercises':exercise_entries,
                        'delivery':'authored; planning example executable; live qualification pending'})
        index.append(f'| {n} | [{cid}: {u["title"]}](courses/{cid}.md) | [{pid}](projects/{pid}.md) | '+
                     ' · '.join(f'[{e}](exercises/{e}.md)' for e in u['exercises'])+' |')
    outputs['curriculum.json'] = json.dumps(entries,indent=2)+'\n'
    outputs['LEARNING_ROUTE.md'] = '\n'.join(index)+'\n'
    return outputs

def generate():
    sys.path.insert(0, str(ROOT))
    from verification.pipeline import render_repository
    return render_repository(ROOT)

if __name__=='__main__': generate()
