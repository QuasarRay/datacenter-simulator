> **AGENT INSTRUCTION: DO NOT IMPLEMENT THIS DOCUMENT.** This is a documentation-only migration proposal. Do not change code, configuration, dependencies, infrastructure, or runtime state from these instructions unless the user separately requests implementation. Documentation review and correction are permitted.

# Ansible recovery runbooks

These are future exercise specifications, not executable playbooks shipped in this PR. Every run starts by checking exact lab/host/device identity and an instructor-owned fault record. The Ansible controller and recovery channel must be outside the failure domain being broken. All commands below describe probes or bounded future actions; destructive examples deliberately require selected disposable targets rather than guessing `/dev/sda`.

## R1 — Recover when SSH, sudo or Python is broken

Applies to E200-04/05/54/57 and E342-08/27.

1. From the controller, observe Incus instance state and management link/address. Probe TCP/22 separately from authentication. A timeout, refused connection and rejected key indicate different layers.
2. For a running guest, switch only its **recovery inventory alias** to `community.general.incus`. Disable fact gathering initially. Check `/usr/bin/python`, `systemctl status sshd`, bounded SSH journal, `ss -lnt`, SSH config, authorized-key paths/ownership and account status. Use `raw` only if Python is unavailable.
3. Validate a candidate SSH configuration with `sshd -t -f <candidate>` before deploying it. Check Match/AllowUsers rules and daemon path from the actual CachyOS package. For sudo, validate the candidate fragment with `visudo`; for PAM, preserve a backup and repair the identified line rather than replacing all vendor files.
4. Restart/reload only the affected unit after validation, open a **new normal SSH connection**, and verify intended user/group/become behavior. Test a wrong credential and revoked/expired account. Keep the recovery channel until normal access is proved.
5. Restore the normal inventory transport, repeat after container restart and remove temporary recovery authorization where created. Collect only redacted account metadata, never passwords or private keys.

Unreachable SSH is not handled by an ordinary task `rescue` block automatically. Use a separate controller/recovery play and explicit transport reset. If Incus init is broken, stop the node and use an authenticated Incus recovery/export workflow to repair an isolated copy of its rootfs; do not `nsenter` a stale PID and guess ownership.

## R2 — Bootloader, root access, initramfs and target recovery

Applies to E200-20–22/43/46 and E342-07–11.

An Incus container shares the host kernel and has no independent firmware/bootloader boot. Use a disposable **CachyOS host** with a second controller, console/rescue access and known disk identities. Ansible cannot contact a host that never reaches networking without such an alternate path.

1. Before fault injection, record partition UUIDs, root/boot/ESP mount layout, encryption, kernel packages, command line, boot entries, active boot manager and initramfs generator. Back up the exact boot config and tested rescue instructions. Do not assume every CachyOS installation uses GRUB or mkinitcpio.
2. Inject one recoverable fault: bad root UUID in a copied entry, missing initramfs hook, wrong default target or failed required unit. Preserve a known-good entry. Capture console symptom and last reachable state.
3. Boot verified CachyOS rescue media. Run Ansible locally there, or expose a temporary authenticated SSH endpoint for the external controller. Identify and mount the installed root and its real boot/ESP layout explicitly. Unlock encryption only with authorized keys.
4. Compare intended UUIDs and boot config to observed files; inspect prior journal where available. Repair the cause using the appropriate role. A service issue may require only a unit/config correction; avoid reinstalling the bootloader blindly.
5. If needed, enter the installed environment using a properly prepared chroot with required pseudo-filesystems. Rebuild through the **detected** generator: use the qualified dracut workflow for dracut installations or `mkinitcpio -P` for installations that actually use mkinitcpio. Regenerate/update only the detected boot manager's artifacts, with its real ESP paths and entries. Avoid universal copy-paste GRUB installation commands.
6. Unmount cleanly, reboot the host, wait from the external controller and verify boot ID, intended kernel/command line, target, mounts and normal SSH/root-control policy. Repeat a normal reboot. Retain evidence showing rescue restored the actual installed system.

For kernel modules, inspect `modinfo`, dependencies and `/sys/module/<name>/parameters`. Stage a persistent config and load only a safe test module on the disposable host. Test wrong kernel/headers and rejected parameter errors; unloading an active disk/network module is not a standard classroom repair.

## R3 — Storage, LVM, encryption and iSCSI

Applies to E200-30–40 and E342-12–15.

