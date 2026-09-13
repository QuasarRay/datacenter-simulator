# 08 — Exact verification and failure evidence

## State contract

| Layer | Predicate | Implementation |
|---|---|---|
| Model | Unique management/loopbacks/endpoints; connected /31 fabric | `fabric_model` |
| Configuration | Interface → normalized addresses; exact peers/ASNs/router ID/networks | `fabric_config_ok` |
| BGP sessions | Exact unfiltered set, Established, correct remote ASN | `fabric_peers_ready` |
| BGP RIB | Exact prefix; valid best path; modeled next hop | `fabric_bgp_ok` |
| Zebra RIB/FIB report | BGP, selected, installed, active next hop marked FIB | `fabric_rib_ok` |
| Kernel | Exact remote /32; unicast through modeled next hop | `labctl dataplane` |
| Data plane | Three loopback-sourced probes, three replies, zero numeric loss | `fabric_ping_ok` |
| Saved state | Same managed-state comparison on startup config | `persist_fabric.yml` |

```bash
ansible-playbook playbooks/verify_fabric.yml
python tools/labctl.py dataplane
```

Both commands must pass. The first command alone establishes configuration/control-plane checks. The second runs `ip -j -4 route show exact` and `ping -I` inside each router namespace on the lab host, covering all 12 ordered router pairs. Controller's network jobs do not have a Docker socket; run the namespace gate on the lab host.

[FRR BGP](https://docs.frrouting.org/en/stable-10.7/bgp.html). [FRR Zebra](https://docs.frrouting.org/en/stable-10.7/zebra.html).

## Schema and test limits

The parser requires the explicit FRR JSON fields named in [the filter implementation](filter_plugins/fabric.py). Unit/Ansible dataflow fixtures are **synthetic**, not packet captures or live FRR JSON. FRR manuals support command semantics, but do not establish every version's complete JSON layout. The first live run must archive actual outputs and confirm those paths. An unknown/missing path fails closed; do not weaken predicates to make a new version pass.

Negative cases cover 33%, 66%, 100%, fractional loss, no summary, duplicate summaries, extra peers in three states, wrong ASN, wrong interface, extra loopback, missing route, invalid/nonbest path and uninstalled FIB state. These establish predicate behavior, not live convergence.

[Ansible filters](https://docs.ansible.com/projects/ansible/latest/playbook_guide/playbooks_filters.html).

