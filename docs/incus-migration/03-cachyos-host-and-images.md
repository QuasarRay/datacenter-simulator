> Implementation authorized by the project owner on 2026-09-24. The former implementation prohibition is withdrawn. See integrations/incus for implemented behavior and validation boundaries.

# CachyOS host and reproducible system images

## Host qualification

1. Record `/etc/os-release`, kernel release/configuration, architecture/CPU ISA, boot manager, initramfs generator, cgroup mode, active LSMs, storage filesystem, free space/inodes, effective limits and current routes. A rolling release needs a dated package manifest (`pacman -Q`), repository configuration and package-cache/lock provenance.
2. Use supported CachyOS repositories and trusted signing keys. Plan a complete `pacman -Syu` transaction in a maintenance window; never teach partial upgrades as the normal workflow. Verify availability with `pacman -Si` before installing named packages. Typical host dependencies include Incus, Ansible/Python, iproute2, nftables, iputils, OpenSSH, jq, Rustup and a native toolchain. Add `dnsmasq`, storage tools, RDMA/OpenSM or GPU libraries only for profiles that need them.
3. Check the installed package's systemd units before enabling Incus; record `systemctl list-unit-files 'incus*'`. Configure non-overlapping root subordinate UID/GID ranges where required, with working `newuidmap`/`newgidmap`. Qualify unprivileged system-container startup and cgroup delegation. Do not disable host confinement globally to make a test pass.
4. Initialize a dedicated lab storage pool and project. Choose a `dir` pool for simplicity or a supported CoW pool for snapshot efficiency after measuring it. A directory pool and CoW pool do not have identical quota/snapshot behavior. Use a private profile with an explicit root disk and management NIC; do not inherit an unknown default profile.
5. Restrict Incus administration. Access to the privileged daemon can confer host-level control. Prefer the local Unix socket for the supervisor. For remote access, use TLS and scoped trust/project restrictions supported by the pinned server. Do not expose an unauthenticated daemon or mount its socket into student nodes.
6. Establish a management CIDR that does not overlap the host LAN/VPN, the existing `172.30.0.0/24` root example, EX457 addresses, NetBox services, Kubernetes service/pod CIDRs or data-link pools. Record IPv6 behavior as deliberately as IPv4. Check host firewalld/nftables integration without flushing existing rules.

The Ansible host role proposed here must gather these facts first, emit a reviewable plan and refuse an unsupported combination. It must never silently repartition a laptop disk, change a running kernel, or grant privileged/nested access to every node.

## Build a real CachyOS root filesystem

Do not assume an `images:cachyos/...` alias exists. Do not import an Arch image and merely edit `/etc/os-release`. Build a rootfs from a verified CachyOS bootstrap/install source with the actual CachyOS repository configuration, keyring, base packages and supported CPU target.

Future build procedure:

1. Allocate a disposable build root and record its ownership. Bootstrap using the CachyOS-supported repository/keyring process current at the build date. If an Arch bootstrap tool is used as a mechanism, install and verify CachyOS packages/repos through their real setup procedure. Do not mix incompatible ISA repositories or bypass signature checks.
2. Include systemd as PID 1, a POSIX shell, Python for Ansible modules, OpenSSH for normal administration, sudo, CA certificates and the selected network manager. Install FRR only in router images; PostgreSQL/NetBox tooling belongs to service images. Use host kernel modules; containers do not boot their own installed kernel.
3. Prepare one network manager per interface: systemd-networkd is a predictable lab choice, while NetworkManager can be a separate exercise. Prevent both from changing `mgmt0`, `data*` or `simnccl`. Disable unsolicited DHCP/SLAAC/default-route creation on data ports. Bake no fixed management IP into a reusable image.
4. Remove per-machine identifiers and cached leases; arrange machine-id and SSH host-key generation on first boot. Remove private keys, tokens, history, package credentials and build mounts. Preserve numeric ownership, ACLs, capabilities and xattrs in the rootfs; do not flatten them with an ordinary recursive copy.
5. Package Incus metadata and rootfs as a unified tarball or a supported split image. Metadata must include the actual architecture and creation timestamp. Hash the exact artifacts and import into the dedicated project. Record the resulting Incus fingerprint; aliases are human convenience only.
6. Boot two unprivileged test instances. Verify distinct machine IDs and host keys, PID 1/systemd health, Python, DNS/SSH, package provenance, writable persistent root, bounded cgroups and a clean shutdown/start. Resolve expected container-only systemd warnings specifically rather than suppressing every failed unit.
7. Publish an immutable base fingerprint and a manifest of installed packages, source revisions, build recipe hash and qualification evidence. Rebuild instead of mutating the published image. Keep reproducible offline inputs for a rolling distribution.

Incus import shape, illustrated only after creating and qualifying those artifacts:

```bash
incus --project dc-build image import metadata.tar.xz rootfs.tar.xz --alias cachyos-lab-qualified
incus --project dc-build image info cachyos-lab-qualified
```

The artifact names are placeholders, not files provided by this PR. Pin the returned fingerprint in the proposed plan rather than copying an example digest. Follow [Incus image format](https://linuxcontainers.org/incus/docs/main/reference/image_format/) for tar layout; the fingerprint rules differ between unified and split artifacts.

## Image families and systemd services

Use one minimal qualified base plus Ansible roles for `router`, `admin-client`, `netbox`, `database`, `loghost`, `slurm-controller`, `slurm-compute`, and `kubernetes-node`. Avoid a full desktop or every integration in every image. Minimize idle services and cap journal retention. Share immutable cached artifacts while preserving private writable roots.

For FRR, install the pinned/tested package, enable only required daemons, set integrated configuration behavior explicitly, and preserve `frr` ownership and routing capabilities. Replace the current shell entrypoint with systemd ordering: interfaces available → FRR config validated → daemon start → readiness check. Make SSH and FRR independent units so a failed routing daemon does not destroy the recovery channel.

The current v6 forced-command adapter allows Ansible `network_cli` access to vtysh. Port its exact public-key/authentication/command restrictions and re-test the chosen FRR and collection versions. A normal Linux shell on port 22 is not a drop-in FRR `network_cli` endpoint.

## Acceptance

Require cold start, restart, rebuild from cached inputs, clean clone identity, denied non-admin daemon access, missing-image failure, cgroup-limit evidence and a second Ansible convergence run. Verify the installed package list is CachyOS-based. A Docker OCI digest cannot stand in for an Incus image fingerprint.

Sources: [CachyOS optimized repositories](https://wiki.cachyos.org/features/optimized_repos/), [CachyOS boot managers](https://wiki.cachyos.org/configuration/boot_manager_configuration/), [Incus installation](https://linuxcontainers.org/incus/docs/main/installing/), [image format](https://linuxcontainers.org/incus/docs/main/reference/image_format/), [Ansible pacman](https://docs.ansible.com/projects/ansible/latest/collections/community/general/pacman_module.html). Package availability and support must be rechecked for the implementation's dated snapshot.
