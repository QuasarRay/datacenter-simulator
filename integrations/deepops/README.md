# DeepOps on the Rust datacenter simulator

This integration deploys the pinned NVIDIA DeepOps **Slurm** playbook into full Ubuntu VM guests on the simulator's patchbay/petgraph fabric. It then runs DeepOps's upstream validation and NVIDIA nccl-tests against the pinned QuasarRay NCCL library. Rust supervises external QEMU, Ansible, and test processes; it does not embed Python or implement substitute collectives.

A namespace alone shares the host filesystem and cannot safely host a DeepOps node. Each controller/compute node therefore gets a private qcow2 disk, cloud-init identity, systemd, and SSH server. QEMU receives a TAP descriptor opened in that node's namespace. Guest `simnccl` addresses are `10.253.0.x/32`; physical traffic crosses the modeled links, switch forwarding, and tc queues. A separate QEMU user-network NIC provides outbound package access. NCCL and MPI are explicitly constrained to `simnccl`.

## Run a GPU cluster

Use a dedicated Linux x86-64 provisioning machine with KVM, IOMMU, at least two NVIDIA GPUs already assigned to `vfio-pci`, approximately 24 GiB available RAM, and sufficient disk for the guest overlays and native builds. Each GPU's complete IOMMU group must belong to one VM. The command below raises only the new process's locked-memory limit so VFIO can pin guest RAM. The runner **does not unbind host GPU drivers**. The default DeepOps driver policy targets Turing or newer GPUs; the guest CUDA toolkit is 12.8. GPU passthrough must be supported by the host platform and device.

Install the provisioning tools and pinned upstreams from the repository root:

```bash
sudo apt-get update
sudo apt-get install -y qemu-system-x86 qemu-utils cloud-image-utils nftables iproute2 iputils-ping openssh-client
integrations/deepops/setup.sh
cargo build --locked --manifest-path simulator/Cargo.toml --features vm
```

`setup.sh` calls DeepOps's own setup script as a regular user; that script uses sudo to install Ansible and Galaxy dependencies on the provisioning machine. Cluster configuration changes happen inside the guests.

