"""Authored lesson bodies. tools/build_curriculum.py renders the learning chain.

Planning examples are executable; live steps deliberately require real evidence.
Never derive a product-completion claim from a successful planning example.
"""
UNITS = {}

def unit(n, skills, semantics, steps, project, incidents, symbols, code):
    if type(n) is not int or n in UNITS:
        raise ValueError('unit identity must be an integer registered exactly once')
    UNITS[n] = dict(skills=skills,semantics=semantics,steps=steps,project=project,
                    incidents=incidents,symbols=symbols,code=code.strip()+'\n')

unit(1, ['immutable intent','units and resource ownership','topology import','DeepOps provenance'],
'''Treat a deployment as a transaction with three independent inputs: the physical envelope, the fabric graph and the workload admission policy. NetBox describes intended identity and cabling; it does not prove that a cable exists. Petgraph can discover a path in that intent; it does not establish that packets traverse it. DeepOps applies reviewed host configuration; a zero exit code does not establish GPU workload correctness.

Allocate no workload permit until all three inputs refer to the same site revision. Use watts for both power and cooling limits, MiB for host memory, bytes for payloads and bits per second for links. Preserve a rejected intent as data. Do not partially mutate a shared plan and then return an error. The immutable envelope below teaches this commit boundary. It is a planning model, not an electrical commissioning procedure.''',
['Complete the linked C01-NETBOX practical: compile the live read-only NetBox scope into Incus, patchbay and generated guest networking with native_fabric. Compare IDs, ports, cable endpoints, routes and snapshot digests.',
 'Build a control node, two compute nodes, a service node and a leaf in the Incus manifest. Assign an explicit image fingerprint and resource budget. Reconcile their hostnames through the pinned DeepOps hosts role.',
 'Run the envelope example. Lower cooling below demand and predict rejection before evaluating it. Then import the manifest into Twin and request a transfer between the two compute identities.',
 'Observe the real Linux path separately. Remove one trainer-owned patchbay cable, repeat the same payload, restore the cable, and compare admission, path and workload outcomes. Repeat against a second NetBox revision.'],
 'Build a commissioning compiler that joins NetBox intent, resource envelopes and workload permits. Its deliverable is an immutable deployment bundle, a dependency DAG, a rejected-plan report and a recovery transcript. Add a second rack without changing the admission contract.',
 ['A new compute pair appears in inventory, but the acceptance workload never starts. Restore a valid deployment from conflicting intent revisions without exceeding the site envelope.',
  'The first rack is accepted; a second rack fails only after one link is withdrawn. Produce a plan that preserves ownership and proves failure-domain independence.'],
 ('Ledger: NetBox revision','Permit: workload admission','Dock: commissioned compute'),
 '''from ncp_lab import Envelope, commissioning
from ncp_lab.macros import macros, require
plan = commissioning(Envelope(gpus=8, watts=4000, rack_watts=5000, cooling_watts=4500))
require[plan.admitted]
try:
    commissioning(Envelope(8, 4000, 5000, 3500))
except ValueError:
    rejected = True
else:
    rejected = False
require[rejected]
''')

unit(2, ['rail locality','physical identity and OOB separation','optical and environmental acceptance'],
'''A rail is an identity-preserving attachment pattern, not merely another reachable switch. A logical GPU ordinal can change after a rebuild; the physical GPU, NIC PCI function, slot and switch port must still join through stable inventory identities. Keep BMC/OOB management distinct from the workload plane so a data-path failure does not also remove the recovery path.

The drawing is a cable loom: each strand has two recorded endpoints. A matching label is weaker evidence than LLDP/port identity, and both are weaker than physical inspection for a mislabelled optic. TPM state, power feeds, cooling, GPU insertion and signal quality require their own observations. A synthetic SMI record proves parser behavior only. Never energize, reseat or replace hardware from a generic lesson; follow the exact system service procedure with the authorized hardware operator.''',
 ['Join rack, slot, GPU UUID, NIC PCI address and switch port into one rail record. Reject duplicate endpoint ownership before generating native network configuration.',
 'Use OOB access to collect BMC identity and TPM/boot evidence. Compare the power and cooling envelope with the actual installed parts and site limits.',
 'Audit cables and optics at both ends: supported part number, speed, breakout mode, firmware and signal/error counters. Obtain supervised physical GPU installation evidence and correlate it with post-install SMI inventory.',
 'Run the same acceptance workload before and after the trainer moves one logical attachment in the rehearsal graph. Explain why a graph repair cannot certify a real optic, then repeat the physical rail check on the holdout.'],
 'Create a rail acceptance service for two scalable units. Generate installation records from NetBox, reconcile through DeepOps, and quarantine any endpoint whose physical and logical identities disagree. Admission consumes only a signed-off rail revision.',
 ['A healthy-looking node loses collective bandwidth after a recable. Find the inconsistent rail while preserving OOB recovery and rejecting unvalidated workload admission.',
  'After component replacement, inventory is green but one workload pair fails acceptance. Reconcile the replacement identity, physical envelope and fabric attachment.'],
 ('Loom: rail attachment map','Tag: immutable device identity','Gauge: signal and workload evidence'),
 '''from ncp_lab.macros import macros, require
rails = [("gpu-A", "nic-A", "leaf1:1"), ("gpu-B", "nic-B", "leaf1:2")]
require[len({gpu for gpu, _, _ in rails}) == len(rails)]
require[len({port for _, _, port in rails}) == len(rails)]
observed = {"nic-A": "leaf1:1", "nic-B": "leaf1:2"}
require[all(observed[nic] == port for _, nic, port in rails)]
''')

