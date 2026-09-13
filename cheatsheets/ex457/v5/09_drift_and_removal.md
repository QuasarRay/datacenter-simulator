# 09 — Drift and removal

## Inject and repair

After a clean backup, inject one fault at a time on the disposable lab:

```bash
sudo docker exec clab-ex457-v5-spine1 vtysh   -c 'configure terminal' -c 'interface lo' -c 'ip address 192.0.2.99/32'
ansible-playbook playbooks/verify_fabric.yml --limit spine1
# Expect a failure, then repair from intent:
ansible-playbook playbooks/configure_fabric.yml
python tools/labctl.py dataplane
```

Repeat with `router bgp 65001` and `neighbor 192.0.2.98 remote-as 65200`: an Idle extra peer must fail verification. Change a valid peer's ASN, remove a loopback advertisement, and move an address to another interface in separate attempts. Keep transcripts of expected failures and successful repair. The role issues explicit `no` commands and then reapplies the candidate.

[cli_config module](https://docs.ansible.com/projects/ansible/latest/collections/ansible/netcommon/cli_config_module.html). [FRR BGP](https://docs.frrouting.org/en/stable-10.7/bgp.html).

## Acceptance

A repair passes only after running all state layers in chapter 08. The file-level and dataflow tests exercise stale-state calculation; actual FRR command acceptance and second-run convergence remain runtime evidence. Do not describe a nonempty BGP JSON object as an installed route.

[FRR Zebra](https://docs.frrouting.org/en/stable-10.7/zebra.html).

