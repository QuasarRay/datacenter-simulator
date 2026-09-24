# Native provisioning and integration contract

The native lab has one explicit ownership chain: NetBox intended objects → the
existing Rust `netbox::compile` validator → simulator manifest/petgraph → Incus
instance plan → patchbay host cables → guest network configuration → native
control-plane/services → independent workload and failure observations.

`dependencies.lock.json` pins the inspected upstream implementations. A pin is
source identity, not a compatibility certificate. Build artifacts need their own
SHA-256 identities, generated with `services.py`, before `services.yml` accepts
them. Never supply an invented digest to get past a preflight.

## Code-first entry point

Prefer the [NetBox compiler practical](../../education/ncp-metablueprint/courses/C01-NETBOX.md)
for new fabrics. `Fabric.compile()` acquires a bounded read-only NetBox snapshot,
then generates the native manifest, guest addressing, static routes and Ansible
variables. Its resource reservations call the same Verus/Kani-verified Rust kernel
used by the assessment system. The graph compiler and OS remain outside that proof.
The decorator `@native_fabric` turns a parameterized intent function into the full
acquire/compile/seal operation. `deploy()` and `reconcile()` realize the generated
plan through Incus, patchbay and actual DeepOps. No hand-written per-node topology
is required. An offline snapshot must be explicitly selected and is not live evidence.

After the Rust lifecycle creates the lab, trainer code uses:

```python
from pathlib import Path
from lab import Lab
from services import control_plane, workers

lab = Lab(Path('/srv/ncp/session-001'))
lab.reconcile_hosts()  # executes NVIDIA's pinned deepops_hosts role
declarations = {
    'control': control_plane('/srv/ncp/qualified-binaries'),
    'services': workers('/srv/ncp/qualified-binaries'),
}
lab.apply('integrations/incus/services.yml', {'ncp_services': declarations})
```

Run this with `integrations/incus` on the Python import path. This is trainer
automation, not code to execute with the host socket inside a student container.
`NCP_JOURNAL` supplies ownership to the custom Ansible connection. API operations
wait for completion; guest exit status and stdout/stderr remain separate.

Before services start, stage a real `ncp` service account, writable
`/var/lib/ncp`, pinned binaries, TLS files, KWOK kubeconfig and stage definitions
inside the chosen image or via reviewed Ansible tasks. The service generator
never fabricates those prerequisites. Supply the JWT secret via a protected
systemd environment file. Keep API authentication enabled and verify the client
CA with a positive client plus a rejected untrusted client.

The initial Rusternetes deployment consists of its actual API server, scheduler
and controller manager, plus native etcd. It deliberately builds/starts no
Rusternetes kubelet. Its API server also needs the explicit
[native source profile](../rusternetes/README.md), which removes runtime clients
and fabricated pod-log fallback and wires the binary client-CA option correctly.
Client TLS verification and bearer-token RBAC are separate requirements.
KWOK owns only nodes selected by `ncp.simulated=true`. Generated workers have a
NoSchedule taint; only explicitly synthetic workloads may tolerate it.
They are worker objects, not a thousand control planes or executing GPUs.

## DeepOps fidelity

Every course starts from `Lab.reconcile_hosts()` and retains the resulting
DeepOps revision and Ansible transcript. Add actual upstream roles only after
checking their target distribution and effects. The shipped common path executes
the real `deepops_hosts` role. This is **not** a claim that full upstream Slurm,
Kubespray, GPU-driver or DPU installation has been ported to CachyOS.

The upstream Slurm role has distribution-dependent package, bootloader, service
and prolog behavior. The upstream Kubernetes entry point installs Kubernetes via
Kubespray; it does not install Rusternetes. Do not forge distribution facts or
rename Rusternetes to kube-apiserver to suppress those incompatibilities. Native
CachyOS Slurm packaging, upstream template reuse, Enroot/Pyxis, GPU drivers and
real workload validation need separately qualified site adapters. Record an
unsupported role as blocked, not silently skipped success.

## NetBox and service plane

Use native PostgreSQL, Redis/Valkey and NetBox processes in the services
container. A source installation requires a pinned NetBox checkout, its exact
Python requirements, a dedicated database/user, a private secret configuration,
schema migrations, static files and a service user. Reuse NetBox's own installation
procedure and systemd examples; do not translate an application image tag into a
source revision or reuse the former CI credentials. This backend currently
accepts a provisioned NetBox endpoint; automatic native NetBox bootstrap remains
an unmet integration gate.

The existing `integrations/netbox/config.example.json`, importer and snapshot
validation are retained. Set the chosen device-role OS mapping to `cachyos` or `ubuntu-24.04`
before compiling an Incus manifest. Retain explicit cable medium; IB snapshots
must select ibsim instead of being flattened to Ethernet. Keep NetBox as intent:
measured state must not overwrite desired state merely to make a comparison pass.

## Required compatibility experiments

Before marking a station live, collect all of the following from its run:

- API discovery, create/get/list/watch, status-subresource update, finalizers and
  resourceVersion conflict behavior against the pinned Rusternetes server.
- KWOK node heartbeat and pod transitions, a selected worker and an unselected
  worker, and recovery after API disconnection. Synthetic pod readiness alone is
  not an application result.
- For Network/GPU Operator or Mokka, real CRD/controller/admission/device-plugin
  observations. If these cannot execute with this runtime, keep that objective
  blocked; an accepted CRD or synthetic condition does not prove an operator ran.
- Source-bound network traffic, cut/restore of the required patchbay cable, exact
  interface counters and absence of a path over the provisioning channel.
- For native NCCL/RDMA, the existing simulator collectors, hardware identities,
  numerical validation and tested transport. Do not upgrade Socket evidence to
  GPUDirect RDMA evidence or ibsim management traffic to real payload bandwidth.

## Cutover disposition

The old root topology template, application images, nested-daemon/Dagger engine,
NetBox compose deployment and kind cluster are removed. The root deploy/destroy
playbooks now call the Incus lifecycle. Historical EX457 launchers refuse use
with a migration explanation; their written lessons, pure contracts and audit
fixtures are preserved. These historical release checkers are not green evidence
for the new platform. Their FRR/SSH live fixtures need a qualified Incus image and
owned journal before reuse. Upstream DeepOps source is preserved verbatim, so its
unselected legacy role definitions remain reference material.

The Incus qualification workflow runs real native containers on a disposable
Ubuntu 24.04 GitHub host. It retains image fingerprints, Ansible transcripts,
traffic/cut/restore/isolation and cleanup results. A separate native control-plane
workflow builds the derived Rusternetes and pinned KWOK binaries and probes their
actual capability boundaries. A workflow definition is not a successful result;
consult its retained qualification report. CachyOS and NVIDIA hardware remain
separate host profiles. Portable checks alone cannot qualify a live station.