unit(3, ['artifact identity','driver and userspace compatibility','reversible software rollout','NGC and runtime diagnosis'],
'''A tag is a mutable name. A digest identifies bytes; a compatibility record states where those bytes were tested. Neither establishes that the running process loaded the expected driver or library. Record the OS, kernel, GPU driver, CUDA userspace, DOCA host/DPU components and payload digest independently. Incus guests share the host kernel: installing a guest package cannot provide an independent GPU kernel driver.

Teach Docker daemon/toolkit diagnosis and GPU execution in the course's external product station because those are explicit source objectives. The projects and incidents use native processes in Incus. Do not relabel a system container as a Docker demonstration. NGC retrieval failures can arise from credentials, DNS, TLS, routing, storage or digest mismatch; test the layer before replacing the payload. Upstream DeepOps OS support must be checked rather than forged with Ansible facts.''',
 ['Create a compatibility manifest from actual source revisions and binary/library hashes. Provision OS and packages only through a reviewed adapter for the real host distribution.',
 'Use a temporary canary to install, update and remove the GPU and DOCA userspace packages; record host driver identity separately. Preserve a rollback package set and drain before disruptive changes.',
 'Install and exercise the NGC client against a permitted payload. Verify digest, entitlement, destination capacity and network path; inject a DNS or TLS failure without changing the payload.',
 'At the external Docker course station, diagnose a daemon failure separately from toolkit/device injection, then demonstrate a real GPU result. Carry the assessor evidence forward; the Incus exercise installs no Docker daemon.'],
 'Build a digest-pinned software promotion pipeline for native Incus services. Couple artifact approval to rail compatibility and an actual workload canary. The pipeline must stop on unsupported DeepOps roles and retain the last known-good native package set.',
 ['The deployment reaches the registry but the GPU process fails after promotion. Restore a compatible native payload without changing the permitted artifact identity.',
  'One canary succeeds while the fleet rollout fails under a second network path. Resolve supply, compatibility and admission together; retain the external Docker-course evidence.'],
 ('Seal: artifact digest','Shelf: compatible driver set','Canary dock: isolated rollout'),
 '''import hashlib
from unpythonic import pipe1
from ncp_lab.macros import macros, require
payload = b"teaching artifact, not a GPU binary"
identity = pipe1(payload, hashlib.sha256, lambda h: h.hexdigest())
manifest = {"digest": identity, "os": "cachyos", "execution": "native-incus"}
require[manifest["digest"] == hashlib.sha256(payload).hexdigest()]
require[manifest["execution"] == "native-incus"]
''')

unit(4, ['authority scopes','fencing','cluster categories','service health versus workload health'],
'''A controller holds a lease to change a resource; possession of a dashboard session is not that lease. Express identity as subject, resource and permitted operation. An admission request may read several categories, but only one active owner may write a given allocation. A failed controller must be fenced before its successor acts. A timeout alone does not prove that the old writer stopped.

Mission Control, Base View and BCM have different responsibilities and require actual product observations. Build categories from hardware and workload needs; do not assume category labels enforce network isolation. Separate reported utilization, measured performance and health. A BCM service can be reachable while scheduling, node provisioning or a DPU/switch workflow remains unhealthy.''',
 ['Install/configure the licensed product in its supported course station and record exact versions. Use DeepOps for the surrounding host intent; identify each tool owner explicitly.',
 'Create least-privilege accounts and roles. Test one allowed and one denied provisioning/allocation action for each role; retain the denied API response.',
 'Build categories for physical inventory and workload purpose. Associate node, DPU and switch intent, then compare Slurm and Kubernetes-oriented allocation views with independent workload observations.',
 'Configure and verify supported BCM HA. Fence the old writer, fail the active controller and check a fresh write through the successor. Recheck Base View utilization, performance and health without treating a stale green tile as recovery.'],
 'Create a tenant provisioning gateway with a single-writer allocation ledger. The gateway reconciles categories and native Incus services, consumes physical/fabric acceptance and emits operator-readable evidence. A product adapter must be explicitly qualified before BCM completion is recorded.',
 ['After a controller handover, two teams both believe they own the same GPU rail. Restore one authoritative allocation while preserving permitted network reachability.',
  'The dashboard is healthy but new nodes receive the wrong category and workload policy. Repair identity, category and deployment without granting a global administrator role.'],
 ('Key ring: scoped role','Lease token: active writer','Fence: revoked stale authority'),
 '''from ncp_lab.macros import macros, require
permissions = {"observer": {"read"}, "provisioner": {"read", "provision"}}
require["provision" not in permissions["observer"]]
epoch, lease_epoch, fenced = 8, 8, True
successor_may_write = fenced and lease_epoch == epoch
require[successor_may_write]
require[not (fenced and 7 == epoch)]
''')

unit(5, ['BGP and EVPN intent','tenant isolation','NVUE generation','idempotent network delivery'],
'''A route advertises reachability; a tenant boundary constrains who may use that reachability. Keep underlay adjacency, overlay membership and workload authorization separate. An established BGP session does not prove correct EVPN route import/export. A permitted route target is not a substitute for testing forbidden traffic.

Compile tenant declarations into reviewed NVUE/Ansible intent, then compare intended and observed state. The simulator graph models topology, and Linux/FRR can exercise selected routing behaviors. Neither implements every Spectrum ASIC or NVUE behavior. Teach idempotence as a postcondition: the second reconciliation produces no new change, while an unauthorized flow remains denied after both runs.''',
 ['Declare two tenants with distinct VNI, subnet and route-target identities; check collisions as a pure function before rendering NVUE templates.',
 'Apply the native Linux/FRR rehearsal through DeepOps-backed automation. On the product station, apply the corresponding NVUE and Ansible VLAN/RoCE intent and read back its effective state.',
 'Probe same-tenant reachability and cross-tenant denial from independent endpoints. Capture BGP/EVPN route and neighbor evidence alongside the workload request ID.',
 'Withdraw one path and repeat both permitted and forbidden probes. Reapply identical intent and compare the change set; a no-op must not erase the isolation test.'],
 'Build a two-rack tenant fabric compiler with staged rollout and rollback. Combine C02 rail identities, C04 authority and C05 overlay intent; admit workloads only when the physical path and negative isolation probes agree.',
 ['A tenant job works on one rack and leaks reachability to a second tenant after failover. Restore correct routing and admission without flattening the fabric.',
  'A generated switch change reports success but the second application changes it again. Resolve configuration drift while preserving tenant isolation and GPU workload service.'],
 ('Canal: underlay path','Lock gate: tenant boundary','Permit book: route-target intent'),
 '''from unpythonic import pipe1
from ncp_lab.macros import macros, require
tenants = [{"name":"red","vni":10001,"rt":"65000:1"}, {"name":"blue","vni":10002,"rt":"65000:2"}]
vnis = pipe1(tenants, lambda rows: [r["vni"] for r in rows])
require[len(vnis) == len(set(vnis))]
imports = {row["name"]: {row["rt"]} for row in tenants}
require[imports["red"].isdisjoint(imports["blue"])]
''')

