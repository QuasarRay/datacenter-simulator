# Air API specification coverage

Specification: [QuasarRay/nv-air-sdk at 0a9ce86a6195e26c5522640d895a1adba9449cbd](https://github.com/QuasarRay/nv-air-sdk/tree/0a9ce86a6195e26c5522640d895a1adba9449cbd). The SDK describes Python client-side models, payloads and endpoints; it delegates many server behaviors and JSON/DOT topology parsing to the hosted API. This implementation uses those concepts as a blueprint for typed local Rust behavior. It does not claim REST wire compatibility or complete SDK feature parity.

## Core local operations

| SDK source / behavior | Rust implementation | Evidence / limits |
|---|---|---|
| [simulations.py](https://github.com/QuasarRay/nv-air-sdk/blob/0a9ce86a6195e26c5522640d895a1adba9449cbd/src/air_sdk/endpoints/simulations.py): create/list/get/update/delete | Simulator and Simulation in api.rs | lifecycle, command-session, strict mutation tests; local synchronous ACTIVE/INACTIVE states |
| simulations: start/shutdown/rebuild/wait_for_state | Simulation lifecycle methods | state guards, generation invalidation, atomic failures; wait observes synchronous completion |
| simulations: sleep_at/expires_at | set_schedule + Simulator::tick | explicit caller-driven wall-clock scheduling; deadline tests |
| simulations: import/export/clone | manifest.rs; clone_simulation | transactional import, fresh IDs, isolated clone state, round trip; JSON only |
| simulations: node_bulk_assign/reset/rebuild | assign_configs + reset_nodes | bulk input validation before mutation; configs contain local content, not hosted userconfig foreign keys |
| [nodes.py](https://github.com/QuasarRay/nv-air-sdk/blob/0a9ce86a6195e26c5522640d895a1adba9449cbd/src/air_sdk/endpoints/nodes.py): nodes and resource sizing | create_node/update_node/delete_node; nodes/node/node_named; Resources | positive sizes, scoped IDs, names, cascade deletion; resource totals are descriptive |
| [interfaces.py](https://github.com/QuasarRay/nv-air-sdk/blob/0a9ce86a6195e26c5522640d895a1adba9449cbd/src/air_sdk/endpoints/interfaces.py): create/get/list/delete, breakout/revert | interface methods; typed DATA_PLANE_INTF/OOB_INTF/PCIE_INTF | endpoint uniqueness, unused port checks, 2/4/8-lane breakout; general PATCH is not exposed |
| [links.py](https://github.com/QuasarRay/nv-air-sdk/blob/0a9ce86a6195e26c5522640d895a1adba9449cbd/src/air_sdk/endpoints/links.py): create/list/get/delete | link methods | two distinct nodes, compatible unused endpoints, reciprocal connection fields; bandwidth/latency/up are local extensions |
| simulations: auto OOB + DHCP | set_auto_oob | logical management addresses; isolated from data graph; no DHCP packet server |
| [checkpoints.py](https://github.com/QuasarRay/nv-air-sdk/blob/0a9ce86a6195e26c5522640d895a1adba9449cbd/src/air_sdk/endpoints/checkpoints.py) and simulation checkpoint arguments | create/update/delete/checkpoints; start/rebuild(checkpoint) | modeled guest state snapshots and clone ID remapping; no VM RAM/disk snapshots |
| [node_instructions.py](https://github.com/QuasarRay/nv-air-sdk/blob/0a9ce86a6195e26c5522640d895a1adba9449cbd/src/air_sdk/endpoints/node_instructions.py) | create/delete/list instructions; typed init/file data | virtual hostname/files and rebuild policy; Rust file input is a map; shell explicitly Unsupported |
| [ztp_scripts.py](https://github.com/QuasarRay/nv-air-sdk/blob/0a9ce86a6195e26c5522640d895a1adba9449cbd/src/air_sdk/endpoints/ztp_scripts.py) | create/update/delete/get script | content round trip; guest execution explicitly Unsupported at start |
| [services.py](https://github.com/QuasarRay/nv-air-sdk/blob/0a9ce86a6195e26c5522640d895a1adba9449cbd/src/air_sdk/endpoints/services.py) | create/delete/list immutable descriptors | port validation, endpoint ownership and cascade deletion; worker_port/fqdn remain None |
| [history.py](https://github.com/QuasarRay/nv-air-sdk/blob/0a9ce86a6195e26c5522640d895a1adba9449cbd/src/air_sdk/endpoints/history.py) | immutable history entries + distinct filters | timestamp/actor/severity/labels; local simulation history only |

The Rust Command enum exposes the common operations over JSON lines. Typed library methods additionally expose schedule ticks, configuration assignments, required resources, checkpoint metadata updates and software verbs. Runtime state is owned by each Simulator session. UUIDs are opaque identities; exported topology uses names.

## Manifest contract

The top-level `format`, `name`, `ztp`, `content.nodes` and `content.oob` structure follows the SDK's [export example](https://github.com/QuasarRay/nv-air-sdk/blob/0a9ce86a6195e26c5522640d895a1adba9449cbd/docs/files/exports/export_without_oob.json). Node CPU/memory/storage/OS fields and extra image metadata survive import/export. The test imports the supplied SDK example envelope including positioning, features and network_pci.

The SDK repository does not contain the hosted topology parser's full schema. Therefore the link endpoint object, role, interface catalog, explicit link conditions, services and instructions shown in examples/gpu-spine-leaf.json are documented **local extensions**, not asserted to be accepted by NVIDIA's server. Unknown structural fields error rather than silently disappearing. Unknown node image metadata is retained as metadata only. Import format must be JSON; DOT/import_from_dot/parse are unsupported.

## SDK families outside this implementation

| SDK family | Disposition |
|---|---|
| images, image shares, upload/download, publish/access records | No hosted image storage or publisher; node image/profile labels are preserved, guest images are not booted |
| user_configs, cloud-init compatibility layer | Local content assignment exists; no catalog with hosted IDs, template execution or guest cloud-init |
| systems, platform_information, OS/plugin manifests, os_templates | No NVIDIA system catalog or container/VM platform provisioning; metadata is not claimed as execution |
| fleets, workers, worker certificates | No remote scheduling, worker enrollment, certificate authority or NGC integration |
| organizations/resource budgets | Local aggregate resource totals only; no organization membership, quota enforcement or billing |
| marketplace demos, tags, publish access records | No publishing or marketplace workflow |
| trainings, attendees, workbenches | No NGC groups, attendee invitations or managed training service |
| client authentication, retries, HTTP pagination and deprecated v2 aliases | No Python client or HTTP server compatibility layer; local typed Rust API and JSON commands |
| DOT import/parse, VM boot/reset, guest ZTP, shell/file deployment to an OS, public service forwarding | JSON topology supported; virtual state and real network namespaces implemented; remaining guest/hosted operations unavailable |

NCCL, software verbs, timing traces and Linux link faults extend the SDK's topology/lifecycle blueprint with executable data-plane behavior; the Air SDK itself does not specify these GPU/NIC algorithms.
