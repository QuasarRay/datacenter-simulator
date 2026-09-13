# Datacenter simulator

A Containerlab spine/leaf routing model driven by Ansible.

The [EX457 cheatsheet v5](cheatsheets/ex457/v5/README.md) adds a complete study pack, runnable automation, a mapping to all 23 public exam objectives, official documentation references, v4 audit dispositions and reproducible validation evidence.

Start in `cheatsheets/ex457/v5` and follow its setup instructions. The v5 lab uses its own topology name and an explicit public-key-only FRR adapter. Its subnet must be free before deployment.

The evidence distinguishes successful local tests from live FRR, Controller, EE and RHEL checks that still require an operator's lab. Review [the validation scope](cheatsheets/ex457/v5/evidence/README.md) before treating the pack as runtime-accepted.

## EX457 v6

The [v6 cheatsheet](cheatsheets/ex457/v6/README.md) adds audit-linked Hypothesis and pyATS tests, strict backup/transport/route contracts, and a Dagger pipeline in `ci/`. See [test and CI instructions](cheatsheets/ex457/v6/21_v6_tests_ci.md) and [historical audit mappings](cheatsheets/ex457/v6/evidence/audit-history.md). The workflow requires both the original simulator and the v6 lab checks to pass.