unit(6, ['queue units','ECN and PFC distinction','bounded headroom','congestion experiments'],
'''ECN asks a congestion-control loop to reduce offered load. PFC pauses a selected priority at a link; it is not end-to-end congestion control. A configuration that prevents one drop can still create pause propagation, unfairness or deadlock. Link utilization alone cannot distinguish useful throughput from stalled work.

The reservoir drawing represents queue occupancy, not a literal shared ASIC implementation. The headroom example estimates bytes arriving while a sender reacts plus one in-flight frame. Real allocation also depends on the exact ASIC, cable delay, MTU, pipeline and buffer policy. Spectrum-X adaptive routing and SuperNIC behavior require product measurements; tc netem provides delay/rate/fault emulation and must never be reported as PFC or ECN ASIC validation.''',
 ['Fix workload placement and collect a baseline with offered load, useful throughput, completion latency, queue/mark/drop counters and pause durations.',
 'Calculate the illustrative headroom lower bound in bytes. Compare it with the documented allocation rules for the actual switch; record the difference rather than rounding toward a desired answer.',
 'Apply QoS, ECN, PFC and adaptive-routing changes one at a time through versioned automation. Observe which counters should change and which workload results must remain correct.',
 'Introduce an incast and a path loss. Confirm that congestion recovery preserves progress for other priorities and tenants. Roll back a policy that avoids drops by stopping useful work.'],
 'Create a congestion policy compiler that emits both switch configuration and an experiment matrix. Combine tenant isolation, GPU placement and job admission; promote only policies that meet useful-work and recovery objectives across independent incasts.',
 ['An incast reports no packet loss but training progress stops. Restore useful work while keeping the tenant and physical resource bounds intact.',
  'A larger MTU improves one run and destabilizes a second failure domain. Reconcile queue policy, placement and admission using independent congestion observations.'],
 ('Reservoir: queue bytes','Warning flag: ECN feedback','Sluice: priority pause'),
 '''from ncp_lab import headroom_bytes
from ncp_lab.macros import macros, require
minimum = headroom_bytes(400_000_000_000, 1000, 9216)
require[minimum == 59216]
require[headroom_bytes(400_000_000_000, 2000, 9216) > minimum]
# This lower bound is not a Spectrum buffer configuration.
''')

unit(7, ['InfiniBand membership','subnet management and HA','diagnostic evidence selection'],
'''InfiniBand reachability requires more than an active physical link. The subnet manager supplies fabric configuration; partition membership restricts communication. A PKey combines a partition identity with a membership bit. Limited members of the same partition cannot communicate directly with each other; at least one endpoint must be a full member. Do not infer tenant isolation from equal low bits alone.

Keep fabric-management simulation, verbs execution and UFM product evidence separate. ibsim can exercise management behavior; the existing software Fabric checks memory regions and completion rules; native rust-ibverbs requires real supported devices. Latency and bandwidth tools answer different questions. A successful ibping cannot certify RDMA bandwidth or workload completion.''',
 ['Provision a reviewed fabric and its supported management HA configuration. Record active/standby manager identities and test takeover without competing active authorities.',
 'Generate PKey membership from tenant intent. Test full/full, full/limited, limited/limited and different-partition cases before admitting the workload.',
 'Collect ibstat, ibnodes and iblinkinfo for identities/state, ibdiagnet for diagnostics, ibping for a management reachability probe, and ib_write_lat/ib_write_bw for distinct data-path measurements. Use Python subprocess arrays only behind an evidence adapter.',
 'Change QoS and adaptive routing with a fixed placement; observe UFM link-state/bandwidth and the workload result. Repeat after a manager or link failover and verify forbidden membership remains forbidden.'],
 'Build a tenant-aware IB acceptance pipeline that joins physical port identity, PKey policy, UFM state and Slurm admission. Use ibsim for preflight, then require independent native measurements before promotion.',
 ['Two healthy endpoints in the same named partition cannot exchange the admitted workload. Restore membership and management consistency without granting every tenant full access.',
  'A manager failover restores link status but one training job loses bandwidth and another gains unauthorized reachability. Repair the joint policy and validate the holdout.'],
 ('Harbor master: subnet manager','Colored key: PKey membership','Dock gauge: verbs result'),
 '''from ncp_lab import pkey_allows
from ncp_lab.macros import macros, require
require[pkey_allows(0x8001, 0x0001)]
require[not pkey_allows(0x0001, 0x0001)]
require[not pkey_allows(0x8001, 0x8002)]
''')

