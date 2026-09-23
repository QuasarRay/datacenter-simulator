> **AGENT INSTRUCTION: DO NOT IMPLEMENT THIS DOCUMENT.** This is a documentation-only migration proposal. Do not change code, configuration, dependencies, infrastructure, or runtime state from these instructions unless the user separately requests implementation. Documentation review and correction are permitted.

# EX294: Ansible administration and development workflow

Source: [official EX294 page](https://www.redhat.com/en/services/training/ex294-red-hat-certified-engineer-rhce-exam-red-hat-enterprise-linux), checked 2026-09-23. The 57 rows below follow its public ordering. The IDs are local. Apply [10](10-exam-coverage-contract.md), including its development-container and product limitations.

## Inherited administration: E294-01–08

These are required practice, not assumed prerequisites that can be omitted from grading. Use the complete corresponding EX200 procedures and build their state on **fresh** CachyOS targets through Ansible.

| ID / topic | Class | Exercise and evidence |
|---|---|---|
| E294-01 — Tools | C | Execute E200-01–11 as prepared fixtures and Ansible tasks; retain shell/SSH/file/permission negative cases. |
| E294-02 — Runtime | C/H | Execute E200-20–29, distinguishing service restart, container restart and actual host reboot. |
| E294-03 — Volumes | H | Execute E200-30–35 against owned scratch devices; reproduce a clean storage layout without damaging existing data. |
| E294-04 — Filesystems | C/H | Execute E200-36–40; verify filesystem-specific behavior and permission diagnosis. |
| E294-05 — Maintenance | C/H | Execute E200-41–46, plus the network persistence needed by the inventory; check current and next-boot state. |
| E294-06 — Accounts | C | Execute E200-51–54; verify both positive access and revocation after a new login session. |
| E294-07 — Protection | C/H | Execute E200-55–62, retaining real SELinux host requirements and explicit adaptation scope. |
| E294-08 — Script reading | C | Review a supplied shell script with a quoting/exit-status defect; predict its effects, run bounded fixtures, explain the discrepancy and fix the template. Do not skip reasoning by substituting an unrelated role. |

## Core mechanics: E294-09–20

| ID / topic | Class | Exercise and evidence |
|---|---|---|
| E294-09 — Inventory | C | Write static YAML inventory for controller/router/service groups and inspect `ansible-inventory --graph/--list`. Assert exact membership and exclusions. |
| E294-10 — Modules | C | Choose stateful modules for packages/files/services and compare with a diagnostic command. Inspect module documentation and verify actual state, not task names. |
| E294-11 — Variables | C | Use group/host/play/role/extra variables with conflicting fixture values; inspect resolved nonsecret values and demonstrate intended precedence. Reject undefined required inputs. |
| E294-12 — Facts | C | Collect minimal facts and custom facts, distinguish registered results from facts, and test stale cache invalidation after an image rebuild. |
| E294-13 — Loops | C | Configure a list of users/services with explicit loop variables and labels; inspect per-item results and failure handling. Test empty and malformed lists. |
| E294-14 — Conditions | C | Branch on supported OS/capability facts and registered status; test true/false/undefined cases. Never falsify distribution facts to run an unsupported role. |
| E294-15 — Plays | C | Separate controller-local planning, guest configuration and verification into plays with deliberate host patterns and privilege boundaries. Show which host ran each task. |
| E294-16 — Failure | C | Use `block`/`rescue`/`always` to preserve a failed validation, roll back only owned changes and report failure. Test unreachable-host handling separately. |
| E294-17 — Playbooks | C | Compose a baseline then role-specific playbook; run syntax checking, real application, independent probes and a second convergence run. |
| E294-18 — Configuration | C | Demonstrate effective Ansible configuration using `ansible-config dump --only-changed`; test project path, environment and command-line interactions. |
| E294-19 — Roles | C | Factor a repeated service into defaults/tasks/handlers/templates/vars, exposing a small documented variable contract and correct handler notification. |
| E294-20 — Local reference | C | Find an unfamiliar module/option with the installed offline `ansible-doc`; validate the selected FQCN and argument spec against the pinned collection. |

## Controller and targets: E294-21–28

| ID / topic | Class | Exercise and evidence |
|---|---|---|
| E294-21 — ansible.cfg | C | Configure inventory, collection path, SSH trust, interpreter and output handling. Verify the loaded file and fail a test using the wrong working directory. |
| E294-22 — Navigator config | C/P | Write project Navigator settings, inspect effective settings and execute a simple play. Baseline host mode runs inside the Incus controller; separately qualify EE behavior. |
| E294-23 — Host entries | C | Create a static host file with exact SSH addresses/ports/users. Use `--list-hosts` before execution and show an unknown host is rejected. |
| E294-24 — Groups | C | Add children/group_vars/host_vars and exercise union/intersection/exclusion patterns. Assert no recovery host accidentally enters a destructive guest play. |
| E294-25 — Node preparation | C | Bootstrap Python, a managed account, sudo and SSH over trusted Incus recovery; switch to normal SSH and verify identity/facts. |
| E294-26 — Key distribution | C | Generate a protected controller key, install its public half and verified host trust, rotate it, and reject the old key. |
| E294-27 — Escalation | C | Deploy validated sudo policy and test successful intended privilege plus denial of unrelated commands; protect escalation credentials. |
| E294-28 — File delivery | C | Deploy templates/static files with explicit modes/owners, validate before replacement, then fetch hashes and test idempotence. |

## Execution, Git and editor: E294-29–36

| ID / topic | Class | Exercise and evidence |
|---|---|---|
| E294-29 — Runners | C/P | Run the same play with `ansible-playbook` and Navigator; compare target state and artifacts. Distinguish host execution from a real OCI execution environment. |
| E294-30 — Discovery | C/P | Use Navigator's installed collection/module inspection, select an unfamiliar module and execute a small valid example. Capture its FQCN/version. |
| E294-31 — Environment | C/P | Inspect inventory and environment via Navigator, change one explicit setting and demonstrate the expected target/configuration change. |
| E294-32 — Clone | C | Clone a designated training Git repository at a known commit, verify remote/branch and reproduce a play from a clean checkout. |
| E294-33 — Stage | C | Create a practice change, inspect diff, stage only intended files and inspect the staged result. Keep secrets/evidence caches excluded. |
| E294-34 — Editor delivery | C/P | In VS Code create and validate a play, commit/push to a designated training branch and compare the remote commit. Ansible can validate the resulting artifact but cannot substitute for editor use. |
| E294-35 — Editor settings | C/P | Configure Navigator from the VS Code workspace and run it with the intended settings file. Verify which environment and inventory the editor actually invoked. |
| E294-36 — Dev container | P | Use a real qualified Ansible development-container workflow backed by Docker-free OCI tooling if permitted; run the play from that environment and record its image digest. Incus Remote SSH practice is separately marked adapted, not exact completion. |

## Authoring and reuse: E294-37–45

| ID / topic | Class | Exercise and evidence |
|---|---|---|
| E294-37 — Common modules | C | Build a small service from package, user, file, template and systemd modules with a handler; observe package/service state and client response. |
| E294-38 — Results | C | Register command/module results, inspect their actual structure and use validated fields in later tasks. Include a missing-key/failure fixture. |
| E294-39 — Flow | C | Apply `when` conditions to single tasks and included content; demonstrate that an unsupported node is refused rather than partially configured. |
| E294-40 — Recovery | C | Intentionally fail candidate config validation, retain the previous working service and run diagnostic tasks; the failed change must still fail the play. |
| E294-41 — Desired state | C | Converge two differently drifted nodes to the same policy, then delete/rebuild one from the qualified image and rerun. Assert persisted behavior and clean second runs. |
| E294-42 — Role creation | C | Author a reusable role with defaults, handlers and argument validation; call it twice with different service instances without variable collisions. |
| E294-43 — Role install | C | Install a pinned role from a reviewed source into the configured path, record its identity and invoke it with its documented variables. |
| E294-44 — Collections | C | Install pinned collections from a requirements file into an isolated path, use FQCNs and prove no system-wide version silently overrides them. |
| E294-45 — Content bundle | C | Use a collection's related roles/plugins/modules together, including documented filters/lookups when needed. Record collection dependencies and reproduce offline installation. |

## System modules and content: E294-46–57

| ID / topic | Class | Exercise and evidence |
|---|---|---|
| E294-46 — Software | C | Implement the CachyOS E200-12–15/45 software exercises with qualified module contracts; verify signatures/source/version. |
| E294-47 — Services | C | Implement E200-28/42/49; demonstrate handler ordering, restart persistence and remote protocol health. |
| E294-48 — Firewall | C/H | Implement E200-50/55 and test allowed/denied traffic plus permanent state. |
| E294-49 — Filesystems | C/H | Implement E200-34/36–40 with appropriate filesystem/mount tools and no generic destructive formatting. |
| E294-50 — Devices | H | Implement E200-30–35/39 with exact scratch-device assertions and pre/post data checks. |
| E294-51 — File content | C | Implement E200-07/08 using structured templates for complete ownership and surgical edits where ownership is partial. |
| E294-52 — Archiving | C | Implement E200-06; verify restore, metadata and safe paths instead of merely archive existence. |
| E294-53 — Schedules | C | Implement all at/cron/timer cases in E200-41, including successful actual execution and removal. |
| E294-54 — Security | C/H | Implement E200-54–62 with genuine negative access checks and SELinux qualification. |
| E294-55 — Identities | C | Implement E200-51–53 and demonstrate account/group changes on clean and drifted hosts. |
| E294-56 — Templates | C | Render node-specific FRR/service config from structured intent using strict input validation; inspect diffs, reject invalid candidates and verify runtime behavior. |
| E294-57 — Vault | C | Encrypt variables/files, run with the intended Vault ID, demonstrate wrong-secret failure, rotate encrypted content and ensure logs/artifacts contain no plaintext secret. |

## Complete controller workflow

Prepare the controller with a pinned Python virtual environment and explicit collection directory. Inside the future training project, a normal sequence is inventory inspection → syntax validation → candidate diff/check mode where meaningful → real application → independent verification → repeat on a clean instance. These example tool shapes use existing Ansible commands; the files shown are proposed exercise files:

```bash
ansible-inventory -i inventory.yml --graph
ansible-playbook -i inventory.yml site.yml --syntax-check
ansible-playbook -i inventory.yml site.yml --check --diff
ansible-playbook -i inventory.yml site.yml
ansible-playbook -i inventory.yml verify.yml
```

`--diff` must not expose secrets. Check mode cannot validate every command or repair. A fresh-instance run is required to find dependencies on state accidentally created by earlier attempts.

Use native VS Code on CachyOS with Remote SSH into the controller for the baseline. For exact development-container behavior, qualify the current extension's supported Docker-free engine and an OCI image built from CachyOS where the exercise permits; do not set `container-engine: incus`, which Navigator does not advertise. Product-required images/entitlements remain external constraints, and a strict Incus-only policy leaves that exact product row blocked. Ansible still provisions and verifies all surrounding workflow state.

Sources: [Navigator configuration](https://docs.ansible.com/projects/navigator/settings/), [Ansible VS Code extension](https://ansible.readthedocs.io/projects/vscode-ansible/), [Ansible playbook guide](https://docs.ansible.com/projects/ansible/latest/playbook_guide/index.html).
