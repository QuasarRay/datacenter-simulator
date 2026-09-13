# v6 fixes and newly discovered defects

v6 preserves the v5 teaching sequence and fixes the audited implementation. See [all historical findings](audit-history.md) for individual mappings, source hashes and remaining acceptance.

| Area | Change |
|---|---|
| SSH bootstrap | Remove the inventory reference to a future registered result; publish a concrete libssh configuration path after creating it. Validate complete public keys with ssh-keygen. |
| Configuration | Preserve BGP/process/address-family scopes; reject unsupported managed grammar. Require unique ASNs, usable unicast router IDs and a reserved management gateway. |
| Forwarding | Every selected installed path must use modeled next hops and expected egress interfaces; check kernel routes and strict ping counts. |
| Preflight | Compare actual rendered topology and bind sources with independent model-derived expectations. Validate effective inventory and exact tool version. |
| Backups | Private directories/files, exclusive run reservation, durable per-node size/hash index, conditional HTTPS export with readback, verified download and restore. |
| Runtime evidence | Unique run IDs, timestamps, source/model hashes; record RUNNING before work and FAIL after exceptions. |
| Controller | AWX 24.6.1 tower namespace, optional machine key unlock field, superuser prerequisite for new custom types, removal of extra workflow nodes and diagnostic edges, exact graph verification. |
| Release evidence | Exact nonempty file manifest and claim registry; generated human closure tables match JSON; all 103 available historical entries mapped. |

## V6-NEW-001 — SSH daemon was never started

The inherited adapter called `sshd -t` but did not invoke the daemon. The pinned FRR [Alpine docker-start source](https://github.com/FRRouting/frr/blob/frr-10.7.1/docker/alpine/docker-start) starts watchfrr and does not start SSH. v6 explicitly starts `/usr/sbin/sshd` after its configuration test. The live Transport case requires successful real libssh access and a specific wrong-host-key rejection. This finding was not present in the supplied v5 audit.

## V6-NEW-002 — pyATS failures returned process status zero

The original root `tests/datacenter_state.py` discarded the value returned by `aetest.main()`. With installed pyATS 26.8, a deliberately failed AEtest case printed FAILED and returned a Failed result object, while Python exited zero. The v6 and root entrypoints now return process status one unless the aggregate result is passed. `test_pyats_failure_has_nonzero_process_exit` recreates the failure and checks the actual subprocess status.

## Limits that remain explicit

Unsupported FRR routing processes, AFs and policies fail closed; this small grammar is not a general FRR parser. Backups restore this model's managed configuration, not arbitrary device state. Hashes detect accidental/tampered bytes against a trusted index; unsigned hashes do not authenticate an attacker-controlled index. Conditional storage needs a service that enforces `If-None-Match: *`. Real Controller, EE registry and host networking acceptance remain product-specific checks.

## v5 improvement notes

I001 is addressed by new blank-file inventory construction and network-role creation exercises in chapters 02 and 06, with explicit positive and negative acceptance. I002 is addressed by Docker backend-aware forwarding diagnostics in chapter 19, sourced to Docker's official firewall documentation. These two improvement notes are separate from the 103 historical finding IDs.
