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

## Rejected suspicion — V6-NEW-001 (SSH startup)

An initial source review examined the generic Alpine startup script and suspected missing sshd startup. Inspection of the **actual published Containerlab image** disproved that inference: its final layer replaces `/usr/lib/frr/docker-start` with a script that starts sshd and then watchfrr. The adapter keeps its original `sshd -t` and relies on the pinned flavor's startup; the temporarily added duplicate daemon start was removed. [Image publication and entrypoint digests](image-publication.json) record the inspected bytes. This is not counted as a confirmed defect. The live Transport case still requires real SSH success and a wrong-host-key rejection.

## V6-NEW-002 — pyATS failures returned process status zero

The original root `tests/datacenter_state.py` discarded the value returned by `aetest.main()`. With installed pyATS 26.8, a deliberately failed AEtest case printed FAILED and returned a Failed result object, while Python exited zero. The v6 and root entrypoints now return process status one unless the aggregate result is passed. `test_pyats_failure_has_nonzero_process_exit` recreates the failure and checks the actual subprocess status.

## Limits that remain explicit

Unsupported FRR routing processes, AFs and policies fail closed; this small grammar is not a general FRR parser. Backups restore this model's managed configuration, not arbitrary device state. Hashes detect accidental/tampered bytes against a trusted index; unsigned hashes do not authenticate an attacker-controlled index. Conditional storage needs a service that enforces `If-None-Match: *`. Real Controller, EE registry and host networking acceptance remain product-specific checks.

## v5 improvement notes

I001 is addressed by new blank-file inventory construction and network-role creation exercises in chapters 02 and 06, with explicit positive and negative acceptance. I002 is addressed by Docker backend-aware forwarding diagnostics in chapter 19, sourced to Docker's official firewall documentation. These two improvement notes are separate from the 103 historical finding IDs.

## CI iteration — exact exported evidence

The first hosted run showed that two separate Dagger calls could execute the graph twice even with the same run ID (different start timestamps appeared in the log). Actions now calls Dagger once and gates the exact exported `results.json`, JUnit and ZIP with `ci/check_results.py`. The checker also requires every mapped pytest regression to appear in the actual JUnit results, preventing a merely present test file from counting as execution evidence.

## V6-NEW-003 — The original simulator used an unavailable image tag

The Docker Hub tag endpoint for `frrouting/frr:v10.5.0` returned HTTP 404 during review. The original simulator's spine/leaf profiles now use the same published FRR 10.7.1 containerlab image digest as the v6 adapter base. Quay's manifest identifies an amd64 image and other supported architectures. The legacy pyATS deployment gate checks the requested image and live node state. The v6 vendored model keeps its original pinned historical bytes; its profile image fields are unused by the v6 adapter topology.

## V6-NEW-004 — Legacy resource assertions used the wrong units and Docker fields

Hosted run 34749504898 deployed all four original nodes and passed interface checks, but the old test treated `256Mb` as 268435456 bytes and required `NanoCpus=500000000`. [Containerlab 0.79.0 Docker runtime](https://github.com/srl-labs/containerlab/blob/v0.79.0/runtime/docker/docker.go) sets CPU quota/period and uses `humanize.ParseBytes`. Its pinned [go-humanize 1.0.1 parser](https://github.com/dustin/go-humanize/blob/v1.0.1/bytes.go) distinguishes decimal MB from binary MiB. The test now checks the actual quota/period ratio, rejects missing or unlimited quotas, and interprets SI/IEC units correctly. Hypothesis exercises both unit families and generated quotas. Resource limits were not removed.

## CI iteration — verify network release after teardown

Run 34749817183 passed original simulator pyATS and built the v6 adapter. Its v6 deploy failed because the original empty management network retained the shared subnet after successful Containerlab teardown. The cleanup helper checks the exact name, expected subnet, bridge driver, Containerlab label and absence of attached containers before removing that network; it then verifies absence. It never prunes unrelated networks. Both teardown paths call it, and mutation tests reject ownership/subnet/active-endpoint mismatches. [Containerlab teardown](https://containerlab.dev/cmd/destroy/) documents management-network removal; its [pinned runtime](https://github.com/srl-labs/containerlab/blob/v0.79.0/runtime/docker/docker.go) can leave a network when endpoints remain. The observed retention is recorded without assuming an unproven underlying race.
