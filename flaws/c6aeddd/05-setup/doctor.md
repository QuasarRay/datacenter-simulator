# Prerequisite discovery mismatches

## SET-01 — Effective capabilities before bootstrap are the wrong test

**Medium · Confirmed predicate mismatch; CLI predicate reproduced.** [simulator/src/doctor.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/doctor.rs#L64) requires effective `CAP_NET_ADMIN` and `CAP_SYS_ADMIN` in the current process for `nccl`, `ibsim` and `vm`. It runs before any namespace bootstrap. The documented IB workflow runs as the checkout owner; [patchbay's pinned bootstrap](https://github.com/QuasarRay/patchbay/blob/3d3c577c4b49ba23cb6b14f6b48aacbec933a93e/patchbay/src/userns.rs) calls `unshare(CLONE_NEWUSER)` and sets UID/GID mappings.

The [Linux user-namespace contract](https://man7.org/linux/man-pages/man7/user_namespaces.7.html) allows a process with no capabilities in the parent namespace to acquire capabilities in its new user namespace, when host policy permits creation. Doctor therefore rejects that supported path based solely on privileges it does not need to hold beforehand. Conversely, current capability bits alone do not prove namespace creation is permitted under host policy.

The feature-enabled binary, with executable fixtures for its four IB tool names, produced:

```json
{"doctor_exit":1,"compiled":true,"all_programs_found":true,"namespace_privileges":false}
```

The fixture only isolates the predicate; it does not establish that this audit container can create namespaces. The source and documented unprivileged workflow establish the mismatch.

**Remedy/acceptance:** distinguish current capabilities, possible bootstrap and a bounded disposable-child bootstrap probe. Never mutate the caller's namespace in a diagnostic function. Test a normal unprivileged host with user namespaces permitted, a host that denies them, and a constrained container. Explain the actual failed capability/policy step rather than steering every user to sudo.

## SET-02 — Tool/path checks differ from the mode that will execute

**Medium · Confirmed by source comparison.** [simulator/src/doctor.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/doctor.rs#L20) requires `/dev/kvm` to exist, although [simulator/src/vm.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/vm.rs#L444) deliberately falls back to TCG for CPU smoke. GPU VM execution correctly has a stronger accessible-KVM/VFIO preflight. A single `vm` discovery result conflates them.

| Discovery behavior | Runtime difference / setup consequence |
|---|---|
| VM omits `ssh-keygen`, `sysctl`, `sha256sum`, `git`, `tar`, `cp`, configured Ansible tools and `/dev/net/tun` | These are consumed by `vm::preflight`, `LinuxFabric::build` or `deepops_runtime`; a successful lookup report is incomplete even before driver/image checks |
| IB checks PATH for `ibsim`/OpenSM/diagnostic binaries | Actual `IbOptions` can use explicit paths and also needs the `umad2sim` library; the discovery result cannot assess the supplied configuration |
| No Linux-only or physical-RDMA doctor mode | The feature matrix exposes both paths, but their users have no matching diagnostic entry point |
| Available memory is printed | No requested plan is accepted as input, so sufficiency is never checked |

These are prerequisite-discovery shortcomings, not a promise that doctor should replace runtime validation. The existing `scope` disclaimer is correct but does not resolve contradictory requirements.

**Remedy/acceptance:** derive discovery from the same per-mode prerequisite descriptions as execution, accept a read-only config/plan where needed and distinguish CPU smoke from GPU passthrough. Fixtures that remove each actual prerequisite must produce an actionable diagnostic before any state directory/native resource is created. Native success still requires the final runtime checks.
