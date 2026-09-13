# EX457 Containerlab cheatsheet v6

## Start here

A complete practice pack for the public AAP 2.6 EX457 objectives, with a four-router spine/leaf datacenter fabric. Read [the coverage matrix](00_blueprint_map.md), then follow chapters 01–12. Chapters 13–20 cover transfer, timed practice, host networking and release gates.

Run all commands from this directory. In the repository this is `cheatsheets/ex457/v6`; in the ZIP it is `ex457-v6-project/cheatsheets/ex457/v6`. Use Bash for shell examples.

The public blueprint has 23 leaf objectives in eight families. All 23 have mapped instruction, an exercise and acceptance evidence. Your assigned LMS version remains authoritative; this pack cannot prove coverage of unseen private exam tasks.

[EX457 public objectives](https://www.redhat.com/en/services/training/ex457-red-hat-certified-specialist-in-ansible-network-automation-exam).

## What v6 proves

v6 adds Hypothesis properties and stateful regressions, real pyATS lab acceptance, and Dagger/GitHub Actions gates. It addresses all 103 available historical findings with explicit test mappings and remaining product-specific acceptance. No finite suite guarantees all project behavior or an exam result.

Read [tests and CI](21_v6_tests_ci.md), [the change record](evidence/v6-changes.md) and [the historical matrix](evidence/audit-history.md). The GitHub Actions artifact records the actual result for each commit. Controller, entitled EE, CachyOS and RHEL reboot checks remain separate target-specific acceptance.

## Study sequence

1. Read [host preparation](19_cachyos_rhel_networking.md); install tools in a virtual environment.
2. Build the adapter, generate keys, preflight and deploy: [01](01_lab_baseline.md).
3. Inspect inventory and run Navigator in host mode: [02](02_inventory_navigator.md).
4. Practice Git, variables, control flow, roles and an EE: [03](03_git_playbooks.md)–[06](06_roles_collections_ee.md).
5. Configure, verify, inject drift, back up and recover: [07](07_network_automation.md)–[10](10_backups_persistence.md).
6. Publish the EE and configure Controller: [11](11_controller.md)–[12](12_controller_iac.md).
7. Complete the [240-minute mock](14_mock_exam_240min.md) and [runtime gates](20_runtime_release_gate.md).



