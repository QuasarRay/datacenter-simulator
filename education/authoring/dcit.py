"""One declaration per source leaf; compile adaptation obligations, never grades.

NCP-DCIT is this project's name, not an NVIDIA or Cisco certification. Source
identities remain distinct even when two source leaves describe related work.
"""
from dataclasses import dataclass


@dataclass(frozen=True)
class Adaptation:
    locator: str
    unit: int
    core: str
    investigation: str
    evidence: str
    boundary: str


ROWS = []


def adapt(locator, unit, core, investigation, evidence, boundary):
    """Expand a small author declaration into a source-bound obligation."""
    if any(row.locator == locator for row in ROWS):
        raise ValueError('duplicate DCIT declaration: ' + locator)
    ROWS.append(Adaptation(locator, unit, core, investigation, evidence, boundary))


UNITS = [
    'Establish rack and adapter identity',
    'Localize a broken host-to-fabric path',
    'Separate link redundancy from overlay reachability',
    'Trace tenant intent into forwarding state',
    'Reconcile server, boot and storage profiles',
    'Localize storage transport stalls',
    'Repair storage identity without widening access',
    'Debug event-driven reconciliation',
    'Recover fleet automation from partial application',
    'Stage and reverse a firmware change',
    'Recover a fabric while preserving isolation',
    'Restore compute and storage authority together',
]

