> Implementation authorized by the project owner on 2026-09-24. The former implementation prohibition is withdrawn. See integrations/incus for implemented behavior and validation boundaries.

# EX457: network automation on the Incus fabric

Source: [official EX457 page](https://www.redhat.com/en/services/training/ex457-red-hat-certified-specialist-in-ansible-network-automation-exam), retrieved 2026-09-23. It describes AAP 2.6; the 23 local rows below follow its public leaf order. Existing `cheatsheets/ex457/v6` is reusable content, not evidence that an Incus port already works.

## Objective ledger

| ID / topic | Class | Ansible practice and proof |
|---|---|---|
| E457-01 — Editor | C/P | In VS Code edit intent, inventory, Jinja and playbooks; run syntax/format checks and inspect a diff. Preserve the resulting commit as evidence. |
| E457-02 — Shell | C | Run inventory, documentation, syntax and diagnostic commands from the controller terminal; inspect exit codes and effective environment. Ansible captures resulting state. |
| E457-03 — SSH | C | Authenticate to both Linux-admin and FRR CLI endpoints with verified host keys. Deny the wrong key and prove no management bypass is mistaken for data connectivity. |
| E457-04 — Inventory | C | Define groups/variables for spines/leaves/clients/controller, validate exact members and resolved transport variables, then compare with the runtime ledger. |
| E457-05 — Navigator | C/P | Configure pinned Navigator, inspect effective settings and run a network play. Mark host-mode Incus execution separately from product EE acceptance. |
| E457-06 — Playbooks | C | Write/run configuration, backup, diagnosis and verification plays against the new transport; independently inspect FRR and kernel state. |
| E457-07 — Git | C | Clone, branch, diff, stage, commit, inspect history and resolve a small training conflict; replay the intended commit on a clean lab. |
| E457-08 — Editor Git | C/P | Use VS Code's Git workflow to review and commit the playbook change, then verify remote branch/commit without including credentials or generated state. |
| E457-09 — Variables | C | Generate per-device ASNs/router IDs/neighbors/prefix lists from structured intent; reject unknown/missing/duplicate endpoints. |
| E457-10 — Facts | C | Gather real FRR/device facts through the qualified connection, normalize expected fields and fail when a required fact is absent. Canned JSON is only a unit fixture. |
| E457-11 — Filters | C | Transform facts using validated filters into exact neighbor/route/config assertions; test extra idle peers, uninstalled routes and wrong interfaces from historical counterexamples. |
| E457-12 — Iteration | C | Loop across links/neighbors with readable labels and conditional capability checks; test empty lists and down links without accepting partial topology. |
| E457-13 — Failure paths | C | Fail a candidate config, collect evidence, restore the previous valid config and report the failure. Include transport loss and failed rollback cases. |
| E457-14 — Credentials | P | On a real Controller create separate machine and SCM credentials using protected injection; verify access and RBAC by successful and denied jobs. Never infer server behavior from client schemas. |
| E457-15 — Projects | P | Configure a project at a reviewed Git ref and correct content path; sync it, inspect resolved revision and reject a missing/unauthorized repository. |
| E457-16 — Workflows | P | Define explicit configure/verify/backup/diagnose nodes and success/failure/always edges. Query the exact graph and remove deliberately inserted extra edges/nodes. |
| E457-17 — Jobs | P | Launch a real job/workflow, poll terminal state and inspect every required child result plus exported artifacts. A launched job ID is not a pass. |
| E457-18 — Controller code | P | Express desired Controller objects using the qualified AAP 2.6 configuration collection, apply twice and repair deliberate drift. Verify actual server state/RBAC and execution routing. |
| E457-19 — Inclusion | C | Exercise static `import_*` and dynamic `include_*` behavior with variables, conditionals and tags; prove expected tasks run in each case. |
| E457-20 — Roles | C | Package network intent/configuration/backup/verification into reusable roles with documented contracts, handlers and platform adapters. Test two node roles. |
| E457-21 — Jinja | C | Render complete candidate FRR configuration with strict schema/input checks; validate before loading and compare active plus saved configuration. |
| E457-22 — netcommon | C/P | Use qualified `ansible.netcommon` CLI/parse/transport facilities through the actual FRR adapter. Verify prompt/error handling and supported capabilities; platform-independent interfaces do not erase NOS differences. |
| E457-23 — Backups | C | Save running/startup config with device identity, run ID and hashes to protected storage. Restore after deliberate drift, verify topology traffic and confirm persistence after restart. |

## Migrate the full existing pack

Port `labctl.py` prepare/build/preflight/deploy/restart/dataplane/saved/destroy actions to the single Incus lifecycle boundary. Replace `lab/frr/Dockerfile` and entrypoint scripts with the qualified CachyOS FRR systemd image/roles. Replace `datacenter.clab.yml.j2` with common plan compilation. Preserve private per-run host identities, exact inventory and state directory modes.

Keep the network facts/filter predicates, configuration removal semantics, backup integrity, HTTPS trust, Controller graph checks, pyATS probes and Hypothesis historical cases. Update `contracts.py`, `state.py`, `backup_store.py`, module-argument checks, documentation/evidence validators and packaging together with changed lifecycle contracts. v6's FRR collection and libssh compatibility assumptions require live requalification against the selected CachyOS FRR build.

Replace `docker exec` observations with trusted Incus exec or SSH as appropriate. Data-plane verification must use source-bound traffic on fabric interfaces, exact BGP peer sets, route next hops, installed FIBs and edge counters. Do not let management or petgraph static routes make a broken BGP configuration pass.

## Network drill sequence

1. Build two spines, two leaves and two client system containers, with one bridge per wire and the separate management plane. Verify endpoint mapping/MTU and lack of an IX uplink before configuring routing.
2. Apply addressing/router IDs/ASNs/neighbors and allow only intended route advertisements. Save the integrated FRR configuration using the validated persistence method.
3. Observe exact BGP session set, local/remote AS, received/advertised prefixes, selected next hops and kernel route installation. Reject extra idle neighbors and unrelated installed routes as false positives.
4. Inject one fault at a time: wrong ASN, missing neighbor, wrong address, denied TCP/179, stale prefix list, shutdown interface, wrong MTU, absent return route or invalid saved file. Capture observations before repair and repair the demonstrated cause.
5. Test redundant path loss, complete partition and restoration; measure convergence as observed wall time, without promising hardware timing fidelity. Verify policy and data traffic after convergence.
6. Restart FRR, then the Incus node, then rebuild a clean node from intent. Reattach recreated NICs and verify host trust according to lifecycle policy. Each test establishes a different persistence/reproducibility boundary.
7. Restore a backup only to the matching intended device or an explicitly mapped replacement. Verify content identity, ownership and active behavior. Corrupt a backup and require refusal.

Additional VXLAN/EVPN/VLAN/VRF/IPv6 exercises can enrich the datacenter lab, but require explicit supported models as described in [05](05-patchbay-petgraph-networking.md). They must not replace any of the 23 rows. FRR is not a simulator of every Cisco/Juniper/Arista command or ASIC feature; vendor transfer remains separately qualified.

## Actual Controller practice on a CachyOS client

The repository's shipped `awx.awx 24.6.1` harness demonstrates pinned client contracts. Preserve that label. For AAP 2.6, verify the current official recommendation and use `infra.aap_configuration` from the proper source with real documented role variables; do not infer them from AWX modules. Configure endpoint URL/API prefix, TLS trust, least-privilege token, SCM credential, machine credential, inventory, project revision, execution environment and workflow dependency ordering.

A CachyOS client can drive the API, but this guide does not claim Red Hat supports hosting AAP on CachyOS. An already available qualified endpoint can provide the product exercises. If none is available under the OS constraint, keep E457-14–18 blocked while preparing reviewable object definitions and client validation. AWX on an Incus-hosted Kubernetes lab is an explicitly adapted alternative, not an AAP acceptance result.

Prove Controller execution nodes can actually reach the management addresses and trust the FRR hosts; controller-local success is insufficient. Keep secrets out of Git and enable intended credential injection without hard-coded temporary filenames. Inspect workflow RBAC, denied access, source update failures and artifact retention. Preserve the existing exact graph and backup checks.

Sources: existing `cheatsheets/ex457/v6/{11_controller,12_controller_iac,17_version_compatibility}.md`, [AAP 2.6 configuration guidance](https://docs.redhat.com/en/documentation/red_hat_ansible_automation_platform/2.6/secure-ref_initial_configuration), and [ansible.netcommon](https://docs.ansible.com/projects/ansible/latest/collections/ansible/netcommon/). Revalidate product documentation/collections at implementation time.
