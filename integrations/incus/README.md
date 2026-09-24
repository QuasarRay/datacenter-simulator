# Incus system-container backend

The Rust `incus` feature compiles the existing simulator manifest into explicit
Incus REST requests and patchbay host cables. No image alias, default profile,
implicit uplink, privileged guest or nested runtime is inherited. The example
uses five CachyOS system containers with 6 GiB of declared RAM; this is a budget,
not a measured host memory claim. KWOK workers are separate logical resources.

Import a CachyOS system-container image into an initialized local Incus daemon,
record its full fingerprint, then replace the deliberately invalid value in
`config.example.json`. The image must include a working init, Python, iproute2
and the packages required by the selected Ansible roles. Image building and
kernel qualification remain deployment prerequisites. A stock Arch image must
not be relabeled as validated CachyOS.

The student-facing interface is Rust configuration and Ansible. The operator can
also use `incus-plan CONFIG`, `incus-up CONFIG NEW_STATE_DIR` and
`incus-down STATE_DIR` in the simulator binary built with `--features incus`.
The same operations are public Rust functions. `curl`, `ip`, `tc` and `timeout`
are internal process dependencies, invoked with argument arrays.

Lifecycle changes require Incus socket access and host network privileges. They
are serialized with `/run/lock/ncp-incus-network.lock`. A crash leaves a lock and
an fsynced journal; inspect the named resources and operation state before
removing a stale lock. Never run student code with those capabilities. A failed
deployment retains evidence and does not recursively delete a project. Cleanup
checks the project owner, every instance owner and the exact bridge membership.
If resources have changed identity, cleanup stops for reconciliation.

Incus owns containers and p2p veths; patchbay owns bridges and traffic shaping.
All cable bridges are unnumbered. Each directional delay is applied at the
destination host peer. The guest network configuration is a subsequent Ansible
step: merely creating a cable does not install addresses, routes, FRR or services.
No hidden management network is added. Out-of-band provisioning uses Incus's
control API, which is never exposed inside student containers.

Supported initial scope: IPv4/Ethernet IP labs with explicit data ports. Reject
InfiniBand manifests; use the existing ibsim backend for management-plane IB
experiments and native RDMA for payload evidence. Bridge behavior for LLDP,
STP/LACP, optical errors, hardware PFC and switch ASIC queues is not qualified.
Neither wall-clock netem latency nor a model label is hardware performance proof.

Validation in the development environment: compile and portable plan tests.
Live network qualification is **blocked**: creating a network namespace returns
`Operation not permitted`. Required before release: two simultaneous labs,
start/stop/restart, borrowed-peer identity changes, fault/restore traffic tests,
foreign member refusal, cleanup after each failure boundary, API timeout and
daemon restart. The patchbay companion PR records the same boundary.

API references: [Incus REST](https://linuxcontainers.org/incus/docs/main/rest-api/)
and [NIC devices](https://linuxcontainers.org/incus/docs/main/reference/devices_nic/).
