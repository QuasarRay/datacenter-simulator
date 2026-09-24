> Implementation authorized by the project owner on 2026-09-24. The former implementation prohibition is withdrawn. See integrations/incus for implemented behavior and validation boundaries.

# Ansible orchestration and troubleshooting contracts

## Proposed role/play boundaries

The names below describe future content; this PR does not create these roles or playbooks.

| Proposed responsibility | Where it executes | Result |
|---|---|---|
| `host_preflight`, `incus_host`, `image_qualify` | Selected CachyOS lab host | Qualified daemon, image, storage and network prerequisites |
| `lab_plan`, `lab_apply`, `lab_destroy` | Rust supervisor through the lab host | One authoritative lifecycle transaction |
| `guest_bootstrap` | Incus exec/file connection | Python, temporary access, identity and SSH readiness |
| `guest_baseline`, `frr`, `netbox`, `slurm`, `kubernetes_node` | SSH to exact published inventory | Persistent service configuration |
| `diagnose`, `repair`, `verify`, `collect_evidence` | Guest SSH, Incus recovery or dedicated host | Observations, bounded repair and independent proof |
| `fault_inject`, `reset` | Instructor-only inventory | Reproducible fault seed and restoration metadata |

Use distinct inventory groups for hosts, Linux guests, network CLI devices, service nodes, CPU mock nodes, physical GPU nodes, and recovery targets. Generate live inventory only from verified ready objects. Preserve static inventory exercises separately. Every host entry records stable node ID, Incus project/name, management address, SSH identity source and capability class; redact credentials.

## Bootstrap and normal transport

`community.general.incus` is a **connection plugin**, not a lifecycle module. It executes in an existing instance using the Incus CLI. Pin `community.general` and verify plugin options with `ansible-doc -t connection community.general.incus`. The controller executing this plugin needs the CLI and authorized access to the correct daemon; it does not magically run on a remote lab host.

Illustrative recovery inventory for an existing authorized instance:

```yaml
all:
  hosts:
    leaf1_recovery:
      ansible_connection: community.general.incus
      ansible_incus_remote: local
      ansible_incus_project: dc-lab
      ansible_incus_host: leaf1
      ansible_user: root
      ansible_python_interpreter: /usr/bin/python
```

Use this channel to bootstrap Python/SSH or repair broken SSH. `raw` can bootstrap a system without Python, but must use a fixed validated command and report changes correctly. For normal Linux exercises switch to SSH with a non-root administrator and explicit `become`. The Incus root execution channel bypasses guest SSH/PAM and must not grade their functionality.

For FRR network exercises retain the authenticated vtysh SSH adapter and `ansible.netcommon.network_cli` with a validated network OS plugin. Keep the Linux-admin inventory separate from the FRR CLI inventory even when both refer to the same node. Never run ordinary Python modules through a vtysh-only connection.

## Repeatable controller environment

Use a pinned Python virtual environment and collection installation paths on CachyOS, outside system Python. Record core, Navigator, Builder, collection, SSH/libssh, FRR and Incus versions. Retain v6's compatibility regressions and transport patches until a replacement is demonstrated; do not copy an old patch blindly to a different collection.

For the Incus-only baseline, configure Navigator host execution (`execution-environment.enabled: false`) and run it inside the qualified controller system container or on the authorized CachyOS controller. This is an adaptation, not an OCI EE. [12](12-ex294-ansible-administration.md) covers the development-container/product distinction.

## Diagnosis is a first-class play

Collect evidence **before** remediation. Use `gather_facts: false` when broken Python, DNS or SSH would prevent startup; select the recovery connection deliberately. Read-only commands use `changed_when: false`, bounded execution, explicit return-code classification and sanitized stdout/stderr. A failure to collect evidence is recorded, not ignored as success.

Illustrative diagnostic task for a reachable systemd guest:

```yaml
- name: Capture failed units before attempting repair
  ansible.builtin.command:
    argv: [systemctl, --failed, --no-legend, --no-pager]
  register: failed_units
  changed_when: false
  failed_when: failed_units.rc != 0
```

This command's successful exit does not prove there are no failed units. A later assertion must parse the returned rows against the exercise's expected state. `systemctl is-active` has different exit semantics; define them per command rather than globally accepting every nonzero result.

A future fault record should include lab/run IDs, objective ID, affected object, fault seed, expected symptom, pre-fault hashes, diagnostic commands, evidence timestamps, proposed repair, rollback and final assertions. Keep the instructor answer key separate from student inventory so the exercise tests diagnosis rather than a supplied cause label.

## Repair safely and prove the result

1. Select the smallest confirmed cause from evidence: wrong file, missing package, denied policy, exhausted resource or failed dependency. Do not solve every scenario with a restart/reinstall.
2. Validate candidate configuration before replacement (`template`/`copy` validation where supported, plus service-specific checks). Back up state necessary for rollback. Restrict destructive storage work to identified disposable devices.
3. Use declarative modules where they model the state: `user`, `group`, `file`, `template`, `systemd_service`, `cron`, `community.general.pacman`, `ansible.posix.mount`, and appropriate firewalld modules. Check actual collection argument specs. Use `command` for diagnostics and missing module coverage with explicit change detection; use `shell` only when shell semantics are the learning objective.
4. Notify handlers on changed configuration. With `serial: 1`, repair one node, flush needed handlers and run health checks before proceeding. `block`/`rescue` must restore or preserve evidence and then fail when the repair was unsuccessful.
5. Repeat the original failed probe independently. Check live state and persisted configuration, reboot/restart the correct boundary, then rerun configuration to demonstrate convergence. A green Ansible task count alone cannot prove recovery.

Check mode is a planning aid, not proof that an external command, storage repair or network operation works. Do not use `ignore_errors` or `failed_when: false` without a subsequent classification/assertion. Unreachable hosts need a separate recovery play; ordinary task rescue does not automatically repair a dead connection.

## Security and evidence

Use Vault or secret injection, scoped credentials and authenticated host-key distribution. Fetch public host keys over the trusted Incus bootstrap channel; a bare `ssh-keyscan` on a potentially spoofed network is discovery, not trust. Limit `no_log` to sensitive tasks and publish useful redacted diagnostics. Never export `.state` wholesale or log controller tokens, private keys, LUKS passphrases, kubeconfigs or packet payloads without a deliberate redaction policy.

Store evidence per run/objective, with source commit, topology/image hashes, versions, command return codes and timestamps. Use distinct `PASS`, `FAIL`, `BLOCKED`, `NOT_RUN`, and `ADAPTED` fields. Details of transport-loss and host-level recovery are in [15](15-ansible-recovery-runbooks.md).

Sources: [Incus connection](https://docs.ansible.com/projects/ansible/latest/collections/community/general/incus_connection.html), [Navigator settings](https://docs.ansible.com/projects/navigator/settings/), [Ansible error handling](https://docs.ansible.com/projects/ansible/latest/playbook_guide/playbooks_error_handling.html).
