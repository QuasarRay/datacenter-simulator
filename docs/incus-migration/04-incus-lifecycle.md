> **AGENT INSTRUCTION: DO NOT IMPLEMENT THIS DOCUMENT.** This is a documentation-only migration proposal. Do not change code, configuration, dependencies, infrastructure, or runtime state from these instructions unless the user separately requests implementation. Documentation review and correction are permitted.

# Incus lifecycle, transactions, storage and recovery

## Define the adapter before its callers

Proposed `IncusClient`/`IncusLab` names in this chapter are design labels, not existing Rust APIs. Provide typed operations for discovery, planning, creating stopped instances, configuring profiles/devices, starting, executing with bounded I/O, observing, stopping, snapshotting and deleting. The outer interface returns operation identity, changed state, evidence and cleanup outcome. Separate declared resource limits from observed usage.

Prefer the Incus REST API through the local Unix socket or authenticated TLS. If an initial implementation uses the CLI, use argument vectors, explicit remote/project, structured JSON, verified executable identity and bounded output/deadlines. Never scrape colorized tables or concatenate manifest strings into a shell command. Ansible can invoke this same adapter; it must not implement a competing transaction engine.

Discover `/1.0` and check required `api_extensions`; a client version alone is insufficient. URL-encode names and query parameters. Pin API assumptions and test ETag/`If-Match` updates to avoid overwriting concurrent operator edits. Incus mutation requests can return asynchronous operations: wait for terminal operation status and inspect its error, then read the resulting instance/device state. An HTTP success or CLI launch alone is not readiness. Preserve the outer wrapper and nested operation error separately.

## Proposed lifecycle protocol

| Phase | Required action | Failure disposition |
|---|---|---|
| Discover | Check daemon, project, pool, image, extensions, privilege and available resources | No mutation; actionable missing prerequisite |
| Plan | Validate source hash, names, networks, role/capability requirements and aggregate budgets | No resource allocation |
| Reserve | Lock lab UUID and run generation; compare desired plan with any existing ledger/objects | Refuse foreign or mismatched objects; never adopt by name alone |
| Create | Create nodes stopped with explicit profiles, owner labels, fingerprints and limits | Journal each acknowledged object before proceeding |
| Bootstrap | Install identity/configuration using the authorized guest file/exec channel | Record changed state, protect keys; no shared-root shortcuts |
| Start | Start instances and wait for systemd/exec readiness | Collect console/daemon logs; bounded cleanup |
| Attach | Build fabric, apply routing ownership and management isolation | Reverse created attachments on any error |
| Configure | Run Ansible roles; validate configuration and flush required handlers | Preserve original error through rescue/cleanup |
| Verify | Check identities, services, traffic, isolation, limits and expected inventory | Do not publish ready state on a partial pass |
| Publish | Atomically publish inventory and evidence tied to plan/run IDs | Avoid stale inventory files being treated as live |

Retain the root lab's conservative first-migration rule: unchanged live intent may be verified/reused; changed intent requires an explicit migration/recreate transaction. Do not secretly destroy a user's active lab to converge a different plan. A future online reconciler needs its own semantics and tests.

## Device and privilege policy

All ordinary nodes are unprivileged system containers. GPU, RNIC, nested Kubernetes, filesystem, and tracing access require named capability profiles with concrete reasons. Do not default to privileged containers, host PID/network namespaces, writable host `/sys`, unrestricted device access, or unrestricted syscall interception.

Management NICs use the dedicated managed network. Data NICs use Incus `p2p` with stable per-run `host_name` and guest `name`; patchbay joins the host peers to owned per-wire bridges as specified in [05](05-patchbay-petgraph-networking.md). Expanded profile configuration must be checked for inherited NICs and defaults. Reattach after every container restart because PIDs, ifindices, namespace handles and veths may change.

Use CPU allowance and memory/process limits appropriate to the tested Incus version. Read back effective cgroup limits. Avoid applying both Incus network rate limits and patchbay netem rate limits to the same path. Quotas depend on storage backend; reject a required hard quota when the selected backend cannot enforce it.

## Volumes, backup and identity

Classify data as reproducible image content, per-node mutable configuration, application state, secrets, and evidence. Use dedicated volumes for databases and retained lab artifacts with explicit UID/GID mapping. Avoid broad `raw.idmap` rules to paper over ownership; demonstrate the exact mapping and test access in both directions. Never bind the repository root or host home writable into every guest.

Back up PostgreSQL logically and store media/config/secrets through separate protected channels. Snapshot only after application quiescence where consistency requires it. Record whether external volumes and service dependencies are included. Restore into an isolated project first; verify data consistency and the restored topology before promoting it.

A fresh node gets new machine-id and SSH keys. A logical same-node restore may preserve identity, but the controller must confirm the authenticated Incus source and expected fingerprint. Updating `known_hosts` must be a verified identity operation, not `StrictHostKeyChecking=no`.

## Stop, delete and interrupted execution

Stop workloads and quiesce data, detach fabric, stop containers with a bounded grace period, delete owned instances/devices, remove empty owned profiles/networks/volumes, and retain the evidence/cleanup ledger. A destroy operation must discover owned runtime objects even if a generated topology file was lost. Empty-name or substring matching is never sufficient ownership proof.

On process cancellation, request cleanup and report any resources that remain. On controller crash/restart, compare journal entries to Incus and kernel state, including operations still in progress. Names can be reused; verify owner UUID, image/config hash and generation. If an object has changed ownership or contains foreign members, stop cleanup and report the conflict. Do not delete an entire Incus project that also contains resources outside this run.

Required lifecycle tests: stop/start changes namespace generation; repeated destroy is harmless; partial start cleans only newly created nodes; daemon restart is recoverable; operation timeout followed by eventual success is reconciled; concurrent deployment cannot duplicate ownership; stale PID/ifindex is rejected; existing unrelated containers survive all failures.

Sources: [Incus REST API](https://linuxcontainers.org/incus/docs/main/rest-api/), [API specification](https://linuxcontainers.org/incus/docs/main/api/), [instance options](https://linuxcontainers.org/incus/docs/main/reference/instance_options/). The transaction protocol above is a proposed project requirement, not an Incus feature claim.
