# 15 — Targeted drills

## Failure practice

| Drill | Expected failure | Recovery |
|---|---|---|
| Supply wrong public host key | libssh rejects server | Restore verified public key |
| Remove remote-node derivation | Undefined variable before route loop | Import model tasks |
| Extra Idle peer | Exact peer-set check | Reconcile stale neighbor |
| Swap interface addresses | Interface map equality | Reconcile from model |
| Remove /32 network advertisement | Exact config/route gate | Reapply candidate |
| Drop every probe | Numeric loss/return-code gate | Restore forwarding/filter |
| Remove saved address then restart | Saved/runtime mismatch | Recover verified backup |
| Remove EE trust credential | Trust preflight failure | Attach verified credential |
| Break backup service | Backup failure blocks Change | Restore service and rerun |
| Handled workflow failure | Core-node acceptance fails | Repair underlying failure |

Run the unit/dataflow checks first. Then execute controlled live faults, recording an expected failure and a restored passing state for each. Never edit a predicate just to accept bad state.

[network_cli connection](https://docs.ansible.com/projects/ansible/latest/collections/ansible/netcommon/network_cli_connection.html). [Ansible blocks](https://docs.ansible.com/projects/ansible/latest/playbook_guide/playbooks_blocks.html). [AWX workflows](https://docs.ansible.com/projects/awx/en/24.6.1/userguide/workflow_templates.html).

