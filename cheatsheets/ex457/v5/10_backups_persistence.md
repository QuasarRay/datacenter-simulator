# 10 — Backups, restore and persistence

## Backup and consume it

```bash
ansible-playbook playbooks/backup_fabric.yml -e run_id=before-drift
# Perform the reversible loopback drift in chapter 09.
ansible-playbook playbooks/restore_fabric.yml -e restore_run_id=before-drift
python tools/labctl.py dataplane
```

`cli_backup` writes `.state/backups/before-drift/<host>.cfg`. The restore play actually reads those bytes, parses the managed configuration, checks that it matches the current model, and reconciles devices from the parsed backup state. It rejects a backup from another intent revision. This is managed-state recovery, not a claim to restore arbitrary FRR features or a full device image.

An explicit run ID is mandatory outside Controller. In Controller, `awx_job_id` supplies one stable ID across all hosts. No per-host timestamps or `set_stats` propagation are involved.

[cli_backup module](https://docs.ansible.com/projects/ansible/latest/collections/ansible/netcommon/cli_backup_module.html). [Ansible variables](https://docs.ansible.com/projects/ansible-core/2.19/playbook_guide/playbooks_variables.html).

## Durable Controller export

The backup play requires an HTTPS `backup_url` and `EX457_BACKUP_TOKEN` for Controller jobs. It PUTs each file to `<base>/<job-id>/<host>.cfg`, GETs it back and compares SHA-256. Provide a service supporting this PUT/GET object contract and the issuing token; storage provisioning is an operator prerequisite. A failed export fails the backup job and blocks the workflow's Change node. Never count an ephemeral `/runner` file as a durable backup.

To recover on the lab host from the service, download the selected job's four files into `.state/backups/<job-id>/` using your service's authenticated client, verify their recorded hashes, then use `restore_run_id=<job-id>`. Keep credentials outside Git.

[cli_backup module](https://docs.ansible.com/projects/ansible/latest/collections/ansible/netcommon/cli_backup_module.html). [AWX custom credentials](https://docs.ansible.com/projects/awx/en/24.6.1/userguide/credential_types.html).

## Save and restart

```bash
ansible-playbook playbooks/verify_fabric.yml
python tools/labctl.py dataplane
ansible-playbook playbooks/persist_fabric.yml
python tools/labctl.py restart
ansible-playbook playbooks/verify_fabric.yml
python tools/labctl.py dataplane
```

Persistence writes integrated config, reads startup config and compares managed state. Containerlab's lifecycle restart restores data links. After it returns, the supplied `labctl.py restart` wrapper explicitly runs `ex457-load` in every router and records `restart-reload.json`. That helper waits for Zebra/BGP readiness and invokes `vtysh -b`; topology `exec` invokes the same helper on initial deploy. This explicit reload is a pack adaptation. Raw `docker restart` is outside this contract. Prove reload by configuring, saving, restarting, then verifying **without a configure play in between**.

Destroy/redeploy removes container state; rebuild that lifecycle from Git with a configure play. It is distinct from saved-config restart. Retain host identities until intentional key rotation; delete private keys/backups only after destroying the lab and exporting any records you need.

[FRR integrated configuration](https://docs.frrouting.org/en/stable-10.7/vtysh.html). [Containerlab restart command](https://containerlab.dev/cmd/restart/). [Containerlab Linux kind](https://containerlab.dev/manual/kinds/linux/).
