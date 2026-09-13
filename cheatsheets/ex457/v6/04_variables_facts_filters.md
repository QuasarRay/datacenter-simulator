# 04 — Variables, facts and filters

## Data model

`model-upstream.yml` is a byte-for-byte copy of the pinned repository model. `fabric_intent` contains interface → address lists, peer IP → ASN, local ASN/router ID, advertised networks and all other router loopbacks. Derivation runs before verifier loops. All addresses are normalized with Python `ipaddress`; invalid or duplicate endpoint data fails closed.

```yaml
- name: Derive per-node state
  ansible.builtin.set_fact:
    fabric_intent: "{{ dc | fabric_intent(inventory_hostname) }}"
- name: Sorted peer addresses
  ansible.builtin.debug:
    msg: "{{ fabric_intent.peers | dict2items | map(attribute='key') | sort }}"
```

Practice `default`, `combine`, `dict2items`, `selectattr`, `map`, `from_json` and `to_nice_yaml`. Registered loop results live in `.results`; ordinary `set_fact` produces host variables. `set_stats` is not a general replacement for host-variable propagation.

[Ansible variables](https://docs.ansible.com/projects/ansible-core/2.19/playbook_guide/playbooks_variables.html). [Ansible filters](https://docs.ansible.com/projects/ansible/latest/playbook_guide/playbooks_filters.html). [Pinned simulator model](https://github.com/QuasarRay/datacenter-simulator/blob/2003f8bdd88423c77c528785f7ffa70f398a23e0/vars/datacenter.yml).

## Device facts

```bash
ansible-playbook playbooks/facts_filters.yml
ansible-doc frr.frr.frr_facts
```

This uses `gather_subset: [config]`, plus FRR's default facts. The legacy module accepts `default`, `hardware`, `config`, `interfaces`, `all` and exclusions; `min` is not a valid subset in 2.0.2. Its published testing is FRR 6.0, so FRR 10.7 compatibility must be demonstrated. Facts are observations; the Git model remains intent. Acceptance: display `ansible_net_version` and `ansible_net_config`, then transform a model dictionary into a sorted list.

[FRR facts](https://docs.ansible.com/projects/ansible/9/collections/frr/frr/frr_facts_module.html).