Obtain `ubuntu-22.04-server-cloudimg-amd64.img` from [Ubuntu's cloud image release](https://cloud-images.ubuntu.com/releases/jammy/release/), verify its published SHA-256, and copy `config.example.json` to `config.json`. Set `image`, `image_sha256`, and the real PCI function lists under `vfio`. Paths resolve relative to the config file. Use a new `state_dir` for each run. Paths cannot contain whitespace or option delimiters. The example PCI addresses and all-zero image digest are placeholders and cannot pass preflight.

```bash
simulator/target/debug/datacenter-simulator deepops-plan integrations/deepops/config.json
sudo prlimit --memlock=unlimited:unlimited simulator/target/debug/datacenter-simulator deepops-run integrations/deepops/config.json
```

The plan lists exactly the generated guest inventory, resources, source revisions, and addresses. Execution verifies the **resolved Ansible inventory** contains only those guests before any deployment playbook runs. It overrides DeepOps's default MAAS inventory and uses a private per-run SSH key and known-hosts file.

The sample topology contains a provisioning namespace, a controller VM, and two compute VMs on separate leaves through a spine. Custom manifests may have 2, 4, 8, or 16 compute guests; the power-of-two requirement makes the upstream hypercube test meaningful. CPU/memory/storage values are VM resources. Leave capacity for the host.

The Slurm profile retains actual GPU discovery, NVIDIA drivers, CUDA, Slurm, munge, OpenMPI/PMIx, and NHC. Optional NFS, containers, HPC SDK, Lmod, monitoring, and logging servers are disabled for this bounded test cluster. No synthetic GRES or fake GPU devices are configured.

## What must pass

The full run requires each stage below. Nonzero exits, deadlines, missing reports, and failed correctness checks fail the run.

1. DeepOps's 27 validation/contract unit tests and firmware parser test.
2. DeepOps `deepops_doctor.py --remote --json` reports `ok: true`.
3. The unmodified DeepOps `playbooks/slurm-cluster.yml` completes.
4. Native NCCL and upstream nccl-tests build inside the GPU guests from archives of the verified pinned commits, with `MPI=1` and explicit `NCCL_HOME`.
5. DeepOps `validate_slurm.py --json` runs on the controller and confirms all expected compute nodes, configured GPUs, and a successful real Slurm GPU job.
6. Guest identity checks, cross-fabric ping, physical-link partition failure, and recovery pass.
7. Eleven native MPI benchmarks run as Slurm jobs: `all_reduce`, `all_gather`, `broadcast`, `reduce_scatter`, `reduce`, `alltoall`, `alltoallv`, `scatter`, `gather`, `sendrecv`, and `hypercube`.

Each benchmark requests one task and one real GPU per compute VM. Tests use doubles, 256 B–1 MiB messages, five timed iterations, and **correctness enabled** (`-c 1`), with all reductions/roots where relevant. Reports must identify every distinct compute hostname and MPI rank, the expected native NCCL version, nonempty checked results, zero wrong values, no errors, and the required Socket/fabric environment. Slurm's `--export=NIL` keeps provisioning-machine environment variables out of guest jobs and upstream JSON environment dumps.

This profile validates host NCCL collectives over real TCP sockets. It does not claim GIN/device-API, communicator-resize, RDMA/GPUDirect, Kubernetes, NFS, container, or performance-fidelity coverage. The existing simulator RDMA backend remains separate. Bandwidth numbers describe the VM/emulated fabric; they are not physical datacenter predictions.

## Evidence and CI

`state_dir/reports/result.json` reports overall success **only after every full-run stage passes**. The reports directory contains upstream doctor/Slurm JSON, benchmark JSON, and process logs. Guest serial/QEMU logs live under `guests/<name>/`. Resolved inventory and deployment logs stay in `private-logs/`, because resolved variables can contain credentials. VM disks, generated SSH keys, and configuration are not uploaded by CI. QEMU guests stop when the run finishes or fails; retained disks allow diagnosis. Delete a completed run's state directory yourself when no longer needed.

`deepops.yml` runs the upstream unit tests, upstream setup and role lint, focused Molecule scenarios for facts/OpenMPI/NHC, Rust VM lint/build, and real CPU VM isolation/partition/recovery on hosted Linux runners. The CPU VM run also resolves the generated inventory and requires DeepOps's remote doctor to reach every guest. The Molecule scenarios test role convergence/idempotence; their upstream placeholder verifier assertions are not GPU evidence.

`deepops-gpu.yml` is an explicit manual hardware workflow using `[self-hosted, linux, deepops, gpu]`. Supply a runner-local config with dedicated VFIO devices and a verified image. It runs the full deployment and correctness pipeline and uploads only reports and guest boot logs. Hardware jobs do not run automatically on PR code.

For VM plumbing and DeepOps preflight, use `deepops-vm-smoke config.json` after `setup.sh`, with an empty `vfio` object, reduced guest resources, and the same verified image. KVM is preferred; CPU smoke can use TCG. Its report says `scope: vm-plumbing-only` and `gpu_tests_run: false`. This cannot satisfy the GPU workflow.

The implementation environment passed the Rust contract tests and DeepOps's 28 unit tests. It has no KVM, QEMU, or GPUs; **a successful full DeepOps/NCCL GPU deployment has not yet been demonstrated here**. Consult the PR's CI results for executed hosted checks, and require a successful `deepops-gpu` report before treating the GPU integration as validated.

Pinned source revisions are in [../../simulator/upstreams.json](../../simulator/upstreams.json) and the Git submodule entries. DeepOps, nccl-tests, and their dependencies retain their upstream licenses.
