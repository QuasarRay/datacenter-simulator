# 06 — Roles, collections and execution environments

## Reuse

The [fabric role](roles/fabric/tasks/main.yml) renders the candidate, calculates explicit removals and applies intent. Defaults live in `roles/fabric/defaults/main.yml`. `configure_fabric.yml` imports it; `restore_fabric.yml` includes it. Imports expand statically; includes select tasks dynamically and can be looped. Network verification is a supplied task file imported or included by full playbooks.

Exercise: create a small reporting handler in a practice role and notify it only after a change; run twice and show when it executes. Keep persistence as an explicit post-verification phase: a generic change handler can save bad state too early.

```bash
ansible-galaxy collection list
ansible-doc ansible.netcommon.cli_config
ansible-playbook --list-tasks playbooks/configure_fabric.yml
```

The collection versions are explicit in `requirements.yml`. FRR and AWX are community collections, not a Red Hat product support guarantee.

[Reusing Ansible artifacts](https://docs.ansible.com/projects/ansible-core/2.19/playbook_guide/playbooks_reuse.html). [Ansible playbooks](https://docs.ansible.com/projects/ansible-core/2.19/playbook_guide/playbooks_intro.html). [FRR facts](https://docs.ansible.com/projects/ansible/9/collections/frr/frr/frr_facts_module.html).

## Build before running

```bash
podman login registry.redhat.io
cd ee
ansible-builder create -f execution-environment.yml
ansible-builder build -f execution-environment.yml -t localhost/ex457-ee:v5 --container-runtime podman
cd ..
podman image inspect localhost/ex457-ee:v5
ssh-add "$ANSIBLE_PRIVATE_KEY_FILE"
env -u ANSIBLE_PRIVATE_KEY_FILE ansible-navigator run ee_smoke.yml --mode stdout \
  --ee true --eei localhost/ex457-ee:v5 --pp never --co=--network=host
```

An entitled Red Hat base-image login must precede the build. `ee-minimal-rhel9:latest` is a discovery default; replace with the inspected digest for a reproducible accepted build and record core/collection/libssh versions. Start an SSH agent first if your desktop session does not provide one; `ssh-add` unlocks the client identity for Navigator's agent integration. The host-only absolute private-key environment setting is removed for this EE command. Local host-mode validation uses community core 2.19.3; the EE's bundled core is a separate compatibility gate. The root-level `ee_smoke.yml` entry point keeps the whole pack adjacent to the playbook and mounted in the EE, including inventory, config and `.state/known_hosts`; use ssh-agent integration for the client key and explicitly verify the trust path in that context.

After the local smoke test, tag, log in to your destination registry, push, record RepoDigest, and set Controller's `ee_image` to that published digest. Prove an execution node can pull it.

[Builder definition](https://docs.ansible.com/projects/builder/en/stable/definition/). [Navigator settings](https://docs.ansible.com/projects/navigator/settings/). [AAP 2.6 execution best practices](https://docs.redhat.com/en/documentation/red_hat_ansible_automation_platform/2.6/develop-assembly_controller_best_practices).


[Navigator SSH-agent and project-mount FAQ](https://docs.ansible.com/projects/navigator/faq/).
