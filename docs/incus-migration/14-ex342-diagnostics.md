> **AGENT INSTRUCTION: DO NOT IMPLEMENT THIS DOCUMENT.** This is a documentation-only migration proposal. Do not change code, configuration, dependencies, infrastructure, or runtime state from these instructions unless the user separately requests implementation. Documentation review and correction are permitted.

# EX342: diagnose and repair through Ansible

Source: [official EX342 page](https://www.redhat.com/en/services/training/ex342-red-hat-certified-specialist-linux-diagnostics-and-troubleshooting), checked 2026-09-23. These 30 rows follow its public leaf order. Every row includes observation, repair/configuration and proof; [15](15-ansible-recovery-runbooks.md) provides operational sequences. C/H/P and all product/host limitations are defined in [10](10-exam-coverage-contract.md).

## Method and observability: E342-01–06

| ID / topic | Class | Ansible diagnosis, action and verification |
|---|---|---|
| E342-01 — Evidence | C/H | Collect dated OS/kernel/package/service/storage/network/pressure facts before repair, with bounded journal windows and run IDs. Compare a healthy baseline; justify the likely failing layer. Recollect the same signals afterward. |
| E342-02 — References | C/H | Locate installed man/info/package/module documentation for an unfamiliar symptom, record the exact reference/version and test the suggested hypothesis with a read-only probe. Do not apply a fix solely because a text search matched. |
| E342-03 — Monitoring | C/H | Use bounded `vmstat`, `iostat`, `pidstat`, `ss`, pressure/cgroup and filesystem probes. Separate CPU saturation, memory pressure, I/O wait and descriptor exhaustion. Repair the actual constraint and compare the same workload. |
| E342-04 — Web console | C/H/P | Provision Cockpit on CachyOS with Ansible where the package is qualified, secure management access and inspect service/log/storage views. Compare the GUI observations with collected CLI data; label RHEL-specific features as product differences. |
| E342-05 — Automation | C/H | Reconstruct the desired state using narrow idempotent roles, with candidate validation, handlers, rescue and independent probes. Test failure to repair and demonstrate the play exits unsuccessfully. |
| E342-06 — Log routing | C | Configure a central collector and node forwarding with bounded retention. Inject a unique marker, trace it across sender/transport/receiver and fix a deliberate endpoint/TLS/filter error; prove delivery after restart. |

## Startup: E342-07–11

| ID / topic | Class | Ansible diagnosis, action and verification |
|---|---|---|
| E342-07 — Boot services | C/H | Inspect failed units, dependency order, journal and exit codes. Inject an invalid unit or missing prerequisite, validate a corrected drop-in/config, reload/reset failed state/start only the affected service, then test startup persistence. |
| E342-08 — Root recovery | C/H | Break a disposable guest's sudo/login, enter through authorized Incus recovery and repair account/policy; for a host use CachyOS rescue boot. Verify normal authenticated administrative access afterward. Recovery-channel root access alone is not proof of repaired login. |
| E342-09 — Boot chain | H | Inspect firmware/boot entries, kernel command line, initramfs, root UUID, encryption hooks and systemd target from rescue. Repair the observed layer, regenerate only the correct boot artifacts and require a real cold/normal boot. |
| E342-10 — Hardware | H | Collect PCI/USB/device enumeration, driver binding, kernel errors and relevant SMART/sensor data without destructive probing. Separate absent hardware, driver mismatch and physical failure; correlate with actual replacement/reconnection when required. Synthetic logs remain fixtures. |
| E342-11 — Modules | H | Inspect `lsmod`, `modinfo`, dependencies, current parameters and kernel logs. Manage approved load/blacklist/options files and initramfs as required; verify after reboot with the matching kernel. Never unload a host's active storage/network driver from a guest exercise. |

## Filesystems and storage: E342-12–15

| ID / topic | Class | Ansible diagnosis, action and verification |
|---|---|---|
| E342-12 — Corruption | H | Work on a copy of a disposable corrupted filesystem image. Collect read-only diagnostic output, unmount it, choose filesystem-specific repair with known exit-code semantics, then remount and compare recovered content. Preserve the original image. |
| E342-13 — LVM recovery | H | Inspect PV/VG/LV UUIDs, activation, device filters and archived metadata. Fix a controlled missing-PV/configuration issue or restore the exact lab metadata only after identity checks; verify extents and data, not just an active LV name. |
| E342-14 — Encryption | H | Inspect LUKS header/UUID, key availability, mapping and filesystem separately. Open with an authorized protected key, repair crypttab/initramfs configuration or restore a matching header backup when justified. Verify content and boot unlock; there is no recovery without required key material. |
| E342-15 — iSCSI | H | Trace target reachability, portal/IQN, CHAP, session, discovered LUN, device and mount layers. Repair only the failed layer on dedicated target/initiator hosts; verify real I/O and reconnect/reboot persistence. Never log CHAP secrets. |

## Packaging: E342-16–18

| ID / topic | Class | Ansible diagnosis, action and verification |
|---|---|---|
| E342-16 — Dependencies | C | Inspect pacman package versions/dependency graph, configured repos, signatures and transaction logs. Repair coherent repository/package state with a full supported transaction; do not use `--nodeps` as a generic cure. Run the affected application afterward. |
| E342-17 — Database | C/H | Back up the pacman database/cache before a disposable corruption fixture. Identify lock vs sync-cache vs local metadata damage, recover from trusted metadata/known installed packages, and verify consistency plus package operations. This is the CachyOS adaptation, not RPM database reconstruction. |
| E342-18 — Modified files | C | Use package ownership/integrity and `.pacnew`/`.pacsave` inspection, compare with trusted package payloads, and preserve intentional local settings. Reinstall or template only the proven damaged file/config, then verify metadata and actual service behavior. |

## Connectivity: E342-19–21

| ID / topic | Class | Ansible diagnosis, action and verification |
|---|---|---|
| E342-19 — Probes | C | Collect link/address/neighbor/route/rule/listener/resolver state and source-bound ping/TCP/application checks. Compare both endpoints. ICMP success is insufficient for DNS, TCP or a service response. |
| E342-20 — Repair paths | C | Inject address/prefix, default/return route, firewall, MTU, DNS and FRR policy faults separately. Form a hypothesis from observations, repair one cause and rerun the originally failing flow through the intended patchbay edges. |
| E342-21 — Packets | C/H | Run a bounded namespace-scoped tcpdump capture with a narrow filter and size/time limit. Fetch the pcap securely, correlate ARP/ND/DNS/TCP behavior with routes/counters, repair and capture again. Protect payload/credential privacy. |

## Applications: E342-22–26

| ID / topic | Class | Ansible diagnosis, action and verification |
|---|---|---|
| E342-22 — ABI | C | Inspect trusted binaries with `readelf`/`objdump`, loader paths and package ownership. Compare required libraries/symbol versions to installed packages; repair the compatible dependency set and run the real workload. Avoid executing untrusted binaries via diagnostic loader tools. |
| E342-23 — Leaks | C/H | Start a bounded known-leak fixture and collect repeated RSS/PSS/heap evidence under the same workload. Distinguish leak, caching and cgroup limit pressure; compare with a corrected build using a suitable profiler/Valgrind where qualified. A restart only clears the symptom. |
| E342-24 — Debugging | C/H | Inspect unit exit/status, bounded `strace`, core dumps and debugger backtraces for a controlled crash/hang. Fix config/permissions/dependency or provide actionable code-level evidence, then rerun the trigger. Record ptrace/core policy limitations. |
| E342-25 — SELinux | H | On the qualified SELinux CachyOS host, correlate real AVC, file/process/port context and Boolean state. Fix the intended policy mapping, keep enforcement active and prove both allowed and still-denied accesses. Do not blindly run `audit2allow`. |
| E342-26 — Container app | C/H | Inspect Incus operation/config/limits, image identity, rootfs ownership, namespaces, mounts, service journal and network endpoints for a failing application. Repair the demonstrated boundary and verify from a client after restart. Add separate CRI/pod diagnosis for Kubernetes workloads; Docker is not required. |

## Authentication and escalation evidence: E342-27–30

| ID / topic | Class | Ansible diagnosis, action and verification |
|---|---|---|
| E342-27 — PAM | C | Preserve recovery access, inspect auth logs and PAM stack ordering/control flags, account expiry and permissions. Repair a single controlled module/config defect; test allowed/denied normal logins and sudo rather than only Incus exec. |
| E342-28 — Identity provider | C/H/P | Separate DNS/time/TLS, NSS lookup, directory credentials, group authorization and home access. Diagnose the qualified LDAP/Kerberos/SSSD lab, repair its actual cause and test online/offline policy. Product-specific identity management remains labeled. |
| E342-29 — Crash dump | H | Prepare a compatible capture kernel, reserved memory and dump target on a disposable CachyOS host; verify readiness, trigger only the controlled lab crash, then collect vmcore and matching symbols. Ansible after reboot checks dump validity; a container core is not a kernel dump. |
| E342-30 — SystemTap | H | Qualify headers/debug symbols/toolchain for the exact running kernel, compile a reviewed bounded probe, load/run it and collect events during a known workload. Verify module cleanup. eBPF/perf alternatives are useful but do not count as compiling/running SystemTap. |

## Troubleshooting grading

Use two stages: diagnose without mutation, then repair after the cause/hypothesis is recorded. Require raw observations supporting the choice, a minimally scoped change, original symptom resolution, persistence, and clean cleanup. A log collector, reboot or `ignore_errors` that masks the symptom must not earn a pass.

A run can conclude “hardware replacement required” or “application defect isolated” with correct evidence when an actual fix is outside the available control. Record this as a diagnosis outcome, not repaired service success. Preserve a sanitized bundle with timeline, reproduction, versions, affected identifiers, relevant configs, logs, pcap/core/symbol metadata and unsuccessful hypotheses for third-party investigation.

This curriculum keeps every objective visible even when the necessary physical host, SELinux stack or product is unavailable. Those rows remain blocked; a default unprivileged Incus lab cannot honestly execute them all.
