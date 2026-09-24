> Implementation authorized by the project owner on 2026-09-24. The former implementation prohibition is withdrawn. See integrations/incus for implemented behavior and validation boundaries.

# EX200: system administration through Ansible on CachyOS

Coverage: 62 public leaf objectives in source order. Topic labels below are short indexing aids; procedures are original CachyOS lab designs. Source: [official EX200 page](https://www.redhat.com/en/services/training/ex200-red-hat-certified-system-administrator-rhcsa-exam), checked 2026-09-23. Use [10](10-exam-coverage-contract.md) for C/H/P classes and adaptation rules, [06](06-ansible-orchestration.md) for task conventions and [15](15-ansible-recovery-runbooks.md) for detailed recovery.

## Tools: E200-01–11

| ID / topic | Class | Ansible practice and independent proof |
|---|---|---|
| E200-01 — Shell | C | Invoke fixed `command.argv` tasks, then a deliberately invalid option. Compare return code/stdout/stderr and correct the invocation. Run a POSIX shell exercise with an explicit interpreter; the user's interactive fish shell must not change playbook semantics. |
| E200-02 — Streams | C | Template a small Bash script exercising overwrite, append, pipes and separate stderr. Run using `command`, fetch all output files and assert their exact contents. Demonstrate a failed pipeline with `pipefail`; avoid hiding failure behind its last command. |
| E200-03 — Patterns | C | Seed logs with matching/nonmatching records; use grep/regular-expression exercises and Ansible regex filters. Assert selected lines and a negative case. Explain grep's no-match exit separately from command failure. |
| E200-04 — SSH | C | Install/start SSH, deploy a verified key and known-host entry, then run a remote command through the SSH inventory. Test denied wrong key and changed host key. Incus exec alone cannot pass this row. |
| E200-05 — Identity | C | Create two users, exercise login and explicit `become_user`, collect `id`, groups, environment and effective privileges. Check denied escalation and successful authorized access in a multi-user systemd environment. |
| E200-06 — Archives | C | Create tar/gzip/bzip2 artifacts from a controlled fixture, unpack with `unarchive` or documented tools, then compare content, paths, permissions and numeric ownership. Reject path traversal in untrusted archives and test restoration to a new directory. |
| E200-07 — Text | C | Use `copy`, `template`, `lineinfile` and `blockinfile` for distinct suitable cases. Validate a candidate config before replacement; slurp and compare the final file, then require a converged second run. |
| E200-08 — Paths | C | Create directories/files, copy and move fixtures, and remove only owned paths. Assert type, content and absence using `stat`/`find`. Teach rename semantics and idempotent guards instead of replaying an unconditional `mv`. |
| E200-09 — Links | C | Use `file` states for symbolic and hard links. Compare inode/device values for hard links and resolved targets for symlinks; delete the source and explain each observed outcome. Include cross-filesystem hard-link refusal. |
| E200-10 — Modes | C | Set owner/group and quoted octal mode with `file`. Test access as owner, group member and unrelated user, including directory traversal. A mode-string assertion alone does not prove effective access. |
| E200-11 — Manuals | C | Install available man/info/package documentation, locate relevant local references and capture the chosen command synopsis with Ansible. Write the diagnostic decision based on that reference; keep an offline source/version record. |

## Software and scripts: E200-12–19

| ID / topic | Class | Ansible practice and independent proof |
|---|---|---|
| E200-12 — Repositories | C | Template a signed pacman repository/mirror configuration and install its trusted key through a verified source. Probe repository availability and signatures, break the URL/key separately, then diagnose. This adapts the RPM-repository objective. |
| E200-13 — Packages | C | Install/remove a harmless package through `community.general.pacman`; check package database and executable availability. Use coherent full upgrades, never a partial upgrade. RPM transactions remain a product difference. |
| E200-14 — Flatpak remotes | C/H | Manage a real remote through `community.general.flatpak_remote`, specifying system/user scope. Confirm `flatpak remotes` as the same user. Require working user-session/nesting prerequisites or run on the disposable host. |
| E200-15 — Flatpaks | C/H | Install/remove a pinned small application/runtime with `community.general.flatpak`; assert installation in the intended scope and actual execution where applicable. A pacman package with the same name does not count. |
| E200-16 — Branching | C | Template a Bash script with `if`/`test` cases and execute fixtures for both branches through Ansible. Assert exit codes and effects, including empty input and a false condition. |
| E200-17 — Iteration | C | Have a script loop over controlled files and arguments, including spaces. Invoke from Ansible with fixed argv; compare the produced result list. Do not replace the shell-language exercise solely with an Ansible loop. |
| E200-18 — Arguments | C | Pass multiple positional parameters to the templated script, validate missing/extra/space-containing inputs, and assert correct quoting. Secrets must not appear in process arguments or task logs. |
| E200-19 — Substitution | C | Capture a command's output inside a script and branch on validated content/exit status. Compare with Ansible `register`; test a failing producer so stale/empty output cannot be accepted. |

## Runtime operation: E200-20–29

| ID / topic | Class | Ansible practice and independent proof |
|---|---|---|
| E200-20 — Power | C/H | Orchestrate orderly container stop/start through Incus, and real host reboot/shutdown through a dedicated host controller. Record boot ID or init generation and service recovery. A stop/start of Incus is not host power-cycle evidence. |
| E200-21 — Targets | H | From a disposable host with console/rescue access, select a different systemd target as a one-time operation; collect active target/services before and after. Ansible arranges and verifies the drill; loss of SSH must have a recovery path. |
| E200-22 — Rescue | H | Prepare a boot interruption/root-recovery scenario on disposable CachyOS media. Use console/rescue to reach a usable control environment, then Ansible to repair the installed root and verify normal boot. See runbook R2. |
| E200-23 — Processes | C | Start bounded CPU/memory fixtures under dedicated units. Collect `ps`, cgroup usage and pressure, identify the responsible PID/unit, terminate only that fixture, and demonstrate restored health. Include a wrong-PID negative control. |
| E200-24 — Scheduling | C/H | Read nice/priority and apply a documented unit `Nice=` or controlled renice operation. Verify effective settings and persisted unit configuration. Distinguish scheduling priority from an Incus CPU quota. |
| E200-25 — Profiles | H | Qualify TuneD on the selected CachyOS snapshot; manage its profile with Ansible and inspect actual applied tunables. Compare before/after and undo. `power-profiles-daemon` or an arbitrary sysctl is not silently substituted for TuneD. |
| E200-26 — Logs | C | Collect journal and relevant service logs bounded by boot/time/unit. Correlate a unique failure marker with service status and exit reason. Retain raw evidence before running repair handlers. |
| E200-27 — Persistence | C | Configure persistent journald storage and bounded retention; generate a marker, restart the container, then retrieve the prior-boot entry. Distinguish volatile logs, disk exhaustion and permission failures. |
| E200-28 — Daemons | C | Manage a network service with `systemd_service`; verify active state, socket bind address and a protocol probe from another node. Test stopped service and wrong listening interface independently. |
| E200-29 — Transfers | C | Use SSH/SFTP-backed copy/fetch or a qualified synchronize workflow; verify hashes and permissions on both endpoints. Test wrong key/path and interrupted transfer. Encrypt/limit access to evidence containing sensitive data. |

## Storage: E200-30–40

All destructive commands in this section target a recorded disposable disk image/loop device or dedicated scratch device, never the host's root disk. Containers cannot independently own the host block subsystem. Use H for authentic partition/LVM/filesystem operations, then expose only the needed test mount to a container.

| ID / topic | Class | Ansible practice and independent proof |
|---|---|---|
| E200-30 — GPT | H | Gather `lsblk`/partition metadata, assert exact scratch-device identity and absence of live mounts, then use a qualified partition module/tool to create/delete GPT entries. Re-read the table and verify boundaries, types and unrelated partitions. |
| E200-31 — PV | H | Create and remove an LVM PV on that scratch partition. Use `pvs` JSON/report output to check UUID/state; refuse removal while the PV contains allocated extents needed by a VG. |
| E200-32 — VG | H | Create/extend a lab-only VG with a qualified `lvg` workflow, inspect membership and free extents, then remove only after its LVs are safely retired. Check the wrong-device refusal path. |
| E200-33 — LV | H | Create/delete named test LVs using `lvol`, record LV/VG UUIDs and actual sizes, and ensure names cannot resolve to a production VG. Preserve and check a sentinel on unrelated LVs. |
| E200-34 — Mount identity | C/H | Configure fstab/mount units using UUID or label, compare `findmnt` with `blkid`, and verify remount/boot behavior. Introduce a bad UUID, observe dependency failure and repair without disabling all mounts. |
| E200-35 — Safe addition | H | Add a partition/LV and swap while retaining existing content. Check `swapon`, available extents and pre/post checksums. Keep global host swap management out of ordinary guest roles. |
| E200-36 — Filesystems | H | Create separate VFAT/ext4/XFS scratch filesystems, mount/write/unmount/remount them and verify data. Record filesystem-specific ownership/growth/repair differences; do not assume VFAT supports Unix metadata. |
| E200-37 — NFS | C/H | Configure a dedicated export and client `ansible.posix.mount`; verify read/write as the intended user and server-restart recovery. Qualify kernel mount permissions on H if unprivileged guests cannot perform them. |
| E200-38 — Automount | C/H | Template autofs maps and service configuration, trigger access, verify the mount appears and later expires. Test an unavailable server with bounded waits; a static fstab mount is not this exercise. |
| E200-39 — Growth | H | Extend a scratch LV and then its filesystem with the correct filesystem-specific operation. Compare block and filesystem sizes independently and verify sentinel data. Do not teach XFS shrinking as supported. |
| E200-40 — Access diagnosis | C | Inject wrong ownership, missing directory execute bit and an ACL conflict separately. Collect `namei`, mode/ACL and user groups; repair the demonstrated cause and test as the affected user. |

## Maintenance and networking: E200-41–50

| ID / topic | Class | Ansible practice and independent proof |
|---|---|---|
| E200-41 — Scheduling jobs | C | Exercise all three: one-shot `at`, recurring cron, and a systemd timer/service pair. Capture each job's marker and schedule, test timezone/missed-run assumptions, and remove jobs idempotently. Installing a timer does not cover at/cron. |
| E200-42 — Unit lifecycle | C | Use `systemd_service` for start/stop/enable/disable and daemon reload as needed. Verify both current and next-boot state; test a bad unit dependency and a handler firing only on a config change. |
| E200-43 — Default target | H | Record and set the desired systemd default target with change detection; reboot the disposable host and inspect actual target. Avoid a GUI target on minimal images unless its prerequisites are installed. |
| E200-44 — Time | H | Template chrony client configuration, validate sources/reach/offset and service startup, then break source resolution. Containers share host time; guest service success alone cannot prove independent clock synchronization. |
| E200-45 — Package sources | C | Install/update from a trusted remote repository and a verified local package artifact using pacman. Record source/version/signature and replay from a clean base. Red Hat CDN/subscription behavior remains explicitly outside this adaptation. |
| E200-46 — Boot manager | H | Detect GRUB/systemd-boot/Limine and the installed initramfs generator; template only its relevant configuration, rebuild through the correct tool and test a recoverable boot change. Keep backup/console rescue evidence. |
| E200-47 — Addresses | C | Configure IPv4 and IPv6 on named lab interfaces with one network manager. Verify address/prefix, neighbor discovery, route and cross-node traffic for both families; restart to confirm persistence. |
| E200-48 — Resolution | C | Manage hostname, hosts entries and the chosen resolver configuration. Compare local name service, DNS queries and application lookup; break search domain vs server reachability independently. |
| E200-49 — Network startup | C | Enable the chosen manager and required network service units with correct readiness dependencies. Restart the container and probe remotely; distinguish `network.target` from actual online/address readiness. |
| E200-50 — Firewall access | C/H | Qualify firewalld and its Python bindings; manage zones/interface placement and allowed services. Verify allowed and denied traffic remotely, plus runtime/permanent state after reload/restart. Coordinate host Incus firewall rules without global flushing. |

## Accounts and security: E200-51–62

| ID / topic | Class | Ansible practice and independent proof |
|---|---|---|
| E200-51 — Accounts | C | Manage account create/change/delete through `user`; inspect UID/home/shell and ownership. Test deletion policy for home/data and uniqueness. Use a non-login service account for daemon identities. |
| E200-52 — Aging | C | Set protected password hashes and explicit aging/expiry parameters supported by the selected modules/tools. Inspect aging and exercise expired/locked login behavior over SSH; avoid exposing `/etc/shadow` in artifacts. |
| E200-53 — Groups | C | Use `group`/`user` with a deliberate supplementary-membership policy. Verify new-session `id` and actual access; test that accidental membership replacement is detected. |
| E200-54 — Privilege | C | Deploy a narrowly scoped sudoers fragment with `visudo` validation. Test an allowed operation and denied unrelated command as the same user. Store any escalation secret in Vault. |
| E200-55 — Firewall state | C/H | Repeat firewall work as a security persistence drill: wrong zone, stale permanent rule and unexpected port. Read actual rules and probe from another node; repair only the owned rule and recheck after reload. |
| E200-56 — Defaults | C | Configure umask for the correct login/service context, create files/directories as that identity and inspect actual resulting modes. Distinguish umask, default ACL and systemd `UMask=`. |
| E200-57 — SSH keys | C | Distribute public keys with `authorized_key`, verify permissions and authenticated host trust, rotate/revoke an old key, and prove the revoked key fails. Keep an independent recovery channel. |
| E200-58 — SELinux mode | H | On the qualified SELinux CachyOS host, change the requested mode through supported tools/config, inspect `getenforce`, then reboot to prove persistence. Disabled SELinux is not permissive mode. |
| E200-59 — Contexts | H | Collect real file labels and process domains; correlate a denied access with the actual AVC and process identity. A fabricated log fixture can teach analysis but cannot earn live acceptance. |
| E200-60 — Relabel | H | Add a deliberate wrong label on a scratch service path, inspect expected policy mapping and restore it with `restorecon`. Verify a persistent fcontext rule when a nonstandard path is intended; do not rely only on `chcon`. |
| E200-61 — Port policy | H | Move a test service to a nonstandard port, observe the SELinux denial, manage the intended port type with a qualified seport/tool workflow, and verify service access with enforcement active. Check conflicts before changing labels. |
| E200-62 — Booleans | H | Select a documented Boolean affecting the test service, record its current/persistent value and observed access behavior, change only the needed setting and verify after reboot. Do not use blanket permissive mode as the repair. |

## Build complete exercises from the ledger

Use a small topology: controller, two admin guests, FRR pair, loghost and a dedicated host recovery target. Reuse nodes sequentially to reduce memory while retaining independent fault evidence. For example, a failed SSH login exercise must check routing, listening socket, host identity, key permissions, PAM and authorization in that order only as evidence warrants; it must not begin by replacing all SSH settings.

Every row requires a meaningful before/after assertion, a wrong-state negative control and appropriate persistence check. For declarative tasks run twice and require no unexplained changes. For one-time operations such as archive creation, reboot or filesystem repair, distinguish completion evidence from convergence. Keep host prerequisites as explicit blockers.

Useful module references: [pacman](https://docs.ansible.com/projects/ansible/latest/collections/community/general/pacman_module.html), [Flatpak](https://docs.ansible.com/projects/ansible/latest/collections/community/general/flatpak_module.html), [Flatpak remotes](https://docs.ansible.com/projects/ansible/latest/collections/community/general/flatpak_remote_module.html), [ansible.posix](https://docs.ansible.com/projects/ansible/latest/collections/ansible/posix/). Verify the pinned collection's parameters before turning these specifications into executable tasks.
