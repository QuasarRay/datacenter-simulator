> **AGENT INSTRUCTION: DO NOT IMPLEMENT THIS DOCUMENT.** This is a documentation-only migration proposal. Do not change code, configuration, dependencies, infrastructure, or runtime state from these instructions unless the user separately requests implementation. Documentation review and correction are permitted.

# Ordered cutover and removal

These are future implementation stages. Staging is for review and qualification; the delivered architecture has one supported infrastructure runtime. Do not expose two permanent backend choices or leave a Docker fallback.

## Stage 1 — Freeze and translate contracts

Record the current commit, component pins, image/config hashes, root/v6 intent, active resource ownership and backups. Enumerate all external execution calls and generated artifacts using the file ledger. Specify the common plan schema, input validators, image identity, routing mode, output/evidence schema and typed lifecycle errors before rewriting callers.

Translate each current field deliberately: Docker image digest → qualified Incus fingerprint; `cpu: 0.5` → tested quota/allowance semantics; memory strings → validated bytes; bind files → protected guest files/volumes; Containerlab labels → authenticated owner UUID/config generation; node names → stable object IDs plus display names; topology file → validated plan plus resource journal. Preserve ports, addresses, ASNs, interfaces, edge direction, MTU and startup-config ownership. Reject untranslatable fields.

## Stage 2 — Qualify host and image

Implement later the CachyOS host/preflight and reproducible image process from [03](03-cachyos-host-and-images.md). Prove a pair of unprivileged instances with distinct identities, working systemd/SSH and actual resource limits. Keep the baseline tiny so failures are attributable. No change to the supported user entry point occurs until these prerequisites pass.

## Stage 3 — Qualify lifecycle and fabric

Implement later the Incus transaction and patchbay ownership extension from [04](04-incus-lifecycle.md) and [05](05-patchbay-petgraph-networking.md). Test create/start/wire/failure/stop/restart/destroy, stale handles and rollback on a two-node link. Then qualify parallel links, routed switches, management isolation and IPv6. Update patchbay gitlink/provenance only with tested fork commits.

## Stage 4 — Replace both lab callers

Switch root deploy/destroy and current EX457 tooling to the common implementation. Retire their `.clab` templates, Docker image/entrypoint builders and Docker inspection/exec paths. Preserve current behavioral tests and source predicates. Update inventories, README/getting-started, version locks, packaging names and supported commands in the same transition. v5 becomes strictly historical fixture content.

## Stage 5 — Replace integration provisioning

Replace NetBox Compose with Incus system services; retain real API import checks. Replace Kind and Docker-based registry/image workflows with Incus nodes and qualified CRI/OCI build/distribution tools. Port DeepOps tasks to CachyOS and Incus, retire QEMU/VFIO guest provisioning, and preserve native Slurm/NCCL evidence. Do not make unsupported DeepOps roles appear compatible by overriding OS facts.

Rebind GPU workers, ibsim and physical RDMA profiles to their qualified execution boundaries. Document any remaining modeling/physical limitations rather than removing the corresponding tests.

## Stage 6 — Replace CI and evidence

Replace Dagger/DinD with direct orchestration, update every workflow from [01](01-repository-architecture.md), and preserve exact verdict gates/artifact redaction. Requalify all changed native paths. Existing historical evidence remains historical; regenerate runtime claims and component identities for the new release.

## Stage 7 — Operational cutover

1. Freeze changes to the old lab and create verified application/configuration backups, including NetBox data and SSH identity policy where relevant.
2. Stop old workloads and remove only recorded project-owned Containerlab/Docker resources using the old version's controlled teardown. This one-time retirement step is not a continuing runtime dependency. If ownership is ambiguous, investigate instead of pruning globally.
3. Check that address pools, names and volumes for the new plan are free. Deploy the Incus release, restore intended application state, and run the required acceptance/fault/persistence tests.
4. Publish the new supported commands and remove old operational recipes/dependencies. Retain rollback artifacts and source commit metadata outside the active runtime.
5. Verify setup, use, CI and teardown on a host without the retired tools. Audit indirect scripts, Python subprocesses, Docker GitHub actions, sockets, registry helpers and generated archive names.

## Deletion and retention policy

Remove active Dockerfiles, Compose/Kind configs, `.clab` templates, Dagger/DinD dependencies, Docker socket access, old backend flags and QEMU guest setup once replaced. Keep upstream source files containing OCI recipes as pinned vendor source if needed to understand/build the upstream artifact; the project must use its qualified Docker-free build path. Do not edit vendor history merely to make a text search return zero.

Historical v5/v6 audit excerpts may mention Docker/Containerlab as evidence. Mark such material historical/inert and exclude it from supported deployment discovery. Do not erase counterexamples or licenses. Update documentation links, archive manifests and provenance to explain relocated fixtures. New current instructions must have no dependency on running old backend recipes.

Search patterns for a future removal audit:

```bash
rg -n -i 'docker|dockerd|containerlab|\.clab\.|compose|kind-simulator|dagger|dind|qemu|qcow2|vfio' \
  playbooks templates vars inventory tests ci integrations simulator \
  cheatsheets/ex457 .github docs README.md
```

Classify every hit as active dependency, vendor source, historical evidence or migration discussion. An allowlisted historical occurrence is acceptable; an executable fallback is not. Never substitute global `docker system prune` or delete all Incus resources.

## Rollback

If acceptance fails before promotion, stop the new owned lab, export diagnostics and restore the previous release **as a controlled rollback**, with its recorded state/backups. Do not run both fabrics on overlapping management networks. Rollback does not justify shipping dual backends. Database rollback may need backup restoration rather than schema downgrade. Once the Incus release is accepted, retain versioned recovery artifacts according to policy and remove obsolete active dependencies.

Completion requires both architecture gates and the objective coverage report. Mark unavailable host/product exercises visibly; never label all four exams practically validated from a CPU-only Incus run.
