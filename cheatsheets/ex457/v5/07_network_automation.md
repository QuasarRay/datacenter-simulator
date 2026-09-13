# 07 — Configure the fabric

## Candidate and apply

```bash
ansible-playbook playbooks/configure_fabric.yml --syntax-check
ansible-playbook playbooks/configure_fabric.yml --diff
ansible-playbook playbooks/verify_fabric.yml
python tools/labctl.py dataplane
```

The Jinja candidate assigns loopbacks and /31 link endpoints, sets local ASN/router ID, and creates directly attached eBGP neighbors. It advertises the local /32. `no bgp ebgp-requires-policy` permits this isolated lab's route exchange; a production border requires explicit policy.

`cli_config` sends platform CLI through the platform's cliconf implementation. It is not universally a replace operation. The role first computes explicit removals from observed managed state; repeated runs should converge. Test second-run changed counts on the actual FRR/plugin combination.

[cli_config module](https://docs.ansible.com/projects/ansible/latest/collections/ansible/netcommon/cli_config_module.html). [FRR BGP](https://docs.frrouting.org/en/stable-10.7/bgp.html).

## Ownership

Managed scope is the default VRF's numeric IPv4 BGP peers, local ASN/router ID, IPv4 network statements and all non-`eth0` IPv4 address configuration. `eth0` management belongs to Containerlab. VRFs, named peer-groups, alternate address families and unsupported address syntax are outside this exercise; ambiguous managed syntax raises an error.

Reconciliation can remove stale addresses and, if the modeled ASN changes, replace the lab BGP process. Work only on this disposable fabric. A successful config push is followed by exact control-plane verification; chapter 08 adds kernel and ping gates before saving.

[FRR Zebra](https://docs.frrouting.org/en/stable-10.7/zebra.html). [cli_config module](https://docs.ansible.com/projects/ansible/latest/collections/ansible/netcommon/cli_config_module.html).