Keep three copies when testing corruption: original fixture, working copy and known-good data/hash manifest. Record exact backing file/device IDs, size and mounts. Abort if the selected device has any relation to the host root, Incus pool or an unrelated VG. `check_mode` does not make a destructive storage command safe.

1. Collect `lsblk -J`, `blkid`, `findmnt`, `pvs/vgs/lvs` structured reports, `dmsetup` metadata and relevant kernel logs. For iSCSI add portal/session/node information. Correlate filesystem→LV→VG→PV→partition→disk before changing anything.
2. For a bad mount entry, validate UUID/options and unit dependencies first. For filesystem corruption, unmount the working copy and use a filesystem-specific read-only diagnostic. `e2fsck` return codes distinguish clean, corrected, reboot-needed and uncorrected states; do not globally treat every nonzero code as identical. `xfs_repair -n` is a diagnostic, not repair. Avoid destructive log-zeroing options unless the exercise explicitly studies their data-loss implications.
3. For LVM, check missing devices, filters/devices files and activation before metadata restoration. Use the exact matching archived metadata for the lab VG, with UUID checks and a restore rehearsal on a copy. Never invent a missing PV's contents from its name.
4. For LUKS, verify header UUID/keyslot availability and separate unlock failure from filesystem/mount failure. Pass keys via a protected file/descriptor and `no_log`, not command-line literals. Back up headers securely; restore only a matching header to the matching disposable device. Without the necessary key material, report unrecoverable access rather than promising decryption.
5. For iSCSI, verify transport first, then target/IQN/ACL/CHAP, then login/session, then discovered LUN and multipath/mount state. Repair the relevant configuration, perform bounded reconnect, write/read a sentinel and verify persisted session/mount ordering. A ping to the target cannot certify block I/O.
6. After repair, remount, compare application data/checksums/ownership, perform a controlled persistence check, and report any lost/recovered files. Preserve pre-repair images and logs for review. Clean up loop mappings, temporary mounts and lab LVM resources in dependency order.

Host block devices are global kernel resources. A bind-mounted directory into an unprivileged container can teach permissions and mount consumption, but it does not replace authentic PV/partition/corruption exercises on the host.

## R4 — Packages, database and configuration drift

Applies to E200-12–15/45 and E342-16–18.

1. Capture pacman configuration, installed package versions, transaction log, lock ownership, available disk/inodes and repository/signature errors. Determine whether another package manager is active before considering any stale lock removal.
2. Use `pacman -Qkk` and package ownership queries for controlled integrity checks, then compare with verified package archives. Distinguish intended administrator modifications from corruption; `.pacnew` and `.pacsave` require a reviewed merge, not deletion.
3. For dependency failure, reconcile repository state and complete a supported coherent upgrade. For damaged sync metadata, rebuild the appropriate cache through pacman. For damaged local installed-package metadata, work from a backup and known trusted package manifests/cache; reconstruct/reinstall the known package set through a documented recovery process and verify consistency. Do not use RPM recovery commands or indiscriminately delete `/var/lib/pacman`.
4. Run the affected service/application and another package transaction, then compare the saved configuration and package versions. Reproduce from the same dated image/package set. Keep signatures enabled and record any package unavailable in the chosen snapshot.

These steps are intentionally a pacman adaptation. RPM query/verification and database recovery have different semantics and remain explicitly unproven by this exercise.

## R5 — Network diagnosis from layers to packets

Applies to E200-47–50, EX457 data-plane drills and E342-19–21.

Collect link/carrier/MTU, addresses, neighbors, route/rule tables, bridge/VLAN membership, FRR adjacency/FIB, nftables/firewalld state, listening sockets and DNS from both endpoints and relevant transit nodes. Always record namespace and source address. Separate management availability from workload availability.

Use a branching investigation: absent link → lifecycle/wiring; missing neighbor → L2/address scope; wrong next hop → routing/policy; SYN without reply → return path/filter/listener; DNS-only failure → resolver/server; small probes working but large flows stalling → MTU/PMTUD. Confirm hypotheses with packet/counter evidence rather than assuming a fixed troubleshooting order proves the cause.

For packet capture, create a private destination directory, apply a narrow filter, cap count/time/file size, and run under a bounded process group. Fetch captures with appropriate mode and retention. Restarting the capture after a container restart requires the new namespace/interface identity. A host-wide capture may see management bypasses but is not a substitute for endpoint captures.

Repair one identified state owner: Incus device, patchbay edge, network manager, FRR config or firewall. Verify the original failing flow and a negative policy case. Cut every intended path and confirm failure with management up. Reboot/restart the correct boundary and rerun to prove persistence.

