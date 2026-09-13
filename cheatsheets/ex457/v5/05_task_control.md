# 05 — Loops, conditions and failures

## Control flow

```bash
ansible-playbook -i inventory/localhost.yml playbooks/task_control.yml
```

Read the named `loop_var`, `when`, deliberate `/bin/false`, `rescue` and `always`. Acceptance: only the spine is selected, recovery runs, cleanup runs and the final assertion passes. A rescued failure can yield a successful play; explicitly assert the outcome you need. Unreachable hosts and invalid task definitions do not behave like ordinary module failures.

The real verifier uses a bounded `until` loop for BGP convergence. `changed_when: false` marks observations as unchanged. Never use `ignore_errors` to turn a failed verification gate green. In production recovery, end `rescue` with an explicit failure if the original operation must remain failed.

[Ansible loops](https://docs.ansible.com/projects/ansible-core/2.19/playbook_guide/playbooks_loops.html). [Ansible blocks](https://docs.ansible.com/projects/ansible/latest/playbook_guide/playbooks_blocks.html).

## Negative outcomes

Run `python -m unittest discover -s tests -v`. The peer test uses the unfiltered peer dictionary so an additional Idle peer fails just like an additional Established peer. Packet loss is parsed numerically with exactly three sent/received probes. No substring predicate is used.

[Ansible filters](https://docs.ansible.com/projects/ansible/latest/playbook_guide/playbooks_filters.html).