unit(8, ['host and DPU authority','DOCA compatibility','packet ownership','SuperNIC congestion evidence'],
'''The host CPU, BlueField Arm subsystem and NIC datapath are distinct execution and authority domains. A command issued on the host does not imply that an Arm service was updated. A packet crossing a representor does not by itself prove the expected offload. State the DPU mode and supported ownership model before changing host/DPU software.

Treat the DPU as a customs station with two control desks and a forwarding lane. The picture must not imply that every packet traverses Arm userspace. Record firmware, host driver, DPU OS/DOCA and service revisions separately. A rollback must restore a compatible set, not just one package. Network isolation and workload admission must agree about who owns the offloaded resources.''',
 ['Inventory host PCI identity, BlueField generation, DPU mode and firmware. Establish an independent management path and recovery procedure before applying a BFB or disruptive configuration.',
 'Install matching supported DOCA components on host and DPU through reviewed automation. Deploy an actual Arm service and capture its process identity and lifecycle.',
 'Observe a permitted and a denied packet path using representor/counter evidence appropriate to the selected mode. Distinguish software forwarding from hardware offload.',
 'Configure SuperNIC packet-processing and congestion behavior on the supported product. Correlate its observations with host GPU workload progress, then test host-service and DPU-service failures separately.'],
 'Build a versioned DPU service rollout across two failure domains. Combine supply-chain approval, fabric isolation and GPU job draining. The service canary must prove both packet behavior and a real workload result before fleet promotion.',
 ['The host reports the expected package version but tenant traffic and GPU jobs diverge after a DPU update. Restore a compatible, correctly owned path.',
  'An Arm service restart recovers management access while packet processing remains wrong. Repair the data path without transferring uncontrolled authority to the host.'],
 ('Customs desk: host control','Second desk: Arm control','Fast lane: NIC datapath'),
 '''from ncp_lab.macros import macros, require
release = {"host":"h2", "dpu":"d2", "firmware":"f2"}
qualified = {("h1","d1","f1"), ("h2","d2","f2")}
require[tuple(release[k] for k in ("host","dpu","firmware")) in qualified]
mixed = ("h2","d1","f2")
require[mixed not in qualified]
''')

unit(9, ['API reconciliation','synthetic worker isolation','operator compatibility','execution evidence'],
'''The API records desired and observed objects. A scheduler selects placement; a controller reconciles objects; a worker executes the payload. Rusternetes supplies the selected control plane. KWOK simulates worker lifecycle and status. A KWOK Running pod is not a CUDA process, and creating a GPU resource field does not allocate a physical GPU.

Use a dispatch board for API state and a workshop floor for native Incus processes. Keep the arrow between them explicit: the native workload adapter, not KWOK, launches the process. NVIDIA Network Operator and BCM Kubernetes bootstrap objectives still require original-product course evidence. Test discovery, watches, status subresources, RBAC, CRDs and admission compatibility before claiming an operator runs on Rusternetes. Unsupported behavior is blocked, not silently replaced by success.''',
 ['Use DeepOps-backed host reconciliation, then start pinned etcd and Rusternetes services natively inside the control container. Select its TLS kubeconfig explicitly through kr8s.',
 'Generate tainted synthetic worker objects with integrations.incus.services.worker_objects. Create them with ncp_lab.cluster.Cluster; test that a normal workload cannot accidentally be treated as executed by KWOK.',
 'Probe API create/read/watch/update/delete and RBAC denial. In the product course station, install/initialize the supported cluster and deploy Network Operator for RDMA/InfiniBand; inspect reconciliation and device/network evidence.',
 'Deploy inference as a native Incus service for the project route. Compare API status, process identity and actual response separately; kill the process while leaving a synthetic Running object and require detection.'],
 'Build a Rusternetes scheduling rehearsal linked to a native Incus inference service. Export API scale tests to KWOK, while physical service acceptance consumes real process/GPU/network observations. Maintain a compatibility report for every NVIDIA operator feature used.',
 ['The API reports Running while the inference endpoint produces no valid result. Restore execution and admission using independent physical and path evidence.',
  'A worker scale-out succeeds but an operator reconciliation stalls after an API update. Diagnose compatibility and preserve isolation; do not count synthetic nodes as physical capacity.'],
 ('Dispatch board: API objects','Paper worker: KWOK state','Workshop floor: native process'),
 '''from ncp_lab.macros import macros, require
nodes = [{"name":"w0","synthetic":True,"advertised_gpu":8},
         {"name":"incus-compute-a","synthetic":False,"observed_gpu":1}]
physical = sum(n.get("observed_gpu",0) for n in nodes if not n["synthetic"])
require[physical == 1]
require[physical != sum(n.get("advertised_gpu",0) for n in nodes)]
''')

unit(10, ['Slurm service lifecycle','GRES locality','training launch','Enroot and Pyxis boundaries'],
'''A Slurm allocation grants scheduler resources; it does not prove that the selected device, NIC and storage path are usable. GRES identity must correspond to the actual visible GPU or MIG device. Node categories and interfaces constrain placement, while partitions and accounts constrain admission. Preserve their distinct meanings in generated configuration.

Use a loading permit for allocation and a crane for execution. Enroot prepares a userspace environment and Pyxis integrates it with Slurm; neither replaces the host GPU driver or creates a new physical GPU. Real DeepOps Slurm roles must be run on a supported or qualified adapted OS. The current CachyOS role port remains a release gate; do not overwrite distribution facts to make upstream tasks appear supported.''',
 ['Inspect the pinned DeepOps Slurm playbook and role inputs. Build a supported course station or qualify the native OS adapter; record installation, controller/worker services and their actual revisions.',
 'Generate categories, interfaces and GRES from stable physical inventory. Submit one allowed and one rejected allocation using the scheduler API or a narrow Python adapter.',
 'Deploy a small real training payload through Slurm. When Enroot/Pyxis is used, record the payload identity, mount/device visibility and process ancestry independently of the allocation response.',
 'Drain one rail, resubmit under a new placement and verify no job escapes its resource grant. Correlate completion with the actual network and storage paths, then recover the drained capacity.'],
 'Build a code-generated Slurm training service over the native Incus lab. Join scheduler accounts, GPU/NIC locality and dataset access; retain a qualified Enroot/Pyxis path where applicable without installing Docker.',
 ['An allocation is granted but training stalls only on one node category. Repair physical/GRES identity, fabric locality and job policy together.',
  'A drained rail still receives a job after configuration regeneration. Restore safe scheduling and prove that both accepted and rejected submissions behave correctly.'],
 ('Loading permit: allocation','Crane: task execution','Berth: GPU and NIC locality'),
 '''from ncp_lab.macros import macros, require
requests = {"team-a": 2, "team-b": 2}
allocations = {"team-a": 2, "team-b": 2}
physical = 4
require[sum(allocations.values()) <= physical]
require[all(0 < n <= allocations[t] for t,n in requests.items())]
drained = {"gpu-3"}
placement = {"gpu-0", "gpu-1"}
require[placement.isdisjoint(drained)]
''')

