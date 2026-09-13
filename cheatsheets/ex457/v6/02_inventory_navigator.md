# 02 — Inventory and Navigator

## Inventory

```bash
ansible-inventory -i inventory/network.yml --graph
ansible-inventory -i inventory/network.yml --host spine1
ansible-inventory -i inventory/network.yml --list
ansible-doc -t connection ansible.netcommon.network_cli
```

Expected graph: `fabric` contains `spines` and `leafs`, two routers each. `group_vars/fabric.yml` defines connection behavior; host entries define management addresses. `playbooks/tasks/model.yml` derives addresses and peers from the source of truth. Exercise: inspect the effective SSH backend and change a host variable on a practice branch; explain which setting wins.

[Ansible inventory](https://docs.ansible.com/projects/ansible-core/2.19/inventory_guide/intro_inventory.html). [Ansible variables](https://docs.ansible.com/projects/ansible-core/2.19/playbook_guide/playbooks_variables.html).

## Host mode first

```bash
ansible-navigator settings --mode stdout
ansible-navigator inventory -i inventory/network.yml --list --mode stdout --ee false
ansible-navigator run playbooks/show_version.yml --mode stdout --ee false
```

The shipped Navigator settings disable EEs so this chapter does not depend on chapter 06. Review effective settings in the interactive `ansible-navigator settings` view as needed. Acceptance: all four routers return FRR version output with strict libssh host trust. Switch to `--ee true` only after building the image in chapter 06.

[Navigator settings](https://docs.ansible.com/projects/navigator/settings/).

