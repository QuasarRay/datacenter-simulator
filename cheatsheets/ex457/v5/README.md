> Historical course text and regression evidence. Its former runtime has been retired. Use [the Incus backend](../../../integrations/incus/README.md) for new deployments. Historical release validation is not a current acceptance gate.

# EX457 Containerlab cheatsheet v5

## Start here

A complete practice pack for the public AAP 2.6 EX457 objectives, with a four-router spine/leaf datacenter fabric. Read [the coverage matrix](00_blueprint_map.md), then follow chapters 01–12. Chapters 13–20 cover transfer, timed practice, host networking and release gates.

Run all commands from this directory. In the repository this is `cheatsheets/ex457/v5`; in the ZIP it is the top-level `ex457_containerlab_cheatsheet_v5` directory. Use Bash for shell examples.

The public blueprint has 23 leaf objectives in eight families. All 23 have mapped instruction, an exercise and acceptance evidence. Your assigned LMS version remains authoritative; this pack cannot prove coverage of unseen private exam tasks.

[EX457 public objectives](https://www.redhat.com/en/services/training/ex457-red-hat-certified-specialist-in-ansible-network-automation-exam).

## What v5 proves

[Validation results](evidence/validation.json) record actual commands and outcomes. The [48-finding disposition table](evidence/audit-closure.md) distinguishes implemented corrections from remaining runtime gates. The [source matrix](evidence/content-to-sources.md) links specific sections and claims to official documentation.

Unit and fixture tests establish the documented verifier behavior. Syntax and argument-spec checks establish narrower parsing properties. Docker image execution, FRR/libssh interoperability, persistence, EE distribution and Controller jobs require the supplied live gates. No runtime pass or vendor support claim is inferred from citations.

The simulator models IPv4 routing and automation practice. It does not reproduce GPU compute, RDMA/RoCE, PFC/ECN, switch ASIC queues, optics, physical faults or production throughput.

[Pinned simulator model](https://github.com/QuasarRay/datacenter-simulator/blob/2003f8bdd88423c77c528785f7ffa70f398a23e0/vars/datacenter.yml).

## Study sequence

1. Read [host preparation](19_cachyos_rhel_networking.md); install tools in a virtual environment.
2. Build the adapter, generate keys, preflight and deploy: [01](01_lab_baseline.md).
3. Inspect inventory and run Navigator in host mode: [02](02_inventory_navigator.md).
4. Practice Git, variables, control flow, roles and an EE: [03](03_git_playbooks.md)–[06](06_roles_collections_ee.md).
5. Configure, verify, inject drift, back up and recover: [07](07_network_automation.md)–[10](10_backups_persistence.md).
6. Publish the EE and configure Controller: [11](11_controller.md)–[12](12_controller_iac.md).
7. Complete the [240-minute mock](14_mock_exam_240min.md) and [runtime gates](20_runtime_release_gate.md).