unit(11, ['cross-scheduler accounting','fairness and preemption','Run:ai service lifecycle','inference versus training SLOs'],
'''A team budget must have one authoritative owner even when several schedulers expose it. Adding the same physical GPU to Slurm and a Kubernetes-oriented scheduler does not double capacity. Distinguish a quota, a reservation, a running allocation and observed utilization. A borrowed idle resource may need to be returned before a higher-priority request starts.

The shared ledger is a scale with conserved weights. It is an accounting analogy: it does not assert that all Run:ai allocation policies are equivalent to simple integer division. Training favors sustained throughput; inference may require a tail-latency objective. Preemption is a workload transition with checkpoint and recovery consequences, not only a subtraction from a counter.''',
 ['Install and administer the supported Run:ai product in the course station. Record identity, project policy and the actual inference/training deployment paths.',
 'Create a common physical resource ledger and separate scheduler views. Generate team policy for Run:ai, Slurm and Kubernetes-oriented admission; prevent simultaneous ownership of the same device.',
 'Measure inference latency and training progress before borrowing idle quota. Preempt through the supported API, verify checkpoint/restart behavior and observe actual resource release.',
 'Repeat under a fabric bottleneck. Explain why a quota increase cannot cure a congested path, and show that a failed preemption does not manufacture free capacity in the ledger.'],
 'Build a two-service GPU tenancy controller with an inference SLO and a checkpointable training queue. Integrate C04 authority, C06 congestion, C10 scheduling and the shared quota ledger. Rusternetes + KWOK rehearse scale; native Incus processes establish actual execution.',
 ['Both schedulers report spare capacity, but a latency-sensitive service is evicted during training admission. Restore a single resource accounting and valid network placement.',
  'A preempted job remains active after the ledger reallocates its devices. Recover ownership, fabric isolation and service objectives without overcommitting physical GPUs.'],
 ('Scale: conserved allocation','Borrowing ticket: revocable quota','Checkpoint crate: restart state'),
 '''from ncp_lab.macros import macros, require
ownership = {"gpu0":"inference", "gpu1":"training"}
slurm = {"gpu1"}
rusternetes = {"gpu0"}
require[slurm.isdisjoint(rusternetes)]
require[slurm | rusternetes == set(ownership)]
running = {"gpu0", "gpu1"}
require["gpu1" in running]  # A requested preemption has not yet freed it.
''')

unit(12, ['MIG profile placement','GI and CI identity','drain and reconfiguration','AI and HPC acceptance'],
'''A MIG profile is a supported partition shape on a particular GPU model. GPU instances partition relevant device resources; compute instances subdivide compute within a GPU instance. Memory-size arithmetic alone does not establish a legal layout. Consult the model-specific profile and placement catalogue, and record the generated GI/CI identities after each change.

Use a machined tray for placement: enough total area does not imply that every shape fits every slot. This is a geometry analogy, not an assertion that all resources have identical boundaries. Drain active users before disruptive reconfiguration and refresh scheduler/device inventory afterward. Test both AI and HPC workload behavior; do not assume a profile appropriate for inference preserves a collective application's requirements.''',
 ['Read the physical GPU model, supported profiles and placement options from actual product tooling. Save that catalogue with the driver revision; never reuse an A100 catalogue for an unrelated model.',
 'Compile requested partitions against the catalogue. Reject an unsupported arrangement even when the sum of advertised memory or slices appears to fit.',
 'Drain affected allocations and confirm users have released the device. Apply the supported MIG transition, rediscover GI/CI identifiers and regenerate scheduler resource advertisements.',
 'Run AI and HPC acceptance workloads over the intended NIC/storage path. Compare isolation and performance, then restore the original layout and reject a stale pre-change device identity.'],
 'Build a MIG reconfiguration controller with a two-phase drain/apply protocol. It must bind the placement catalogue to GPU identity, coordinate fabric placement and resume only after scheduler inventory and native workload evidence agree.',
 ['A partition plan fits the memory total but one requested instance cannot be created. Repair layout, network placement and admission without inventing capacity.',
  'Reconfiguration succeeds yet a resumed job references a retired instance. Restore consistent GI/CI identity and prove AI/HPC acceptance on the new layout.'],
 ('Machined tray: supported placement','Slot tag: GI and CI identity','Empty-work lock: drained device'),
 '''from ncp_lab import mig_fits
from ncp_lab.macros import macros, require
# Illustrative symbols only; replace with the observed model-specific catalogue.
catalogue = [("small@0","small@1"), ("large@0",)]
require[mig_fits(("small@0","small@1"), catalogue)]
require[not mig_fits(("small@1","small@0"), catalogue)]
''')

