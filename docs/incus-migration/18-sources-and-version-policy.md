> Implementation authorized by the project owner on 2026-09-24. The former implementation prohibition is withdrawn. See integrations/incus for implemented behavior and validation boundaries.

# Sources, baseline and version policy

Research date: **2026-09-23**. Sources below are official project/vendor documentation or pinned source. This guide's architecture, recipes and acceptance requirements are original project proposals. A documentation citation does not establish that an Incus port or exam exercise has passed.

## Blueprint basis

| Exam | Public page used | Mapping policy |
|---|---|---|
| EX200 | [Public objectives](https://www.redhat.com/en/services/training/ex200-red-hat-certified-system-administrator-rhcsa-exam) | 62 leaf rows in published order; current software/Flatpak topics included |
| EX294 | [Public objectives](https://www.redhat.com/en/services/training/ex294-red-hat-certified-engineer-rhce-exam-red-hat-enterprise-linux) | 57 rows including eight inherited topics and four standalone managed-node tasks |
| EX457 | [Public objectives](https://www.redhat.com/en/services/training/ex457-red-hat-certified-specialist-in-ansible-network-automation-exam) | 23 leaf rows; page identifies AAP 2.6 and version-specific alternatives |
| EX342 | [Public objectives](https://www.redhat.com/en/services/training/ex342-red-hat-certified-specialist-linux-diagnostics-and-troubleshooting) | 30 leaf rows including Web console, container diagnosis, kdump and SystemTap |

The public titles of some exams have changed; the exam codes identify the requested scope. These counts do not claim access to a private booked-version syllabus. Local row IDs retain source order so a reviewer can compare every bullet without reproducing the full vendor text. The ledgers use brief topic labels and original CachyOS instructions, with explicit product differences.

## Repository sources

- [Simulator baseline](https://github.com/QuasarRay/datacenter-simulator/tree/c6aedddfbb9ac569b5e0eb68acc616452bf66136): root Ansible lab, all 27 Rust source modules, eight workflows, current/historical EX457 packs, CI and integrations.
- [patchbay fork](https://github.com/QuasarRay/patchbay/tree/3d3c577c4b49ba23cb6b14f6b48aacbec933a93e): actual namespace/lifecycle interfaces and IB integration used by the simulator.
- [petgraph fork](https://github.com/QuasarRay/petgraph/tree/a4d94bd2c39ac198c22682b2dcb1ff21583d0db0): existing graph dependency, not a new proposed package.
- `simulator/upstreams.json` and `.gitmodules`: exact NCCL, rust-ibverbs, ibsim, DeepOps, nccl-tests, NetBox and k8s-test-infra identities.

## Technical references

| Subject | Primary reference | How used |
|---|---|---|
| Incus setup | [Installation](https://linuxcontainers.org/incus/docs/main/installing/) | Host requirements and package-specific qualification |
| System images | [Image format](https://linuxcontainers.org/incus/docs/main/reference/image_format/) | Rootfs/metadata packaging and image identity |
| Network devices | [NIC reference](https://linuxcontainers.org/incus/docs/main/reference/devices_nic/) | `p2p`, host peer naming and explicit device ownership |
| API | [REST API](https://linuxcontainers.org/incus/docs/main/rest-api/) | Async operations, discovery and client boundary |
| Instance settings | [Configuration](https://linuxcontainers.org/incus/docs/main/reference/instance_options/) | Version-qualified resource/security/profile settings |
| Incus Ansible | [Connection plugin](https://docs.ansible.com/projects/ansible/latest/collections/community/general/incus_connection.html) | Existing-instance exec/file channel; not lifecycle creation |
| Navigator | [Settings](https://docs.ansible.com/projects/navigator/settings/) | Host mode versus advertised OCI engines |
| Editor | [Ansible extension](https://ansible.readthedocs.io/projects/vscode-ansible/) | Development-container qualification and editor objectives |
| Package automation | [pacman](https://docs.ansible.com/projects/ansible/latest/collections/community/general/pacman_module.html) | CachyOS adaptation of package state |
| Flatpak | [Remote module](https://docs.ansible.com/projects/ansible/latest/collections/community/general/flatpak_remote_module.html), [application module](https://docs.ansible.com/projects/ansible/latest/collections/community/general/flatpak_module.html) | Separate remote/application and user/system scope |
| Ansible failures | [Error handling](https://docs.ansible.com/projects/ansible/latest/playbook_guide/playbooks_error_handling.html) | Rescue boundaries and connection-failure handling |
| CachyOS packages | [Optimized repositories](https://wiki.cachyos.org/features/optimized_repos/) | Real distribution/ISA provenance |
| CachyOS boot | [Boot managers](https://wiki.cachyos.org/configuration/boot_manager_configuration/) | Detect installed manager rather than assuming one layout |
| NetBox | [Installation](https://netbox.readthedocs.io/en/stable/installation/) | Native service decomposition; implementation must use pinned release docs |
| Kubernetes | [Runtimes](https://kubernetes.io/docs/setup/production-environment/container-runtimes/), [cgroup v2](https://kubernetes.io/docs/concepts/architecture/cgroups/) | Incus node vs CRI workload boundary |
| Kernel dumps | [Kernel kdump documentation](https://www.kernel.org/doc/html/latest/admin-guide/kdump/kdump.html) | Real host capture-kernel requirements |
| SystemTap | [Beginners guide](https://sourceware.org/systemtap/SystemTap_Beginners_Guide/) | Compile/run/evidence scope, with exact-kernel qualification |
| SELinux | [Upstream userspace components](https://github.com/SELinuxProject/selinux/wiki/Userspace-Packages) | Required components; no unverified CachyOS package/support claim |
| Ethernet bridge | [Kernel reference](https://cdn.kernel.org/doc/html/latest/networking/bridge.html) | Link-local frame forwarding and transparent-wire limitations |
| AAP configuration | [AAP 2.6 guidance](https://docs.redhat.com/en/documentation/red_hat_ansible_automation_platform/2.6/secure-ref_initial_configuration) | Existing v6 correction identifies `infra.aap_configuration`; recheck product docs/installed role schemas before implementation |

The AAP configuration page did not load successfully during this review; the recommendation is grounded in the repository's already documented v6 correction, not claimed as a newly verified page retrieval. The ArchWiki SELinux page was inaccessible during this review; no default CachyOS SELinux support is inferred from it. Host qualification remains required.

## Lock and evidence policy

For future implementation record Incus server/client/API extensions; CachyOS kernel/boot manager/initramfs/packages/repos/ISA; image fingerprints; Rust toolchain/lockfile; patchbay/petgraph/native gitlinks; Ansible core/Navigator/Builder/collections/transport; FRR; NetBox/Python/PostgreSQL/Redis; Kubernetes/containerd/CNI/Helm/Mokka; GPU driver/CUDA/NCCL/RDMA stack; and host recovery tools/symbol builds.

Pin artifacts by hashes/revisions, not moving aliases or `latest`. Requalify compatibility when a pin changes, and keep model, compile, mock, live container, live host, physical hardware and product evidence distinct. A rolling CachyOS package update can change multiple boundaries at once; preserve the dated tested snapshot or rebuild and retest it coherently.

If a source is unavailable, record that uncertainty and inspect the pinned source/installed documentation. Do not fabricate a collection option, Incus feature, package name, image alias or vendor-support claim to fill a gap.