# Each row states a falsifiable investigation, the observation that would settle
# it, and the behavior the present native platform must not impersonate.
adapt('1.1.a', 2, 'U05', 'Locate IPv4 and IPv6 adjacency, area, MTU or route-installation disagreement along a GPU service path.', 'Compare intended peers with both neighbor databases, selected routes and a source-bound payload in each address family.', 'Static routes are not an OSPF implementation; both protocol versions need an explicit routing adapter.')
adapt('1.1.b', 2, 'U05', 'Separate BGP session health, address-family exchange, policy rejection and forwarding installation.', 'Retain received and selected prefixes, next-hop resolution, return path and application transfer before and after repair.', 'A graph path does not prove BGP convergence; real FRR or qualified switch observations are required.')
adapt('1.1.c', 2, 'U05', 'Diagnose multicast receiver registration, reverse-path selection and tree state in surrounding CPU services.', 'Trace one identified source and group through membership, control state and captured delivery.', 'Do not assume the GPU collective uses IP multicast; identify its actual transport first.')
adapt('1.1.d', 2, 'U20', 'Distinguish gateway ownership loss from duplicate ownership during a CPU service failover.', 'Observe gateway identity, neighbor resolution, interruption interval and post-failover bidirectional traffic.', 'An anycast gateway and a first-hop election protocol have different failure models; qualify the selected mechanism.')
adapt('1.2', 3, 'U05', 'Separate loop prevention, LACP agreement and dual-switch consistency when a GPU server loses one uplink.', 'Compare bridge roles, partner identities, bond members, peer health and traffic on the surviving physical path.', 'Cumulus MLAG transfers the redundancy investigation; it does not implement Cisco vPC or every RSTP+ detail.')
adapt('1.3', 3, 'U05', 'Trace an isolated GPU tenant from local attachment through VNI, route target and remote next hop.', 'Correlate endpoint learning, EVPN routes, encapsulated packets and both directions of the workload path.', 'A routed Ethernet lab is not a VXLAN/EVPN implementation; underlay success alone is insufficient.')
adapt('1.4.a', 4, 'U01', 'Explain why a physically present switch is absent from intended fabric membership.', 'Reconcile NetBox serial and port identity with discovery, admission state and the affected job path.', 'NetBox is intent, not an ACI discovery controller; preserve controller-specific admission as a separate gate.')
adapt('1.4.b', 4, 'U05', 'Localize a tenant attachment failure to port selection, VLAN allowance or policy application.', 'Join endpoint, port, bridge and policy identities; prove permitted traffic succeeds while a neighboring tenant stays denied.', 'A generic policy mapping does not reproduce ACI access-policy object semantics.')
adapt('1.4.c', 4, 'U09', 'Investigate disagreement between virtual workload placement and physical network attachment.', 'Join workload UID, virtual interface, host uplink and tenant segment; distinguish controller state from actual payload.', 'Rusternetes/KWOK object transitions do not execute a VMM integration or virtual machine.')
adapt('1.4.d', 4, 'U05', 'Find the first policy boundary that changes an intended tenant relationship.', 'Retain intended producer/consumer identities, applied rules, allowed flow and forbidden companion flow.', 'VRF and ACL exercises preserve isolation reasoning without claiming ACI tenant-contract equivalence.')
adapt('1.4.e', 4, 'U05', 'Separate unicast lookup, multicast replication and broadcast flooding along the same tenant path.', 'Observe destination classification and each expected egress independently with bounded captures.', 'One successful unicast ping cannot certify multicast or broadcast behavior.')
adapt('1.4.f', 4, 'U19', 'Diagnose a cluster that is reachable internally but cannot reach its external data or artifact service.', 'Compare boundary routes, import/export policy, return path and authenticated application transfer.', 'An external connectivity adaptation does not reproduce ACI L2Out/L3Out object behavior.')
adapt('2.1', 1, 'U02', 'Establish the physical GPU server identity before replacing or reprovisioning anything.', 'Correlate rack position, serial, BMC inventory, power state, adapter presence and a named affected workload.', 'An Incus guest cannot certify a physical rack server or Cisco UCS hardware.')
adapt('2.2.a', 1, 'U02', 'Localize a shared rack failure across chassis, power feed and I/O paths.', 'Retain independent power and module observations plus the affected-host set before choosing a maintenance boundary.', 'System containers do not simulate electrical redundancy or chassis module service procedures.')
adapt('2.2.b', 5, 'U01', 'Find why a templated host receives the wrong network identity or allowed segment.', 'Diff pool allocation, template revision, realized port attachment and the resulting GPU job reachability.', 'NVIDIA/Linux host profiles are not Cisco UCS service profiles; preserve that distinction in evidence.')
adapt('2.2.c', 5, 'U13', 'Trace a storage attachment mismatch through transport, isolation, allocation and template intent.', 'Correlate initiator identity, fabric membership, target authorization and a checksummed dataset read.', 'FC zoning and VSAN behavior require a real SAN adapter; a TCP volume is not a substitute.')
adapt('2.2.d', 5, 'U03', 'Diagnose a host that joins the wrong allocation pool or boots an incompatible payload.', 'Bind boot source, artifact digest, pool ownership and observed operating system to one immutable host identity.', 'DeepOps hosts reconciliation alone does not qualify network boot or firmware boot policy.')
adapt('2.3', 2, 'U16', 'Trace one CPU-side data transfer required by a GPU job from the named source interface to its storage peer.', 'Bind NetBox endpoints, guest routes, packet counters, exact payload and cut/restore evidence to one attempt.', 'Linux Ethernet evidence does not establish GPU execution, switch ASIC forwarding or RDMA.')
adapt('2.4.a', 1, 'U08', 'Check the adapter identity and host driver agreement before blaming the fabric.', 'Correlate PCI function, firmware, driver, link mode and the bound data path.', 'ConnectX or BlueField inspection is an adaptation, not a Cisco VIC emulation.')
adapt('2.4.b', 1, 'U17', 'Determine whether a component firmware revision belongs to the supported compatibility tuple.', 'Record actual model, revision, driver and release-matrix decision with a reproducible pre-change workload.', 'A source pin is not a hardware compatibility certificate.')
adapt('2.4.c', 1, 'U02', 'Locate an I/O module mismatch that disconnects several hosts together.', 'Correlate slot, lane mapping, module revision, physical links and common affected endpoints.', 'Virtual NIC presence cannot certify chassis I/O module interoperability.')
adapt('2.4.d', 1, 'U02', 'Separate a server uplink fault from a shared fabric interconnect fault.', 'Compare both sides of each connection and the blast radius across independent hosts.', 'An Ethernet switch-role guest does not implement UCS fabric interconnects.')
adapt('2.5', 10, 'U17', 'Stage a component upgrade only after its dependency tuple and rollback image are known.', 'Bind package signatures, installed versions, drained jobs, failover result and rollback outcome.', 'Real device update and interruption behavior require the relevant hardware adapter.')
adapt('3.1.a', 6, 'U13', 'Locate the first storage fabric membership transition that did not complete.', 'Collect ordered link, login and fabric membership events before the dataset read is attempted.', 'FC initialization is neither Ethernet link-up nor InfiniBand subnet management.')
adapt('3.1.b', 6, 'U13', 'Distinguish storage receiver-credit exhaustion from packet loss or slow media.', 'Correlate both-end credit counters, link distance and I/O completion latency during a controlled load.', 'FC buffer credits, Ethernet PFC and NVMe queue depth are different mechanisms; do not relabel one as another.')
adapt('3.1.c', 6, 'U13', 'Trace virtual initiator identity through a shared physical storage uplink.', 'Bind virtual identity, physical port, fabric login and target access to the affected dataset.', 'NPV/NPIV requires an FC-capable environment; virtual NIC aliases cannot satisfy it.')
adapt('3.1.d', 6, 'U13', 'Explain why a storage endpoint is in the wrong logical fabric.', 'Compare intended and observed fabric membership with denied cross-tenant and permitted local I/O.', 'A VRF or InfiniBand partition is a comparison, not a VSAN implementation.')
adapt('3.2.a', 7, 'U13', 'Distinguish persistent initiator identity from a fabric-assigned storage address.', 'Correlate login history, assigned address and target lookup after reattachment.', 'FCID assignment must be observed on a qualified FC fabric.')
adapt('3.2.b', 7, 'U13', 'Repair initiator-to-target authorization without opening another tenant path.', 'Show the intended pair, effective policy and positive plus negative dataset access trials.', 'File permissions alone do not demonstrate FC zoning enforcement.')
adapt('3.2.c', 7, 'U13', 'Detect a friendly storage name that resolves to the wrong immutable endpoint.', 'Join alias, immutable identity, effective policy and checksum-valid application data.', 'Keep vendor alias semantics separate from generic naming examples.')
adapt('3.2.d', 7, 'U20', 'Identify disagreement between distributed copies of storage-fabric configuration.', 'Compare authoritative revision, replication outcome, conflict ownership and post-recovery I/O.', 'A Git/NetBox reconciliation example does not implement Cisco Fabric Services.')
adapt('4.1.a', 8, 'U16', 'Diagnose a recovery automation event that is missing, repeated or associated with the wrong host.', 'Bind event identity and time to an idempotent guarded change plus an independent workload observation.', 'A native event adapter teaches event reasoning without claiming Cisco EEM execution.')
adapt('4.1.b', 8, 'U10', 'Separate automation timer overlap from GPU workload scheduler admission.', 'Retain timer ownership, last run, exit state, retry record and the effect on a named job.', 'A maintenance timer is not a Slurm or Kubernetes scheduler.')
adapt('4.2.a', 8, 'U03', 'Find a native automation process failing because its environment or authority differs from the operator context.', 'Capture executable identity, arguments, environment allowlist, UID, output and exit status through a code API.', 'Incus is the execution boundary; this does not qualify NX-OS guest-shell behavior.')
adapt('4.2.b', 8, 'U01', 'Diagnose a partial API reconciliation without retrying an unsafe write blindly.', 'Keep bounded request identity, authorization, status, pagination and observed postcondition.', 'Transport success alone is not successful configuration or workload execution.')
adapt('4.2.c', 8, 'U01', 'Reject an intent document that changes meaning during decoding or serialization.', 'Exercise duplicate keys, missing values, types, namespaces and a round-trip comparison against the reviewed schema.', 'JSON validation does not cover XML; qualify each format that the selected product exposes.')
adapt('4.2.d', 8, 'U03', 'Localize an automation defect to input normalization, transformation or an external effect.', 'Retain typed input, expanded plan, exception boundary and a controlled replay without duplicate effects.', 'A planning example is not evidence that the external Python-controlled service ran.')
adapt('4.2.e', 8, 'U01', 'Diagnose a real DeepOps reconciliation whose second application still changes hosts.', 'Compare pinned role source, inventory, generated variables, task outcomes and second-run changes.', 'Only actually selected upstream roles are qualified; hosts reconciliation is not full DeepOps deployment.')
adapt('4.2.f', 9, 'U01', 'Repair desired/observed/state disagreement after an interrupted infrastructure change.', 'Review refresh and proposed changes, ownership conflicts, provider behavior and rollback against the original intent.', 'Pulumi may teach the workflow; Terraform state and provider semantics need explicit separate practice.')
adapt('4.2.g', 9, 'U16', 'Trace a fleet operation from a management request to the actual GPU host effect.', 'Correlate target identity, task revision, permission, execution status and independent host evidence.', 'Mission Control or another fleet manager does not implement Cisco Intersight.')
adapt('5.1', 10, 'U17', 'Diagnose a fleet-wide update inconsistency separately from a single-device upgrade.', 'Compare package provenance, cohorts, compatibility constraints and recovery for both changed and unchanged hosts.', 'This distinct operations obligation is retained even though source 2.5 also concerns updates.')
adapt('5.2', 9, 'U16', 'Locate the boundary at which centralized network or compute management loses accurate state.', 'Correlate collector identity, credentials, timestamps, intended revision and actual service behavior.', 'NetQ and Mission Control are adaptations; their behavior is not Nexus Dashboard or Intersight equivalence.')
adapt('5.3.a', 11, 'U05', 'Detect an unauthorized endpoint attached to an otherwise healthy data path.', 'Bind intended port and endpoint identity, admission decision and denied unauthorized traffic.', 'Switch access control is distinct from FC fabric binding; qualify both where used.')
adapt('5.3.b', 11, 'U04', 'Restore operator access while keeping a lesser role unable to mutate network intent.', 'Observe authentication and authorization separately with positive and negative principals.', 'A trusted TLS client is not automatically an authorized API administrator.')
adapt('5.3.c', 11, 'U05', 'Locate a forged first-hop binding that redirects a CPU dependency of a GPU workload.', 'Correlate lease, source binding, neighbor cache and enforcement on the actual ingress port.', 'Ordinary Linux forwarding does not provide switch DHCP-snooping or inspection semantics.')
adapt('5.3.d', 11, 'U05', 'Distinguish link-key disagreement from reachability or application TLS failure.', 'Observe secure association, replay protection, key rotation and permitted payload on the selected link.', 'TLS and an unencrypted veth do not implement MACsec.')
adapt('5.4', 11, 'U04', 'Find a role-to-tenant mapping that grants authority in the wrong administrative domain.', 'Test the same principal against permitted and forbidden scopes after the mapping change.', 'Generic RBAC teaches scope reasoning but does not implement ACI security domains.')
adapt('5.5.a', 12, 'U04', 'Recover host and scheduler access without granting unrestricted execution to a tenant.', 'Join OS identity, scheduler account and effective permission with positive and negative job operations.', 'A synthetic Ready node does not prove host or scheduler security enforcement.')
adapt('5.5.b', 12, 'U03', 'Rotate a compute credential without breaking retrieval or leaking the old key.', 'Bind issuer, consumer, validity interval, revocation and authenticated artifact retrieval.', 'A secret placeholder is not key lifecycle evidence.')
adapt('5.6.a', 12, 'U13', 'Restore authorized dataset access while another tenant remains denied.', 'Correlate storage principal, effective role, target and content-verified read/write outcomes.', 'Storage authorization must be observed at its enforcement point, not inferred from API login.')
adapt('5.6.b', 12, 'U13', 'Reject a storage endpoint connected through an unauthorized port.', 'Bind initiator, physical attachment, effective restriction and failed unauthorized I/O.', 'A directory permission cannot replace a storage port-security observation.')
adapt('5.6.c', 12, 'U13', 'Recover storage-fabric membership without admitting an untrusted participant.', 'Observe fabric identity and admission state, then test intended and forbidden storage paths.', 'Qualified SAN evidence is required for fabric binding; graph membership alone is insufficient.')
