> Implementation authorized by the project owner on 2026-09-24. The former implementation prohibition is withdrawn. See integrations/incus for implemented behavior and validation boundaries.

# Attach Incus nodes to patchbay without duplicate namespaces

## What the pinned implementation actually provides

`simulator/src/linux.rs` builds `Lab::new()`, per-wire `RouterPreset::PublicV4` routers and per-node `Device` namespaces, then disables each router's `ix` interface. It uses `Device::run_sync`, per-interface `tc`, and generated routes. The pinned patchbay internals in `patchbay/src/{lab,device,netns,wiring,netlink,qdisc}.rs` create and own these namespaces. There is no verified public API in this checkout that simply attaches an Incus instance.

The future adapter therefore requires a **patchbay fork change**. Do not claim that `Lab::add_device` represents an existing container or invent a working `attach_incus` method in examples. `LabBuilder::allow_real_root` is not itself an external-namespace integration API. Calling `init_userns` in the Incus orchestration process can remove the authority needed to manipulate Incus-owned networking.

## Chosen attachment design

Use one actual network namespace per Incus node. Incus creates one `p2p` NIC for each modeled Ethernet port. Its guest end stays inside the container and its host peer stays in the host initial network namespace. A new patchbay host-attachment layer connects each edge's two host peers through one private bridge, with no L3 address, uplink, DHCP or shared IX. It also borrows verified node netns handles for scoped observation or shaping where needed.

Illustrative endpoint request, using qualified names from a future plan:

```bash
incus --project dc-lab config device add leaf1 data1 nic \
  nictype=p2p name=data1 host_name=dc_a1e1 mtu=1500
```

This only creates a NIC. It does not build a wire, configure FRR, add an address, apply impairments or prove traffic follows a graph. The future patchbay adapter must do the rest. Keep generated kernel names within Linux's 15-byte limit and persist an injective mapping; truncated display names are not enough.

For an edge A:data1—B:data2, the plan records both Incus device names, host peer names/ifindices, namespace generation, bridge identity, MACs and direction-specific conditions. The bridge has exactly those two ports. Configure isolation and link-local behavior explicitly. Inspect the bridge FDB/VLAN membership; never let NetworkManager or an incidental host DHCP daemon claim these devices. Because the host can observe/forward traffic, test host firewall interaction and IPv6 as well as guest routes.

