# Datacenter simulator

A Rust datacenter simulator with a Containerlab/Ansible network automation lab.

The [Rust simulator](simulator/README.md) implements local NVIDIA Air-inspired lifecycle and topology APIs, actual CUDA/NCCL collectives over Linux namespace networks, a checked software NIC model, and native RDMA execution. See its [API coverage and execution boundaries](simulator/docs/api-coverage.md) and build instructions.

The [EX457 cheatsheet v6](cheatsheets/ex457/v6/README.md) contains the current study pack, runnable automation, a mapping to all 23 public exam objectives, official documentation references and dispositions for all 103 available historical audit findings.

Start in `cheatsheets/ex457/v6` and follow its setup instructions. The v6 lab uses its own topology name and a public-key-only FRR adapter. Its subnet must be free before deployment.

Review [tests and CI](cheatsheets/ex457/v6/21_v6_tests_ci.md) and the [official-document audit](cheatsheets/ex457/v6/evidence/official-doc-audit.json) before treating the pack as exam-ready. The source-grounding layer keeps direct Red Hat/Ansible quotations separate from repository policy and live compatibility evidence; a citation alone is not a pass.

## EX457 v6

v6 adds audit-linked Hypothesis and pyATS tests, strict backup/transport/route contracts, and a Dagger pipeline in `ci/`. See [historical audit mappings](cheatsheets/ex457/v6/evidence/audit-history.md). The workflow requires both the original simulator and the v6 lab checks to pass.

The [v5 pack](cheatsheets/ex457/v5/README.md) is **historical regression input, not current exam guidance**. In particular, do not use its older Controller/CaC wording to choose an AAP 2.6 configuration-as-code collection; the current product correction is documented in [v6 chapter 12](cheatsheets/ex457/v6/12_controller_iac.md).

The root Containerlab/Ansible example is an IP connectivity lab: ASN values are
inventory labels, not configured BGP sessions. Its tests check exact owned
resources, direct data-link traffic, partition and recovery. Routed BGP exercises
live under `cheatsheets/ex457`; the Rust simulator uses explicit static routes.
Redeployment refuses an existing lab and leaves it untouched; destroy it explicitly
before applying changed intent. A failed new deployment rolls back its resources.
