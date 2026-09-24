> Implementation authorized by the project owner on 2026-09-24. The former implementation prohibition is withdrawn. See integrations/incus for implemented behavior and validation boundaries.

# Replace Compose services and preserve NetBox intent

## Replace the actual service stack

The current `integrations/netbox/compose.ci.yml` supplies NetBox, PostgreSQL and Valkey/Redis-compatible state. `seed-ci.sh` seeds a disposable database and produces a read-only import token. Replace this stack with CachyOS Incus service nodes and Ansible roles; do not run the same Compose file inside Incus.

Use separate database and application nodes for failure exercises, or one service node for a measured minimal lab with clearly stated fault coupling. Run PostgreSQL, the compatible Redis service, the NetBox web service and background worker as systemd services. Provision a reverse proxy/TLS endpoint when needed. Follow the **pinned NetBox source's** supported Python/PostgreSQL/Redis versions; the current rolling CachyOS versions might not form a supported combination. Build a pinned isolated application environment or fail qualification rather than disabling version checks.

Future sequence:

1. Build qualified service images from [03](03-cachyos-host-and-images.md); create protected data volumes and service accounts with explicit ownership.
2. Configure PostgreSQL authentication, database and role with least needed rights; use a dedicated NetBox database role, not superuser application credentials. Restrict listeners/firewall to service peers.
3. Configure Redis-compatible database separation, credentials and persistence as required by the pinned NetBox release. Preserve the cache/queue distinction in the current example.
4. Install NetBox from the recorded source revision in a virtual environment. Template database/cache/settings/secret-key configuration, run migrations once through a serialized job, collect static content and start web/background workers with proper dependencies.
5. Verify API readiness with an authenticated read and actual database objects, not merely an open TCP port. Apply the fixture seed through an explicit ephemeral-lab task. Create the read-only importer token using the supported API/version and keep its value out of logs.
6. Import, verify the expected node/interface/cable graph, then run ibsim or Ethernet tests on the resulting manifest. Capture application, database and worker logs before teardown.

The public example credentials are CI fixture values, never production defaults. Replace Compose environment variables with managed protected settings. Preserve token pepper/secret material across a same-instance restore and include it in protected backup planning.

## Preserve the importer contract

`simulator/src/netbox.rs` is a Rust HTTP client; leave it independent of NetBox's implementation language and provisioning mechanism. Preserve role mappings, `nb<ID>` object identity, interface-name encoding, cable endpoint validation, speed-unit conversion, admin status and the selected Ethernet/InfiniBand medium. Unsupported passive panels, breakout, bridge/parent interfaces and LAGs must continue to fail explicitly until their semantics are designed.

Keep same-origin pagination, redirect rejection, request/byte/record/deadline budgets, token redaction, two matching complete observations and offline replay provenance. A double read reduces observed inconsistency; it is not database snapshot isolation. Use a NetBox change window or a qualified export for strict input consistency. Do not let an importer refresh mutate a running topology without the lifecycle transaction's change policy.

Distinguish **NetBox desired state**, **compiled plan**, and **observed Incus/FRR/Kubernetes state**. Never write observed failures back into NetBox merely to make the plan match a failed deployment. A drift report lists missing/extra nodes, changed interfaces, image/config mismatch and actual failed links, with source object IDs.

## Supporting services for the curriculum

| Service | Ansible work | Required exercise evidence |
|---|---|---|
| DNS | Zone/record or resolver configuration, listener policy and systemd unit | Resolve forward/reverse names from the affected namespace; break upstream/record separately. |
| Time | Configure host chrony/time source; containers observe shared host clock | Demonstrate source selection/offset; do not grant every container clock-setting authority. |
| Central logs | Journal persistence and rsyslog/syslog-ng forwarding, TLS where used, bounded storage | Inject a unique node/run marker and find it at the collector; test destination outage/backpressure. |
| NFS/autofs | Dedicated exports, identity/permissions and explicit mount options | Test cold automount, server loss, access denial and persisted restoration. |
| Identity | Local users plus a qualified LDAP/Kerberos/SSSD-style lab when available | Separate name lookup, credential validation, group authorization and home-directory access. |
| Backup endpoint | Restricted account/path or object API, integrity manifest and retention | Restore actual content to a new instance and compare hash/ownership, not just upload status. |
| Cockpit | Install/configure the CachyOS package if available; bind to management, authorize users | Compare displayed service/storage/log state to Ansible-collected observations. |

Do not infer these services exist merely because `model.rs` has service metadata. Add each operational role and its tests in a future implementation. Identity product differences (for example FreeIPA vs a generic LDAP provider) must remain visible.

## Backup, upgrade and removal

Back up the database, NetBox media/customizations, configuration and protected secrets coherently. Restore to a fresh isolated service group; verify the importer reconstructs the same supported intent. Before upgrades, pin the new dependency set, test migrations on a restored copy and test rollback using backups rather than assuming database downgrades are supported.

At cutover retire Compose/seed lifecycle calls and replace `intended-fabric.yml` setup/log/teardown stages together. Preserve the source fixture JSON and importer tests. Package caches and database volumes are not garbage unless the run ledger explicitly owns them and their retention policy permits deletion.

Sources: [NetBox installation](https://netbox.readthedocs.io/en/stable/installation/), [pinned NetBox source](https://github.com/netbox-community/netbox/tree/dbec965a5186a40971312c9331dfb9db46cc9183), baseline `integrations/netbox/**` and `simulator/src/netbox.rs`.
