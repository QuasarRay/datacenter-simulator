# Datacenter simulator

A Containerlab spine/leaf routing model driven by Ansible.

The [EX457 cheatsheet v6](cheatsheets/ex457/v6/README.md) contains the current study pack, runnable automation, a mapping to all 23 public exam objectives, official documentation references and dispositions for all 103 available historical audit findings.

Start in `cheatsheets/ex457/v6` and follow its setup instructions. The v6 lab uses its own topology name and a public-key-only FRR adapter. Its subnet must be free before deployment.

Review [tests and CI](cheatsheets/ex457/v6/21_v6_tests_ci.md) for the distinction between local regressions, required live FRR gates and target-specific Controller, EE, CachyOS and RHEL acceptance. Each Actions run records its actual results; a test mapping alone is not evidence that its runtime gate passed.

## EX457 v6

v6 adds audit-linked Hypothesis and pyATS tests, strict backup/transport/route contracts, and a Dagger pipeline in `ci/`. See [historical audit mappings](cheatsheets/ex457/v6/evidence/audit-history.md). The workflow requires both the original simulator and the v6 lab checks to pass. The [v5 pack](cheatsheets/ex457/v5/README.md) remains as historical input for regression comparisons.