unit(13, ['storage sizing','buffer lifetime','GPUDirect and fallback','durability and integrity'],
'''A storage read has at least four separate obligations: locate the intended bytes, transport them through an authorized path, place them in a live destination buffer and establish completion before consuming them. Completion of a transfer does not automatically mean application consumption or durable storage. Keep the owner of a registered buffer alive until the relevant completion rule permits release.

GPUDirect Storage can avoid a CPU bounce buffer on a supported path. Fallback can still return correct bytes while changing CPU use and throughput. Diagnose correctness first, then prove which path executed. The existing simulator Fabric models registered-memory lifetime and transfers; it does not implement a filesystem or certify GPUDirect Storage. Third-party storage tuning must use measured workload distributions and the vendor's supported parameters.''',
 ['Compute working-set, checkpoint, retention and recovery capacity from explicit units. Install/configure the actual storage service using its supported third-party parameters and DeepOps-managed host mounts.',
 'Run sequential, random and concurrent acceptance loads with recorded block sizes, queue depth, dataset identity and cache conditions. Compare latency distributions and sustained throughput rather than one warm-cache sample.',
 'Trace Magnum IO/GDS path selection, buffer registration and completion. Introduce a controlled unsupported alignment or path condition and observe the documented failure/fallback behavior without labelling it direct DMA.',
 'Corrupt or truncate a test object, reject its checksum, restore it and verify a real GPU consumer. Repeat with a slower network path and distinguish storage, transport and compute bottlenecks.'],
 'Build a versioned dataset and checkpoint service for training. Combine C03 artifact identity, C05 isolation, C10 admission and C13 buffer/durability rules. Expose an explicit recovery-point contract and verify it under interrupted writes.',
 ['GPU utilization falls while storage throughput looks healthy. Restore the actual data path and admission policy using cache-aware measurements and valid buffer lifetimes.',
  'A recovered training job reads a complete-length checkpoint with wrong contents. Restore integrity, path identity and workload state without accepting a stale artifact.'],
 ('Sealed crate: registered buffer','Warehouse: durable object','Receipt: transfer completion'),
 '''import hashlib
from ncp_lab import RouteBudget
from ncp_lab.macros import macros, require
checkpoint = b"epoch=12;weights=example"
expected = hashlib.sha256(checkpoint).hexdigest()
require[hashlib.sha256(checkpoint[:-1]).hexdigest() != expected]
budget = RouteBudget(100_000_000_000, 1000)
require[budget.optimistic_ns(1_000_000) == 81_000]
# Cache, queueing, storage and GPU time are additional measured costs.
''')

unit(14, ['collective shape and rank mapping','intra-node and inter-node fabrics','native NCCL evidence','Fabric Manager lifecycle'],
'''A collective is a contract over ranks, buffers, element counts and an operation. Every rank must participate consistently. A correct mathematical result does not identify the transport used, and an attractive bandwidth number does not prove the result is correct. Distinguish intra-node NVLink/NVSwitch paths from inter-node NIC/switch paths and host/storage paths.

The diagram is a set of loading cranes inside a dock and shipping lanes between docks. NVSwitch is not an Ethernet leaf, and Fabric Manager is not a generic Linux bridge manager. The existing simulator deliberately delegates collective arithmetic to native NCCL. It validates shapes and device assignments first, and native execution records library identities and real completion time; use that path rather than writing a Python sum and calling it NCCL.''',
 ['Create a rank-to-node-to-GPU-to-NIC map. Validate equal element counts and supported reduce-scatter divisibility; reject duplicate device assignments before launching.',
 'Inspect the actual NVLink/NVSwitch topology and supported Fabric Manager service state. Run a local collective and correlate its correctness and transport evidence with that topology.',
 'Use the simulator native NCCL backend with pinned CUDA/NCCL libraries for a namespace-separated rehearsal on the host. Physical inter-node acceptance additionally requires actual separate hosts and a qualified distributed launch adapter. Record rank-local completion time, aggregate network counters and result values; do not attribute all interface bytes exclusively to NCCL.',
 'Inject one local-fabric service failure and one inter-node path failure in separate authorized trials. Recover each, repeat on a different rank placement and explain GPU/CPU/storage interconnect latency independently.'],
 'Build a collective acceptance service that selects local and remote tests from the rank graph. Join physical rail identity, scheduler admission and transport observations; reject a numerically correct result collected on the wrong execution path.',
 ['A local collective passes while the same ranks split across nodes time out. Recover placement, service state and fabric path with independent evidence.',
  'A benchmark reports high bandwidth but one rank returns a wrong result after failover. Restore collective correctness and prove the intended transport on the holdout.'],
 ('Dock cranes: local GPU fabric','Shipping lane: inter-node NIC path','Manifest: rank and buffer shape'),
 '''from ncp_lab.macros import macros, require
ranks = [("node-a",0),("node-b",0)]
inputs = [[1.0,2.0],[3.0,4.0]]
require[len(ranks) == len(inputs)]
require[len({tuple(x) for x in ranks}) == len(ranks)]
require[len({len(x) for x in inputs}) == 1]
# Shape validation only. Actual reduction is delegated to native NCCL.
''')

unit(15, ['acceptance thresholds','soak experiments','thermal and error trends','workload-specific validation'],
'''Acceptance is a predeclared predicate over a measurement window, not the best sample in that window. Define correctness, duration, concurrency, thermal envelope, permitted errors and recovery behavior before a soak begins. HPL, NCCL, ClusterKit, NeMo and node stress test different behavior; one cannot stand in for all the others.

Use a test kiln: sustained heat reveals a defect that a quick inspection misses. The analogy stops at the actual cooling and device limits. A monotonic timestamp bounds elapsed time; aligned device and network records allow diagnosis. Keep the workload identity and environmental conditions fixed when comparing runs. A tool that exits successfully while skipping a GPU is not an accepted GPU test.''',
 ['Write thresholds and abort conditions before collecting data. Verify the physical installation, cooling and network path with an initial hardware acceptance workload.',
 'Execute node stress, HPL and ClusterKit separately. Record actual participating devices, correctness checks, duration, temperatures, throttling and errors.',
 'Run NCCL, HPL and NeMo burn-in workloads with a justified duration and concurrency. Collect time-series observations across compute, storage and fabric; keep every failed interval.',
 'Introduce a bounded path impairment and a recoverable service interruption. Confirm both the steady-state objective and recovery bound on a second workload placement. Stop safely when a declared abort condition holds.'],
 'Build an acceptance pipeline for a newly commissioned scalable unit. Generate the full benchmark matrix from inventory, enforce abort/recovery contracts and publish a traceable release report that cannot omit a failed workload family.',
 ['A short benchmark passes but training slows after sustained load. Identify the interacting physical, fabric and operational cause without relaxing the acceptance window.',
  'A release report is green although one benchmark skipped a device. Restore complete participation and prove all workload families against the declared thresholds.'],
 ('Kiln: sustained load','Thermometer: bounded operating envelope','Test card: declared acceptance window'),
 '''from ncp_lab.macros import macros, require
samples = [{"second":i,"errors":0,"participants":4,"progress":10+i} for i in range(5)]
require[all(s["participants"] == 4 and s["errors"] == 0 for s in samples)]
require[all(a["progress"] < b["progress"] for a,b in zip(samples,samples[1:]))]
require[samples[-1]["second"]-samples[0]["second"] == 4]
# Five model samples are not a hardware soak.
''')

