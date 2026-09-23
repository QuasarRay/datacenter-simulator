> **AGENT INSTRUCTION: DO NOT IMPLEMENT THIS DOCUMENT.** This is a documentation-only migration proposal. Do not change code, configuration, dependencies, infrastructure, or runtime state from these instructions unless the user separately requests implementation. Documentation review and correction are permitted.

# Exam coverage contract and CachyOS boundaries

The four following ledgers use the public Red Hat pages retrieved on **2026-09-23**, linked in [18](18-sources-and-version-policy.md). They enumerate leaf objectives in page order, retaining repeated topics where the source repeats them. EX294 also has four standalone tasks between inventory configuration and playbook execution; they get their own rows. These local IDs are not Red Hat objective numbers.

| Ledger | Local IDs | Rows | Practice design |
|---|---|---:|---|
| [EX200](11-ex200-system-administration.md) | E200-01–E200-62 | 62 | CachyOS system administration through Ansible |
| [EX294](12-ex294-ansible-administration.md) | E294-01–E294-57 | 57 | Administration, authoring, tooling, reuse and secrets |
| [EX457](13-ex457-network-automation.md) | E457-01–E457-23 | 23 | Network automation and Controller |
| [EX342](14-ex342-diagnostics.md) | E342-01–E342-30 | 30 | Evidence-driven troubleshooting and recovery |

Counts describe the retrieved public pages, not every historical or private exam version. EX457 explicitly identifies AAP 2.6 and multiple versions. Confirm the booked version's objectives separately; do not silently use v5 evidence or older online lists. Older container objectives are not inserted into current EX200 as though they were present; container troubleshooting appears explicitly in EX342 and in the migration curriculum.

## Execution classes

| Code | Meaning | Grading rule |
|---|---|---|
| C | CachyOS system container or its Ansible control environment | Execute and verify within the stated namespace/service boundary. |
| H | Disposable CachyOS host or CachyOS rescue boot | Required for real boot/kernel/global storage/security operations. Container restart is insufficient. |
| P | Product-specific or external endpoint | Exact AAP/IDE/provider behavior requires that product; a substitute remains marked adapted. |

The class is a location requirement, not permission to perform a dangerous action on a personal laptop. Host labs use named disposable devices/hosts and recovery access. The primary lab still uses Incus system containers; host labs are necessary tests of the shared kernel/boot boundary and do not reintroduce Docker or QEMU node infrastructure.

## Resolve unavoidable non-equivalences explicitly

| Topic | CachyOS practice | What it cannot establish |
|---|---|---|
| RPM repositories/CDN/database | Signed pacman repositories/packages, dependency diagnosis, cache/database recovery | Red Hat subscription/CDN or RPM database mechanics; do not call `rpm --rebuilddb` on pacman's database. |
| SELinux | A dedicated CachyOS host with verified SELinux kernel, userspace and policy, then real AVC/context/port/Boolean exercises | Default AppArmor or a fake `getenforce` output cannot satisfy SELinux behavior. If no qualified stack is available, retain `BLOCKED`. |
| Bootloader/initramfs | Actual detected CachyOS boot manager and generator, with rescue access | GRUB-only instructions do not cover systemd-boot/Limine, and container init is not a kernel boot. |
| RHEL Web console | Cockpit on CachyOS, with Ansible configuring/collecting the same state | RHEL-specific integrations/product support; GUI interpretation still requires the learner. |
| Navigator/VS Code EE/dev container | Incus controller plus Remote SSH for the baseline; qualified Podman OCI development tooling for exact dev-container mechanics where permitted | Remote SSH into Incus is not a VS Code Dev Containers backend; Incus is not a Navigator OCI engine. |
| Automation Controller/AAP | CachyOS Ansible client against an actual qualified Controller; AWX may provide an explicitly separate compatibility exercise | AAP support on CachyOS, server RBAC/EE behavior from module-spec tests, or equality of AWX and AAP. |
| Physical/kernel failures | Dedicated CachyOS host with actual hardware/kernel evidence | Full hardware faults, panic dumps or module loading from an unprivileged system container. |

**Scope of OCI tooling:** the operational simulator has no Docker/Containerlab/Kind/Dagger backend. Kubernetes containerd and narrowly scoped Podman-based EE/dev-tool exercises are workload/tool protocols, not replacement infrastructure nodes. No lab deployment may fall back to them. If the intended policy is stricter—no OCI execution at all—retain the associated EE/dev-container items as documented, blocked product exercises; do not falsely mark the Incus adaptation equivalent. No separate RHEL installation is prescribed as a workaround for the CachyOS-only OS requirement.

## Use Ansible for the entire troubleshooting cycle

For each objective create future `prepare`, `inject`, `diagnose`, `repair`, `verify`, and `reset` stages. Fault injection is instructor-controlled and not automatically repaired before diagnosis. Ansible captures raw evidence and structured assertions, runs the chosen repair, then verifies from an independent observer. Store hypothesis/reasoning alongside the automation output so a playbook that blindly restarts everything does not pass diagnosis.

Visual editing, console interaction and interpreting a hardware symptom remain human skills. Ansible prepares the environment, drives diagnostics and validates artifacts; it does not make an editor GUI click or a failed machine's firmware accessible through a dead SSH session. Rescue can run Ansible locally after booting CachyOS rescue media, or remotely from another controller once connectivity exists.

## Evidence and completion

Per-row evidence must include objective ID, source date/version, run/commit/image/plan identity, execution class, prerequisite status, pre-fault observation, selected repair, exit codes, post-repair independent probe, persistence result and second-run convergence. Record `scope: cachyos-adaptation` where appropriate. A missing host, product, kernel feature or physical device is a visible block, never an automatic skip-to-pass.

For every group, include at least one negative control: a real wrong state must fail the verifier. For boot-persistence claims, reboot the correct host or restart the intended container and explicitly state which boundary was tested. A rebuild from a clean base is additionally required for Ansible configuration reproducibility.