## R6 — Application, identity and policy diagnosis

Applies to E342-22–28 and E200 security rows.

For an application, correlate service exit status, journal, configuration validation, resource usage, dependency/loader information and client behavior. Use `readelf`/`objdump` for unknown binaries rather than executing them through a diagnostic loader. Run trusted debug fixtures under bounded strace/debugger/Valgrind instrumentation; collect matching build IDs and symbols. Separate a heap leak from a growing cache, allocator retention or shared page accounting. Fixing a source defect may mean producing a precise upstream bug bundle rather than pretending Ansible can patch arbitrary software safely.

For identity, test DNS/time/TLS before directory authentication; test NSS lookup (`getent`/`id`), credential validation, authorization, session setup and home access separately. Inspect SSSD/PAM/Kerberos logs only within a redacted window. Repair the demonstrated cause and test both allowed and denied users through the normal SSH/application route, including intended offline behavior.

For SELinux, require a real qualified CachyOS SELinux kernel/userspace/policy stack on a disposable host. Confirm active LSM and enforcement, collect AVCs, inspect process/file/port labels and relevant Booleans, then change the minimal intended policy/config mapping. Prefer persistent fcontext plus relabel for moved service paths; do not use blanket permissive mode or automatically generate policy from every denial. If the stack cannot be qualified, this is a blocked lab, not an AppArmor substitute.

SELinux qualification itself needs a recorded procedure: inspect the selected kernel's SELinux/security options and boot LSM selection; obtain a mutually compatible, reviewed userspace set (libselinux, libsepol, libsemanage, policycoreutils and Python bindings) plus a policy appropriate to CachyOS paths/services; verify SELinux-aware core applications; load and relabel an isolated installation under a recoverable initial policy; then boot and test enforcement, AVC reporting and the required management tools. Package names/availability are not guaranteed on CachyOS. Use the [upstream userspace component guide](https://github.com/SELinuxProject/selinux/wiki/Userspace-Packages) to identify dependencies and qualify distro-specific builds; do not install an arbitrary RHEL policy package and assume compatibility. Preserve a known-good boot path throughout.

For container applications, trace image fingerprint → effective config/profile → UID mapping/volume ownership → cgroup/device/LSM policy → PID 1/service → endpoint. Distinguish Incus host errors from guest application errors. For Kubernetes additionally inspect pod events, CRI/container logs, image pull, device plugin and CNI state. Repair each layer through its owner and preserve the distinction between CPU Mokka contracts and real GPU execution.

## R7 — Kernel dump and SystemTap evidence

Applies to E342-29–30. Use a disposable CachyOS host with console, backup and an external controller; these procedures affect the shared kernel and cannot be safely isolated by an ordinary system container.

For kdump, first qualify kernel support, kexec tooling, architecture/boot security restrictions, capture-kernel/initramfs, reserved memory and accessible dump storage. Stage configuration with Ansible, reboot into the intended kernel, and verify the capture image is actually loaded. A deliberately triggered crash is a separate instructor action after readiness checks. After recovery, Ansible verifies vmcore existence, size/readability, capture log, exact kernel build ID and matching symbols. A user-process core or a copied dummy file does not pass. Do not hard-code one crashkernel size for every laptop.

For SystemTap, pin toolchain, headers, matching debug information and a kernel-compatible SystemTap version. Compile a reviewed bounded script, record compiler diagnostics and module identity, load it subject to signing/lockdown policy, run a known workload, collect expected events, and unload/verify cleanup. Missing symbols, compile failure, denied module load and no events are different failures. A bpftrace/perf approximation may aid diagnosis but must not be graded as SystemTap execution. Keep output bounded and never load arbitrary student-generated privileged probes on shared infrastructure.

For third-party escalation, bundle the minimal sanitized reproducer, timeline, kernel/package/build IDs, hardware identities, relevant configuration, dump/probe metadata and observations before/after attempted repair. State whether the system was repaired, only diagnosed, or blocked by missing prerequisites.

Sources: [CachyOS boot configuration](https://wiki.cachyos.org/configuration/boot_manager_configuration/), [Linux kdump documentation](https://www.kernel.org/doc/html/latest/admin-guide/kdump/kdump.html), [SystemTap guide](https://sourceware.org/systemtap/SystemTap_Beginners_Guide/), [Ansible error handling](https://docs.ansible.com/projects/ansible/latest/playbook_guide/playbooks_error_handling.html). Read the precise filesystem/package/security tool documentation for the pinned environment before implementing destructive repair steps.