A two-port Linux bridge is not automatically a transparent wire for every Ethernet control frame. Its link-local forwarding mask and kernel restrictions can affect LLDP, STP or LACP traffic. For each supported protocol, qualify bridge/per-port behavior with packet captures at both guests; configure only the permitted required forwarding behavior or reject that protocol/profile. Do not claim full L2 fidelity from successful IP pings. Record multicast snooping, bridge netfilter, VLAN filtering and offload/GRO/GSO effects that can change captures or timing. See the [kernel bridge reference](https://cdn.kernel.org/doc/html/latest/networking/bridge.html).

## Patchbay fork work, in dependency order

1. Introduce resource ownership types for `OwnedNamespace`, `BorrowedNamespace`, `OwnedBridge` and `BorrowedInterface` (proposed names). Borrowed cleanup releases worker/FD references only; it must not kill container init, delete Incus veths or remove Incus namespaces.
2. Create a host-attachment mode that does not initialize a new user namespace or default internet exchange. Keep existing namespace-only tests isolated. Avoid assuming that the existing `Lab` root namespace is the real host namespace.
3. Register an external endpoint using an authenticated Incus observation: project, instance, lifecycle generation, init PID/start identity, netns handle/inode and expected guest interface/MAC. Hold the FD while operating and invalidate it on stop/restart. Never trust a cached PID alone.
4. Add controlled wiring operations that inspect an Incus-created host peer, create/own a private bridge, and enslave only that peer. Keep the peer visible to Incus for its own cleanup. Refuse a peer already attached to a foreign master, conflicting IP address or wrong namespace.
5. Add scoped qdisc/rule/route inspection and mutation. Every operation must return errors and acknowledgements; no best-effort silent success. Keep namespace entry on dedicated workers; do not `setns` a general Tokio executor thread. Prefer audited safe wrappers or the privileged helper boundary over new ad hoc unsafe code.
6. Extend event/evidence output with owner/generation/ifindex and rollback results. Update fork tests and gitlink/upstreams pins together only after attachment lifecycle tests pass.

A privileged helper is a narrow capability boundary, not an arbitrary root shell. Validate requests against an approved plan, enforce path/name limits and exact ownership, and expose only needed netlink operations. Keep the unprivileged planner separate. User namespace nesting cannot confer authority over resources owned by a more privileged namespace.

## Graph and routing rules

Retain directed edges and explicit link IDs so parallel cables remain distinct. The portable routing graph uses latency plus hop-count tie-breaking; it is not BGP policy or ECMP. Validate endpoints, no duplicate port assignment, medium, VLAN/VRF scope, address overlap, MTU and queue bounds before allocation. Keep disconnected nodes meaningful instead of auto-adding an uplink.

For **static model routing**, derive next hops from the validated graph and install only owned routes. End hosts do not forward. Record source-interface addresses and return paths. For **FRR network training**, install connected addressing and daemon configuration, then let FRR own dynamic routing. Use graph reachability as an expectation, not a way to inject a route that hides a missing BGP session. Validate RIB and installed kernel FIB independently.

L2 switching, VLAN trunking, bonds/LAGs, EVPN/VXLAN, VRFs and IPv6 require explicit model/schema/backend capability work. The current NetBox importer rejects several of these; do not silently flatten them into point-to-point edges. VXLAN belongs inside switch namespaces with correct underlay/MTU/VNI/FDB behavior and multicast/unicast assumptions documented. Model support, configuration support and measured dataplane support are separate acceptance claims.

## Shaping and fault direction

Apply each one-way impairment once. With host `p2p` peers, host-peer egress is traffic **toward its guest**. To shape A→B, apply the selected queue at B's host peer egress, or A's guest egress, but not both. Document the choice and account for reverse-direction ACK/control traffic. Retain the current MTU/queue-bound checks and preserve sub-millisecond units; patchbay's integer-millisecond high-level setting is insufficient for the existing nanosecond intent.

Latency, rate, loss, jitter and reordering are distinct capabilities. Do not silently accept fields not implemented by the pinned backend. Linux qdisc wall-clock behavior is not calibrated datacenter timing, and a queue limit is not GPU switch buffer/PFC modeling. Incus rate limiting must be disabled on links shaped by patchbay.

A link-down transaction updates both endpoint state and the graph only after kernel success. Save previous state, restore both directions/routes on failure, and poison the run if restoration fails. For FRR, a cable cut must cause genuine convergence or failure; a graph-only toggle does not count. Distinguish administrative-down, carrier, one-way drop and route-policy failure in evidence.

## Prove absence of shortcuts

For each accepted topology, collect addresses/routes/rules, bridge membership, qdiscs, counters and representative packet captures. Generate a flow bound to the intended data source. Cut the only path and require failure while management SSH remains usable. In redundant topologies, cut one path, verify use of the alternate exact edge, then cut all paths and require failure. Repeat with IPv6 and with the host's normal internet/VPN active.

Test start/stop/restart, hotplug, daemon restart, interface-name collision, netns/PID reuse, concurrent labs with identical guest names, failed qdisc change, canceled deployment, and teardown with a foreign interface accidentally attached. Never remove foreign interfaces to make cleanup pass.

Sources: [pinned patchbay](https://github.com/QuasarRay/patchbay/tree/3d3c577c4b49ba23cb6b14f6b48aacbec933a93e), [Incus NIC reference](https://linuxcontainers.org/incus/docs/main/reference/devices_nic/), and baseline `simulator/src/{linux,topology}.rs`. This entire host-attachment design still requires implementation and live qualification.