unit(16, ['causal diagnosis','independent clocks and provenance','DCGM NVSM and SMI','WJH NetQ and telemetry'],
'''A correlated symptom is a candidate explanation. A causal diagnosis also predicts a discriminating intervention. Align device, fabric and job observations by resource identity and time uncertainty; do not join unrelated samples merely because their timestamps print the same second. Preserve raw observations alongside normalized fields.

Use three inspection windows: compute health, transport behavior and admitted workload progress. DCGM, NVSM and SMI observe different aspects of the node. WJH and in-band telemetry concern specific network observations, while NetQ provides network visibility. None is interchangeable with a made-up JSON field carrying the product name. Localize GPU, fan and NIC faults separately, and distinguish utilization from useful performance.''',
 ['Collect BCM usage/performance/incident reports and raw DCGM, NVSM and SMI observations with identities and collection intervals. Explain disagreements instead of averaging them away.',
 'Collect WJH drop reasons, in-band telemetry and NetQ congestion/latency evidence on a supported product. Join them to a specific job and path, preserving clock uncertainty.',
 'Form two competing hypotheses for a GPU, fan or NIC incident. Choose one bounded intervention whose predicted observations differ between the hypotheses.',
 'Apply that intervention, verify workload recovery and repeat on a holdout fault location. Produce an incident report that cites evidence and states what remains unobserved.'],
 'Build an incident evidence graph across job IDs, physical devices, fabric ports and time intervals. Produce a minimal discriminating probe plan and an auditable recovery narrative, using all prior course measurement methods.',
 ['SMI utilization is high, a network dashboard is green and the job makes no progress. Determine which observations are stale or irrelevant and recover the coupled workload.',
  'Two alarms point at different devices after a clock adjustment. Reconstruct identity and time relationships, then prove the root-cause intervention on a second fault.'],
 ('Inspection window: independent observer','Thread: resource identity','Time ruler: uncertainty interval'),
 '''from ncp_lab.macros import macros, require
def overlap(a,b): return max(a[0],b[0]) <= min(a[1],b[1])
gpu = {"job":"j7","window":(100,104)}
nic = {"job":"j7","window":(103,107)}
unrelated = {"job":"j8","window":(100,101)}
require[gpu["job"] == nic["job"] and overlap(gpu["window"],nic["window"])]
require[not (gpu["job"] == unrelated["job"] and overlap(gpu["window"],unrelated["window"]))]
''')

unit(17, ['failure-domain maintenance','firmware compatibility','physical replacement','canary and rollback'],
'''Maintenance acquires an exclusive right to disrupt a bounded failure domain. Drain and verify release before changing firmware or hardware. Keep the recovery path outside the domain being changed. A service restart, image update, switch firmware transition, BlueField update and optic firmware update have different compatibility and rollback rules.

Use a lockout tag on the dock. The tag represents an enforced maintenance state, not a substitute for physical safety procedures. HGX firmware faults, GPU/card/PSU isolation and replacement must follow the exact system procedure and authorized service personnel. Reconcile replacement identities before readmission. A successful flash command is not proof that the new component booted, passed validation or preserved redundant paths.''',
 ['Construct a version compatibility graph for BCM images/patches, HGX, switches, BlueField-3 and optics. Record upgrade order, supported rollback and recovery access for each component.',
 'Acquire the maintenance lease, drain workloads and verify that surviving paths and capacity satisfy the service objective. Fence stale writers before proceeding.',
 'Perform the supported canary update or supervised card/GPU/PSU isolation and replacement. Collect physical identity and firmware/software observations after the transition; detect an intentionally failed canary.',
 'Run physical, fabric and workload acceptance before readmission. Roll back only by the supported procedure and prove that a second failure-domain maintenance cannot violate the capacity bound.'],
 'Build a rolling maintenance controller for two redundant scalable units. Its state machine coordinates physical service, network redundancy and scheduler drains; every release of the maintenance lease requires new acceptance evidence.',
 ['A canary firmware update completes but redundant workload capacity disappears. Recover the correct failure domain without rolling the fault across the fleet.',
  'A replacement part is visible but an old identity remains allocated. Reconcile hardware, network intent and workload state before ending maintenance.'],
 ('Lockout tag: exclusive maintenance','Spare dock: surviving capacity','Inspection seal: readmission evidence'),
 '''from ncp_lab.macros import macros, require
domains = {"rack-a":8,"rack-b":8}
required = 8
def may_drain(names): return sum(n for k,n in domains.items() if k not in names) >= required
require[may_drain({"rack-a"})]
require[not may_drain({"rack-a","rack-b"})]
states = ["admitted","draining","released","maintaining","validated","admitted"]
require[states.index("released") < states.index("maintaining")]
''')

