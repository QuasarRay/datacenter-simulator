# 16 — Fast troubleshooting

## Work from the first failing layer

| Symptom | Inspect |
|---|---|
| Inventory empty | Working directory, `-i`, YAML group nesting |
| Local playbook missing | Use pack root; all referenced files are supplied |
| SSH denied with a key | Client key, account lock, sshd policy, frrvty membership |
| Manual SSH works; network_cli fails | Patched libssh backend, ansible_libssh_known_hosts and trust inside the actual EE |
| Unknown module | Collection version/path and EE contents |
| Facts fail | Legacy frr.frr compatibility; supported gather subsets |
| Peer count right but verifier fails | Full peer identities, states and ASNs |
| BGP route present but traffic fails | Zebra selected/installed/FIB flags, kernel route, source address |
| Restart loses routes | Lifecycle link recreation, `vtysh -b`, saved config |
| Controller Project succeeds but job fails | Playbook path, EE pull, credentials, routes from execution node |
| Workflow green after error | Individual node job status, not only workflow status |

```bash
ansible-config dump --only-changed
ansible-galaxy collection list
ansible-doc -t connection ansible.netcommon.libssh
ansible-playbook playbooks/diagnose_fabric.yml --limit spine1
```

Use verbose logs locally to diagnose, then redact any secrets before committing transcripts.

[Ansible inventory](https://docs.ansible.com/projects/ansible-core/2.19/inventory_guide/intro_inventory.html). [network_cli connection](https://docs.ansible.com/projects/ansible/latest/collections/ansible/netcommon/network_cli_connection.html). [libssh connection](https://docs.ansible.com/projects/ansible/latest/collections/ansible/netcommon/libssh_connection.html). [AWX workflows](https://docs.ansible.com/projects/awx/en/24.6.1/userguide/workflow_templates.html).

