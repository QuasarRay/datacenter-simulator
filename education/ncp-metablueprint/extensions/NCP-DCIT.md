# NCP-DCIT adaptation obligations

This independent extension preserves the three NVIDIA domains as a conjunctive core. It is not an official certification.

Source: [Cisco DCIT 1.2](https://learningcontent.cisco.com/documents/marketing/exam-topics/300-615-DCIT-v1.2.pdf), reviewed 2026-09-27. The registry retains 52 numbered leaves and 84 inline concepts. A parent heading is covered by its children. Entries are project-authored investigations, not exam questions.

The ledger specifies work; it does not claim that these courses, projects, collectors or hardware gates are complete. No extension entry can grant live mastery. NCP-DCIT reuses the core three-domain evidence rule; one or two domains grant no unit.

The units below are the delivery sequence. All NVIDIA courses precede extension projects. New skills must first gain a course, then project practice, then an independent exercise. Source-specific mechanisms stay visible even when the NVIDIA adaptation differs.

## D01 · Establish rack and adapter identity

Planned chain: DC01 → DP01 → DE01a, DE01b.

| Source locator / concepts | GPU-datacenter investigation | Required observations | Boundary and NVIDIA core |
|---|---|---|---|
| p1 `2.1` · rack-server | Establish the physical GPU server identity before replacing or reprovisioning anything. | Correlate rack position, serial, BMC inventory, power state, adapter presence and a named affected workload. | An Incus guest cannot certify a physical rack server or Cisco UCS hardware. Core: U02. |
| p1 `2.2.a` · chassis, power, io-module | Localize a shared rack failure across chassis, power feed and I/O paths. | Retain independent power and module observations plus the affected-host set before choosing a maintenance boundary. | System containers do not simulate electrical redundancy or chassis module service procedures. Core: U02. |
| p2 `2.4.a` · vic-adapter | Check the adapter identity and host driver agreement before blaming the fabric. | Correlate PCI function, firmware, driver, link mode and the bound data path. | ConnectX or BlueField inspection is an adaptation, not a Cisco VIC emulation. Core: U08. |
| p2 `2.4.b` · firmware | Determine whether a component firmware revision belongs to the supported compatibility tuple. | Record actual model, revision, driver and release-matrix decision with a reproducible pre-change workload. | A source pin is not a hardware compatibility certificate. Core: U17. |
| p2 `2.4.c` · io-module | Locate an I/O module mismatch that disconnects several hosts together. | Correlate slot, lane mapping, module revision, physical links and common affected endpoints. | Virtual NIC presence cannot certify chassis I/O module interoperability. Core: U02. |
| p2 `2.4.d` · fabric-interconnect | Separate a server uplink fault from a shared fabric interconnect fault. | Compare both sides of each connection and the blast radius across independent hosts. | An Ethernet switch-role guest does not implement UCS fabric interconnects. Core: U02. |

## D02 · Localize a broken host-to-fabric path

Planned chain: DC02 → DP02 → DE02a, DE02b.

| Source locator / concepts | GPU-datacenter investigation | Required observations | Boundary and NVIDIA core |
|---|---|---|---|
| p1 `1.1.a` · ospf-v2, ospf-v3 | Locate IPv4 and IPv6 adjacency, area, MTU or route-installation disagreement along a GPU service path. | Compare intended peers with both neighbor databases, selected routes and a source-bound payload in each address family. | Static routes are not an OSPF implementation; both protocol versions need an explicit routing adapter. Core: U05. |
| p1 `1.1.b` · mp-bgp | Separate BGP session health, address-family exchange, policy rejection and forwarding installation. | Retain received and selected prefixes, next-hop resolution, return path and application transfer before and after repair. | A graph path does not prove BGP convergence; real FRR or qualified switch observations are required. Core: U05. |
| p1 `1.1.c` · pim | Diagnose multicast receiver registration, reverse-path selection and tree state in surrounding CPU services. | Trace one identified source and group through membership, control state and captured delivery. | Do not assume the GPU collective uses IP multicast; identify its actual transport first. Core: U05. |
| p1 `1.1.d` · fhrp | Distinguish gateway ownership loss from duplicate ownership during a CPU service failover. | Observe gateway identity, neighbor resolution, interruption interval and post-failover bidirectional traffic. | An anycast gateway and a first-hop election protocol have different failure models; qualify the selected mechanism. Core: U20. |
| p1 `2.3` · server-fabric-packet-path | Trace one CPU-side data transfer required by a GPU job from the named source interface to its storage peer. | Bind NetBox endpoints, guest routes, packet counters, exact payload and cut/restore evidence to one attempt. | Linux Ethernet evidence does not establish GPU execution, switch ASIC forwarding or RDMA. Core: U16. |

## D03 · Separate link redundancy from overlay reachability

Planned chain: DC03 → DP03 → DE03a, DE03b.

| Source locator / concepts | GPU-datacenter investigation | Required observations | Boundary and NVIDIA core |
|---|---|---|---|
| p1 `1.2` · rstp-plus, lacp, vpc | Separate loop prevention, LACP agreement and dual-switch consistency when a GPU server loses one uplink. | Compare bridge roles, partner identities, bond members, peer health and traffic on the surviving physical path. | Cumulus MLAG transfers the redundancy investigation; it does not implement Cisco vPC or every RSTP+ detail. Core: U05. |
| p1 `1.3` · vxlan, evpn | Trace an isolated GPU tenant from local attachment through VNI, route target and remote next hop. | Correlate endpoint learning, EVPN routes, encapsulated packets and both directions of the workload path. | A routed Ethernet lab is not a VXLAN/EVPN implementation; underlay success alone is insufficient. Core: U05. |

## D04 · Trace tenant intent into forwarding state

Planned chain: DC04 → DP04 → DE04a, DE04b.

| Source locator / concepts | GPU-datacenter investigation | Required observations | Boundary and NVIDIA core |
|---|---|---|---|
| p1 `1.4.a` · discovery | Explain why a physically present switch is absent from intended fabric membership. | Reconcile NetBox serial and port identity with discovery, admission state and the affected job path. | NetBox is intent, not an ACI discovery controller; preserve controller-specific admission as a separate gate. Core: U01. |
| p1 `1.4.b` · access-policy | Localize a tenant attachment failure to port selection, VLAN allowance or policy application. | Join endpoint, port, bridge and policy identities; prove permitted traffic succeeds while a neighboring tenant stays denied. | A generic policy mapping does not reproduce ACI access-policy object semantics. Core: U05. |
| p1 `1.4.c` · vmm-integration | Investigate disagreement between virtual workload placement and physical network attachment. | Join workload UID, virtual interface, host uplink and tenant segment; distinguish controller state from actual payload. | Rusternetes/KWOK object transitions do not execute a VMM integration or virtual machine. Core: U09. |
| p1 `1.4.d` · tenant-policy | Find the first policy boundary that changes an intended tenant relationship. | Retain intended producer/consumer identities, applied rules, allowed flow and forbidden companion flow. | VRF and ACL exercises preserve isolation reasoning without claiming ACI tenant-contract equivalence. Core: U05. |
| p1 `1.4.e` · unicast, multicast, broadcast | Separate unicast lookup, multicast replication and broadcast flooding along the same tenant path. | Observe destination classification and each expected egress independently with bounded captures. | One successful unicast ping cannot certify multicast or broadcast behavior. Core: U05. |
| p1 `1.4.f` · external-connectivity | Diagnose a cluster that is reachable internally but cannot reach its external data or artifact service. | Compare boundary routes, import/export policy, return path and authenticated application transfer. | An external connectivity adaptation does not reproduce ACI L2Out/L3Out object behavior. Core: U19. |

## D05 · Reconcile server, boot and storage profiles

Planned chain: DC05 → DP05 → DE05a, DE05b.

| Source locator / concepts | GPU-datacenter investigation | Required observations | Boundary and NVIDIA core |
|---|---|---|---|
| p1 `2.2.b` · vlan, pool, policy, template | Find why a templated host receives the wrong network identity or allowed segment. | Diff pool allocation, template revision, realized port attachment and the resulting GPU job reachability. | NVIDIA/Linux host profiles are not Cisco UCS service profiles; preserve that distinction in evidence. Core: U01. |
| p1 `2.2.c` · san-connectivity, fc-zoning, vsan, pool, policy, template | Trace a storage attachment mismatch through transport, isolation, allocation and template intent. | Correlate initiator identity, fabric membership, target authorization and a checksummed dataset read. | FC zoning and VSAN behavior require a real SAN adapter; a TCP volume is not a substitute. Core: U13. |
| p1 `2.2.d` · server-pool, boot-policy | Diagnose a host that joins the wrong allocation pool or boots an incompatible payload. | Bind boot source, artifact digest, pool ownership and observed operating system to one immutable host identity. | DeepOps hosts reconciliation alone does not qualify network boot or firmware boot policy. Core: U03. |

## D06 · Localize storage transport stalls

Planned chain: DC06 → DP06 → DE06a, DE06b.

| Source locator / concepts | GPU-datacenter investigation | Required observations | Boundary and NVIDIA core |
|---|---|---|---|
| p2 `3.1.a` · initialization | Locate the first storage fabric membership transition that did not complete. | Collect ordered link, login and fabric membership events before the dataset read is attempted. | FC initialization is neither Ethernet link-up nor InfiniBand subnet management. Core: U13. |
| p2 `3.1.b` · credit-starvation | Distinguish storage receiver-credit exhaustion from packet loss or slow media. | Correlate both-end credit counters, link distance and I/O completion latency during a controlled load. | FC buffer credits, Ethernet PFC and NVMe queue depth are different mechanisms; do not relabel one as another. Core: U13. |
| p2 `3.1.c` · npv, npiv | Trace virtual initiator identity through a shared physical storage uplink. | Bind virtual identity, physical port, fabric login and target access to the affected dataset. | NPV/NPIV requires an FC-capable environment; virtual NIC aliases cannot satisfy it. Core: U13. |
| p2 `3.1.d` · vsan | Explain why a storage endpoint is in the wrong logical fabric. | Compare intended and observed fabric membership with denied cross-tenant and permitted local I/O. | A VRF or InfiniBand partition is a comparison, not a VSAN implementation. Core: U13. |

## D07 · Repair storage identity without widening access

Planned chain: DC07 → DP07 → DE07a, DE07b.

| Source locator / concepts | GPU-datacenter investigation | Required observations | Boundary and NVIDIA core |
|---|---|---|---|
| p2 `3.2.a` · fcid | Distinguish persistent initiator identity from a fabric-assigned storage address. | Correlate login history, assigned address and target lookup after reattachment. | FCID assignment must be observed on a qualified FC fabric. Core: U13. |
| p2 `3.2.b` · zoning | Repair initiator-to-target authorization without opening another tenant path. | Show the intended pair, effective policy and positive plus negative dataset access trials. | File permissions alone do not demonstrate FC zoning enforcement. Core: U13. |
| p2 `3.2.c` · device-alias | Detect a friendly storage name that resolves to the wrong immutable endpoint. | Join alias, immutable identity, effective policy and checksum-valid application data. | Keep vendor alias semantics separate from generic naming examples. Core: U13. |
| p2 `3.2.d` · cfs | Identify disagreement between distributed copies of storage-fabric configuration. | Compare authoritative revision, replication outcome, conflict ownership and post-recovery I/O. | A Git/NetBox reconciliation example does not implement Cisco Fabric Services. Core: U20. |

## D08 · Debug event-driven reconciliation

Planned chain: DC08 → DP08 → DE08a, DE08b.

| Source locator / concepts | GPU-datacenter investigation | Required observations | Boundary and NVIDIA core |
|---|---|---|---|
| p2 `4.1.a` · eem | Diagnose a recovery automation event that is missing, repeated or associated with the wrong host. | Bind event identity and time to an idempotent guarded change plus an independent workload observation. | A native event adapter teaches event reasoning without claiming Cisco EEM execution. Core: U16. |
| p2 `4.1.b` · scheduler | Separate automation timer overlap from GPU workload scheduler admission. | Retain timer ownership, last run, exit state, retry record and the effect on a named job. | A maintenance timer is not a Slurm or Kubernetes scheduler. Core: U10. |
| p2 `4.2.a` · bash, guest-shell | Find a native automation process failing because its environment or authority differs from the operator context. | Capture executable identity, arguments, environment allowlist, UID, output and exit status through a code API. | Incus is the execution boundary; this does not qualify NX-OS guest-shell behavior. Core: U03. |
| p2 `4.2.b` · rest | Diagnose a partial API reconciliation without retrying an unsafe write blindly. | Keep bounded request identity, authorization, status, pagination and observed postcondition. | Transport success alone is not successful configuration or workload execution. Core: U01. |
| p2 `4.2.c` · json, xml | Reject an intent document that changes meaning during decoding or serialization. | Exercise duplicate keys, missing values, types, namespaces and a round-trip comparison against the reviewed schema. | JSON validation does not cover XML; qualify each format that the selected product exposes. Core: U01. |
| p2 `4.2.d` · python | Localize an automation defect to input normalization, transformation or an external effect. | Retain typed input, expanded plan, exception boundary and a controlled replay without duplicate effects. | A planning example is not evidence that the external Python-controlled service ran. Core: U03. |
| p2 `4.2.e` · ansible | Diagnose a real DeepOps reconciliation whose second application still changes hosts. | Compare pinned role source, inventory, generated variables, task outcomes and second-run changes. | Only actually selected upstream roles are qualified; hosts reconciliation is not full DeepOps deployment. Core: U01. |

## D09 · Recover fleet automation from partial application

Planned chain: DC09 → DP09 → DE09a, DE09b.

| Source locator / concepts | GPU-datacenter investigation | Required observations | Boundary and NVIDIA core |
|---|---|---|---|
| p2 `4.2.f` · terraform | Repair desired/observed/state disagreement after an interrupted infrastructure change. | Review refresh and proposed changes, ownership conflicts, provider behavior and rollback against the original intent. | Pulumi may teach the workflow; Terraform state and provider semantics need explicit separate practice. Core: U01. |
| p2 `4.2.g` · intersight | Trace a fleet operation from a management request to the actual GPU host effect. | Correlate target identity, task revision, permission, execution status and independent host evidence. | Mission Control or another fleet manager does not implement Cisco Intersight. Core: U16. |
| p2 `5.2` · nexus-dashboard, intersight | Locate the boundary at which centralized network or compute management loses accurate state. | Correlate collector identity, credentials, timestamps, intended revision and actual service behavior. | NetQ and Mission Control are adaptations; their behavior is not Nexus Dashboard or Intersight equivalence. Core: U16. |

## D10 · Stage and reverse a firmware change

Planned chain: DC10 → DP10 → DE10a, DE10b.

| Source locator / concepts | GPU-datacenter investigation | Required observations | Boundary and NVIDIA core |
|---|---|---|---|
| p2 `2.5` · firmware, package, interoperability | Stage a component upgrade only after its dependency tuple and rollback image are known. | Bind package signatures, installed versions, drained jobs, failover result and rollback outcome. | Real device update and interruption behavior require the relevant hardware adapter. Core: U17. |
| p2 `5.1` · firmware, package, interoperability | Diagnose a fleet-wide update inconsistency separately from a single-device upgrade. | Compare package provenance, cohorts, compatibility constraints and recovery for both changed and unchanged hosts. | This distinct operations obligation is retained even though source 2.5 also concerns updates. Core: U17. |

## D11 · Recover a fabric while preserving isolation

Planned chain: DC11 → DP11 → DE11a, DE11b.

| Source locator / concepts | GPU-datacenter investigation | Required observations | Boundary and NVIDIA core |
|---|---|---|---|
| p2 `5.3.a` · fabric-binding, port-security | Detect an unauthorized endpoint attached to an otherwise healthy data path. | Bind intended port and endpoint identity, admission decision and denied unauthorized traffic. | Switch access control is distinct from FC fabric binding; qualify both where used. Core: U05. |
| p2 `5.3.b` · aaa, rbac | Restore operator access while keeping a lesser role unable to mutate network intent. | Observe authentication and authorization separately with positive and negative principals. | A trusted TLS client is not automatically an authorized API administrator. Core: U04. |
| p2 `5.3.c` · arp-inspection, dhcp-snooping, port-security | Locate a forged first-hop binding that redirects a CPU dependency of a GPU workload. | Correlate lease, source binding, neighbor cache and enforcement on the actual ingress port. | Ordinary Linux forwarding does not provide switch DHCP-snooping or inspection semantics. Core: U05. |
| p2 `5.3.d` · macsec | Distinguish link-key disagreement from reachability or application TLS failure. | Observe secure association, replay protection, key rotation and permitted payload on the selected link. | TLS and an unencrypted veth do not implement MACsec. Core: U05. |
| p2 `5.4` · security-domain, role-mapping | Find a role-to-tenant mapping that grants authority in the wrong administrative domain. | Test the same principal against permitted and forbidden scopes after the mapping change. | Generic RBAC teaches scope reasoning but does not implement ACI security domains. Core: U04. |

## D12 · Restore compute and storage authority together

Planned chain: DC12 → DP12 → DE12a, DE12b.

| Source locator / concepts | GPU-datacenter investigation | Required observations | Boundary and NVIDIA core |
|---|---|---|---|
| p3 `5.5.a` · aaa, rbac | Recover host and scheduler access without granting unrestricted execution to a tenant. | Join OS identity, scheduler account and effective permission with positive and negative job operations. | A synthetic Ready node does not prove host or scheduler security enforcement. Core: U04. |
| p3 `5.5.b` · key-management | Rotate a compute credential without breaking retrieval or leaking the old key. | Bind issuer, consumer, validity interval, revocation and authenticated artifact retrieval. | A secret placeholder is not key lifecycle evidence. Core: U03. |
| p3 `5.6.a` · aaa, rbac | Restore authorized dataset access while another tenant remains denied. | Correlate storage principal, effective role, target and content-verified read/write outcomes. | Storage authorization must be observed at its enforcement point, not inferred from API login. Core: U13. |
| p3 `5.6.b` · port-security | Reject a storage endpoint connected through an unauthorized port. | Bind initiator, physical attachment, effective restriction and failed unauthorized I/O. | A directory permission cannot replace a storage port-security observation. Core: U13. |
| p3 `5.6.c` · fabric-binding | Recover storage-fabric membership without admitting an untrusted participant. | Observe fabric identity and admission state, then test intended and forbidden storage paths. | Qualified SAN evidence is required for fabric binding; graph membership alone is insufficient. Core: U13. |
