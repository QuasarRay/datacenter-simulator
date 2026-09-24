# Native Rusternetes control-plane profile

The pinned upstream API server also contains runtime clients, a synthetic log
fallback and a declared client-CA option that its binary entry point does not
apply. `apply_profile.py` makes those differences explicit before building.
It accepts only the reviewed clean revision, checks every replacement boundary,
then records hashes of the derived files. Binding and eviction logic remain.

The derived API server authorizes log/exec/attach/port-forward requests, verifies
that the pod exists, and returns a Kubernetes Status failure with HTTP 501.
It never connects to an application-container daemon or invents process output.
Native payloads execute separately in owned Incus guests. KWOK changes API state;
it does not execute those payloads. This is a deliberate capability boundary,
not Kubernetes conformance or NVIDIA Operator compatibility.

The binary's client-CA option now configures the TLS verifier. A client must also
supply an independently authorized bearer token: certificate CN is not mapped
to an RBAC principal by this profile. A certificate alone grants no access.
The service asset must carry `ncp-profile.json` and exact binary hashes. Build only
the three pinned control-plane packages. The build gate inspects their production
dependency graph for the removed runtime client; it builds no upstream kubelet.

Remaining upstream synthetic behavior, including metrics and hostpath storage,
is not qualified evidence. Do not grade HPA, storage delivery, execution or GPU
Operator health from this API's status alone. Every station needs an independent
collector for the actual subsystem and a negative control.