unit(18, ['finite ASIC capacity','CPU NUMA affinity','AMD and Intel host tuning','capacity forecasting'],
'''A switch has finite forwarding resources, and a host has a finite locality budget. Count the resource actually consumed: routes, neighbors, next hops and other ASIC tables can exhaust independently. On the host, CPU affinity, memory placement, PCIe topology and NIC/GPU attachment interact. Increasing thread count can increase remote memory traffic and reduce useful throughput.

The drawing is a warehouse with shelves and loading aisles. A free shelf does not imply a free aisle. Use cl-resource-query on the supported platform for ASIC evidence. Tune AMD and Intel hosts using their actual topology and supported settings; do not copy a NUMA node number between machines. The experiment must bind physical identity, fabric pressure and job placement.''',
 ['Collect the switch resource report and relate used/available tables to the actual route/neighbor scale. Keep a reserve for failover expansion and reject a forecast that assumes unlimited ASIC entries.',
 'Collect CPU, NUMA, PCIe, GPU and NIC locality on both AMD and Intel course hosts. Generate an affinity plan from observed topology rather than fixed CPU numbers.',
 'Measure a workload with local and intentionally remote placement under the same network/storage conditions. Change one tuning parameter at a time and retain a correctness test.',
 'Scale tenant routes and job concurrency together, then remove one path. Validate that ASIC reserve and host affinity still satisfy the service objective; compare the holdout hardware topology.'],
 'Build a capacity forecast and placement compiler for heterogeneous CPU hosts. Integrate ASIC table reserves, GPU/NIC locality, storage demand and scheduler policy; explain the limiting resource for each admitted plan.',
 ['Adding a tenant causes intermittent failures despite spare link bandwidth. Restore finite fabric resources and locality-aware admission.',
  'A tuning profile improves an AMD host but degrades an Intel host with the same GPU count. Rebuild the placement from observed topology and prove the coupled workload objective.'],
 ('Shelf: ASIC table capacity','Aisle: NUMA and PCIe path','Reservation: failover reserve'),
 '''from ncp_lab.macros import macros, require
tables = {"routes":(900,1200,200), "neighbors":(400,800,100)}
require[all(used+reserve <= total for used,total,reserve in tables.values())]
locality = {"gpu0":0,"nic0":0,"cpu-worker":0}
require[len(set(locality.values())) == 1]
''')

unit(19, ['edge reconciliation','offline identity','cloud VMI validation','reconnect conflict handling'],
'''An edge node can keep running after its controller loses contact. Reconnection must reconcile two histories without replaying obsolete authority or overwriting newer workload state. Give desired configuration a revision and bind artifacts to identities. Separate a disconnected-but-healthy service from a reachable-but-stale controller report.

Fleet Command and cloud VMI remain source objectives from the AIO guide and require the real product/cloud course station. The Incus project models an edge lifecycle and exercises actual Linux network disconnection; it does not become Fleet Command by naming a function after it. A cloud GPU image still requires driver, network, storage and workload validation on its actual instance type.''',
 ['Administer a supported Fleet Command environment and record permitted device/service lifecycle operations. At the cloud station, deploy the intended VMI with explicit instance, image and network identity.',
 'Validate GPU, NIC and storage behavior before admitting the remote workload. Use DeepOps-managed host configuration where supported, recording any platform-specific adapter.',
 'Disconnect the rehearsal edge via an owned patchbay path while the native service continues. Queue versioned desired state and preserve the service checkpoint separately.',
 'Reconnect with competing old and new revisions. Reject stale desired state, recover the authorized workload and repeat with a rotated endpoint identity and different path latency.'],
 'Build an intermittently connected inference outpost with a revisioned control journal. Combine artifact pinning, network isolation, quota ownership and checkpoint integrity; qualify the real Fleet Command/cloud adapter separately.',
 ['An edge site reconnects and reverts to an obsolete image while the control plane reports success. Restore identity, network and deployment consistency.',
  'A cloud VMI is healthy in isolation but fails after reconnecting to the data service. Recover the correct path and resource envelope without accepting stale control evidence.'],
 ('Outpost: disconnected service','Sealed journal: desired revisions','Ferry route: reconnect path'),
 '''from ncp_lab.macros import macros, require
applied = 12
pending = [9,12,13,11]
accepted = [revision for revision in pending if revision > applied]
require[accepted == [13]]
identity = ("edge-a","gpu-uuid-a","image-digest-a")
require[identity != ("edge-a","gpu-uuid-b","image-digest-a")]
''')

unit(20, ['recovery dependency graphs','split-brain exclusion','end-to-end rollback','novel incident synthesis'],
'''Recovery is a new controlled transition, not an unconditional rewind. A checkpoint can restore application state while leaving a stale controller, revoked credential or replaced physical identity outside that checkpoint. Restore dependencies in a valid order and reject any operation whose ownership epoch no longer matches.

Use a salvage yard with numbered crates and a single lifting permit. Two cranes holding copies of an old permit must not both lift the same load. BCM job/node/bottleneck diagnosis, BCM control-plane diagnosis and UFM health belong in the same incident narrative but remain separate product observations. The final recovery must establish physical capacity, reachable isolated paths, single-writer authority and a correct workload result.''',
 ['Capture a baseline dependency graph linking controller state, device identity, fabric policy, storage checkpoint and workload admission. Identify which state is external to each backup.',
 'Diagnose a compound BCM job/node/bottleneck and UFM health incident using independent observations from prior courses. Select the smallest failure domain that explains the evidence.',
 'Fence stale owners, restore dependencies and reconcile current physical/network identity before resuming work. Reject an old epoch and a correct-looking checkpoint from the wrong dataset.',
 'Run the trainer holdout with reordered failures and delayed observations. Demonstrate that removing any one of the operations, networking or infrastructure repairs prevents acceptance; restore each and collect recovery evidence.'],
 'Build a production-oriented recovery orchestrator for a multi-tenant AI factory. Integrate all prior projects into a versioned plan/evidence/rollback system. Its release artifact includes the complete objective ledger, all holdouts, explicit blocked capabilities and an operator handover.',
 ['A control-plane recovery restores old jobs onto replacement hardware while the fabric manager is degraded. Recover a single consistent factory state without split brain.',
  'A delayed success report arrives during a second failure and would incorrectly complete maintenance. Preserve ordered ownership, restore the actual workload and reject stale evidence.'],
 ('Salvage ledger: recovery DAG','Lifting permit: current epoch','Recovered crate: validated workload state'),
 '''from ncp_lab.macros import macros, require
order = ["fence","identity","fabric","storage","admission","workload"]
edges = [("fence","identity"),("identity","fabric"),("fabric","admission"),
         ("storage","admission"),("admission","workload")]
position = {name:i for i,name in enumerate(order)}
require[all(position[a] < position[b] for a,b in edges)]
current_epoch, report_epoch = 31, 30
require[current_epoch != report_epoch]
''')
