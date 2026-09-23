> **AGENT INSTRUCTION: DO NOT IMPLEMENT THIS DOCUMENT.** This is a documentation-only migration proposal. Do not change code, configuration, dependencies, infrastructure, or runtime state from these instructions unless the user separately requests implementation. Documentation review and correction are permitted.

# CI replacement and acceptance gates

## Replace orchestration, preserve the checks

`ci/src/datacenter_ci/main.py` currently constructs a Dagger container with Docker and Containerlab; `ci/run_ci.py` starts dockerd and runs root/v6 labs. Wrapping this inside Incus would retain the old infrastructure. Replace it with a direct runner entry point orchestrating the Rust Incus transaction and Ansible plays. Retire `prepare_dind.py`, Dagger engine/module dependencies and Docker-specific cleanup when their replacement gates pass. Keep report freshness, exact required-check accounting, failure diagnostics, artifact allowlists and package-only-on-success behavior.

Use CachyOS runners for host/runtime qualification. Fast read-only documentation and portable Rust checks may run elsewhere, but they must not be reported as CachyOS runtime evidence. Privileged and hardware jobs run only reviewed/trusted revisions on disposable or dedicated hosts; do not expose a persistent Incus daemon or GPU runner to untrusted fork code. No GitHub Actions `container:`/Docker action may quietly restore Docker dependence in the accepted runtime workflow.

## Future test matrix

| Gate | Inputs | Must demonstrate |
|---|---|---|
| Documentation | This guide, source links, objective IDs and file ledger | Links/IDs/counts/prefixes valid; no runnable file included in this documentation PR |
| Portable Rust | Existing default tests/properties | API lifecycle, bounds, topology, import and software verbs behavior unchanged |
| Compile/contracts | Pinned Incus adapter, patchbay fork, all relevant features | Real API/module schemas and error paths; no invented interface calls |
| Image | Immutable CachyOS fingerprint | Clean systemd boot, unique identity, SSH/Python, package provenance and limits |
| Incus lifecycle | Empty/different/partially failed runs | Correct ownership, idempotence/refusal, restart generations and bounded cleanup |
| Fabric | Small directed/parallel/disconnected graphs | Exact wire membership, MTU/rate/delay direction, no host/management/IPv6 bypass |
| Root lab | Original intended direct-IP topology | Connectivity, physical partition/recovery and foreign-resource preservation |
| EX457 | Routed FRR topology | Exact peers/RIB/FIB/config, backups, restart/rebuild and historical counterexamples |
| NetBox | Actual service plus offline fixtures | Signed/pinned dependencies, real API seed/import, limits/provenance and unsupported-input refusal |
| IB | Native ibsim/OpenSM | Discovery, routes, partition and cleanup; clearly management-only |
| Kubernetes/Mokka | Incus nodes, CRI/CNI and source-built image | Real upstream tests, digest/opt-in safety, pod/consumer checks and rollback |
| DeepOps CPU | Ported CachyOS roles | Actual Slurm/munge/node registration and CPU job, no fake GPU pass |
| CUDA/NCCL | Dedicated GPUs | Correct native results, exact transport/interface and real cut/failure/recovery |
| RDMA | Named physical/software RDMA device class | Real completion path and correctly labeled hardware scope |
| DeepOps GPU | Qualified host/driver/Incus nodes | Existing full benchmark/correctness gates preserved |
| Host curriculum | Disposable CachyOS recovery target | Boot, storage, SELinux, kdump and SystemTap evidence for their rows |
| Product curriculum | Qualified IDE/Controller/EE endpoints | Actual behavior for product rows, with blocked/adapted outcomes explicit |

## Property and regression coverage

Preserve existing Rust proptest and Python Hypothesis/pyATS investment. No Python embedding in the simulator is needed to retain external test harnesses. Keep v5 historical counterexamples as inert fixtures; do not run its retired Containerlab deployment to exercise a predicate.

Require properties for deterministic plan normalization; unique node/interface/link mapping; no host transit; correct parallel-edge identity; bounded bytes/resources; refusal before side effects for invalid intent; unchanged desired state on failed apply; cleanup limited to created owned objects; idempotent destroy; and no traffic after all real paths are cut. Use generated graphs small enough for real kernel tests and larger graphs only for portable algorithm tests.

Test delayed/missing Incus acknowledgements, failed starts, canceled API operations, daemon restart, stale ETags, PID/ifindex reuse, image mismatch, foreign bridge members, namespace handle invalidation, qdisc partial failure and failed rollback. Unit mocks must model errors honestly; a mock returning success for every API is not a lifecycle test.

For EX457 retain all historical predicates concerning extra peers, wrong interfaces, route not installed, backup hash/path/identity, HTTPS/host-key trust, Controller graph drift and stale evidence. A wrapper replacement must not simply delete tests mentioning Docker; extract the behavioral assertion and rebind its observation method.

## Verdict schema and artifacts

Each report needs schema version, run ID, exact source commit, desired-plan hash, image fingerprint, fork revisions, package/collection/kernel versions, execution class, start/end time, check list, command status, result scope and cleanup outcome. Keep `PASS`, `FAIL`, `BLOCKED`, `NOT_RUN`, and adaptation metadata separate. A gate is green only when every required check for its declared profile actually passed; omitted rows or stale reports fail validation.

Keep artifacts useful after failure: redacted Incus operations/config, service journals, interface/routes/qdiscs, packet/counter evidence, Ansible recap/events, test XML and cleanup ledger. Do not archive private keys, tokens, whole database dumps, `.state` or unredacted core/pcap content by default. Preserve original failure and cleanup failure independently.

Hardware promotion must continue to require trusted, recent results for the exact commit from the revised NCCL/RDMA/DeepOps workflows. Existing `hardware-evidence.yml` policy is not replaced by CPU compilation. Branch-protection changes are a separate repository-policy action, not something this guide claims to have performed.

## Criteria for claiming replacement complete

Run the full supported deployment/teardown on a qualified CachyOS host where Docker, Containerlab, Kind and Dagger are unavailable. Trace invoked executables and check dependencies, not just executable names in Markdown. The root and EX457 labs must deploy through the same Incus transaction and pass actual fault tests. NetBox, Mokka and DeepOps paths must pass their declared profiles or remain explicitly unavailable; no silent fallback is allowed.

Perform a cold host restart, observe/reconcile retained labs and verify intended services/data wiring. Destroy an owned lab and confirm no owned NICs, bridges, namespaces, containers, processes or temporary secrets remain; unrelated host resources must remain intact. Publish measured resource usage and actual incomplete capability rows.

For **this documentation-only PR**, verify only the documentation contract, file scope, mappings and review consistency. Do not execute proposed deployment or fault-injection commands and do not manufacture runtime results.
