> Implementation authorized by the project owner on 2026-09-24. The former implementation prohibition is withdrawn. See integrations/incus for implemented behavior and validation boundaries.

# Baseline file coverage ledger

This ledger accounts for every **349 tracked file/gitlink paths** in baseline `c6aedddfbb9ac569b5e0eb68acc616452bf66136`. It is an inventory of migration disposition, not a claim that every historical log or every line of upstream vendor code has been audited. Submodule gitlinks represent pinned upstream trees; their consumed boundaries are described in the linked chapters. Generated/untracked runtime artifacts are covered separately below.

No path in this ledger is changed by this documentation proposal. Every action is for a future separately requested implementation. Re-run the inventory against the eventual implementation base and review added/deleted/renamed paths.

## Dispositions

| Code | Required later treatment |
|---|---|
| CONTRACT | Preserve core behavior; inspect callers, feature gates and execution boundaries per the module map. Change only where the new runtime requires it. |
| PORT | Translate OS/runtime/transport assumptions and qualify the actual replacement; retain semantics. |
| REBIND | Preserve the test/example/tool behavior and bind it to Incus observations or the common lifecycle; do not delete the assertion. |
| RETIRE | Remove the active old-runtime artifact after its replacement passes; preserve history where needed. |
| CI | Replace orchestration/dependencies and retain strict evidence/verdict/cleanup contracts. |
| PRODUCT | Preserve and qualify actual IDE/Controller/EE semantics; label CachyOS adaptation or unavailable product requirements. |
| HIST | Keep inert historical regression input; eliminate it as a supported deployable backend. |
| EVID | Preserve historical provenance, then regenerate current-runtime claims; never relabel old results as Incus passes. |
| PIN | Review/update lock, feature or gitlink only with matched source and qualification evidence. |
| DOC | Update supported commands, fidelity boundaries and cross-links at cutover; keep historical reports labeled. |
| META | Review ignore/output paths and packaging consequences; retain useful exclusions. |
| LEGAL | Preserve license/notice attribution and applicable source provenance. |

## Inventory totals

| Disposition | Paths |
|---|---:|
| CI | 13 |
| CONTRACT | 21 |
| DOC | 9 |
| EVID | 41 |
| HIST | 107 |
| LEGAL | 3 |
| META | 2 |
| PIN | 11 |
| PORT | 81 |
| PRODUCT | 8 |
| REBIND | 40 |
| RETIRE | 13 |

## All tracked baseline paths

| Path | Disposition | Detailed instructions |
|---|---|---|
| `.github/workflows/ci.yml` | CI | [16](16-ci-and-acceptance.md) |
| `.github/workflows/deepops-gpu.yml` | CI | [16](16-ci-and-acceptance.md) |
| `.github/workflows/deepops.yml` | CI | [16](16-ci-and-acceptance.md) |
| `.github/workflows/hardware-evidence.yml` | CI | [16](16-ci-and-acceptance.md) |
| `.github/workflows/intended-fabric.yml` | CI | [16](16-ci-and-acceptance.md) |
| `.github/workflows/nccl-runtime.yml` | CI | [16](16-ci-and-acceptance.md) |
| `.github/workflows/rdma-runtime.yml` | CI | [16](16-ci-and-acceptance.md) |
| `.github/workflows/rust-simulator.yml` | CI | [16](16-ci-and-acceptance.md) |
| `.gitignore` | META | [17](17-cutover-and-removal.md) |
| `.gitmodules` | PIN | [18](18-sources-and-version-policy.md) |
| `README.md` | DOC | [17](17-cutover-and-removal.md) |
| `cheatsheets/ex457/v5/.gitignore` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/00_blueprint_map.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/01_lab_baseline.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/02_inventory_navigator.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/03_git_playbooks.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/04_variables_facts_filters.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/05_task_control.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/06_roles_collections_ee.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/07_network_automation.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/08_verification.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/09_drift_and_removal.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/10_backups_persistence.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/11_controller.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/12_controller_iac.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/13_multivendor_transfer.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/14_mock_exam_240min.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/15_targeted_drills.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/16_speed_troubleshooting.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/17_version_compatibility.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/18_repo_migration_checklist.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/19_cachyos_rhel_networking.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/20_runtime_release_gate.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/LICENSE.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/README.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/ansible-navigator.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/ansible.cfg` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/component-lock.json` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/controller/configure.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/controller/launch.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/controller/site.example.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/ee/execution-environment.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/ee/requirements.txt` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/ee_smoke.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/README.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/audit-closure.json` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/audit-closure.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/blueprint.json` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/claims.json` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/content-to-sources.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/input-provenance.json` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/logs/actual_module_arguments.log` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/logs/ansible_syntax-backup_fabric.log` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/logs/ansible_syntax-configure.log` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/logs/ansible_syntax-configure_fabric.log` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/logs/ansible_syntax-diagnose_fabric.log` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/logs/ansible_syntax-ee_smoke.log` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/logs/ansible_syntax-facts_filters.log` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/logs/ansible_syntax-launch.log` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/logs/ansible_syntax-persist_fabric.log` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/logs/ansible_syntax-render.log` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/logs/ansible_syntax-restore_fabric.log` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/logs/ansible_syntax-show_version.log` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/logs/ansible_syntax-task_control.log` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/logs/ansible_syntax-verify_fabric.log` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/logs/ansible_synthetic_dataflow.log` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/logs/builder_definition.log` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/logs/dataflow-extra_idle_peer.log` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/logs/dataflow-good.log` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/logs/dataflow-undefined_remote_nodes.log` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/logs/dataflow-uninstalled_route.log` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/logs/dataflow-wrong_interface.log` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/logs/navigator_host_inventory.log` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/logs/navigator_settings.log` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/logs/predicate_unit_tests.log` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/manifest.sha256` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/preparation-smoke.json` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/source-review.md` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/sources.json` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/evidence/validation.json` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/filter_plugins/fabric.py` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/inventory/group_vars/fabric.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/inventory/localhost.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/inventory/network.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/lab/frr/Dockerfile` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/lab/frr/ex457-load` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/lab/frr/ex457-start` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/lab/frr/sshd_config` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/model-upstream.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/playbooks/backup_fabric.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/playbooks/configure_fabric.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/playbooks/diagnose_fabric.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/playbooks/facts_filters.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/playbooks/filter_plugins/fabric.py` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/playbooks/persist_fabric.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/playbooks/render.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/playbooks/restore_fabric.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/playbooks/show_version.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/playbooks/task_control.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/playbooks/tasks/cleanup_trust.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/playbooks/tasks/model.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/playbooks/tasks/trust.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/playbooks/tasks/verify.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/playbooks/tasks/verify_remote.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/playbooks/verify_fabric.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/requirements.txt` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/requirements.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/roles/fabric/defaults/main.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/roles/fabric/tasks/main.yml` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/templates/datacenter.clab.yml.j2` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/templates/fabric.conf.j2` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/tests/fixture_device.py` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/tests/test_fabric.py` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/tools/check_dataflow.py` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/tools/check_module_args.py` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/tools/labctl.py` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/tools/render_evidence.py` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v5/tools/validate.py` | HIST | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/.gitignore` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/00_blueprint_map.md` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/01_lab_baseline.md` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/02_inventory_navigator.md` | PRODUCT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/03_git_playbooks.md` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/04_variables_facts_filters.md` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/05_task_control.md` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/06_roles_collections_ee.md` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/07_network_automation.md` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/08_verification.md` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/09_drift_and_removal.md` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/10_backups_persistence.md` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/11_controller.md` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/12_controller_iac.md` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/13_multivendor_transfer.md` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/14_mock_exam_240min.md` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/15_targeted_drills.md` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/16_speed_troubleshooting.md` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/17_version_compatibility.md` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/18_repo_migration_checklist.md` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/19_cachyos_rhel_networking.md` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/20_runtime_release_gate.md` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/21_v6_tests_ci.md` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/LICENSE.md` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/README.md` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/ansible-navigator.yml` | PRODUCT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/ansible.cfg` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/component-lock.json` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/controller/configure.yml` | PRODUCT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/controller/filter_plugins/controller_graph.py` | PRODUCT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/controller/launch.yml` | PRODUCT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/controller/site.example.yml` | PRODUCT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/ee/execution-environment.yml` | PRODUCT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/ee/requirements.txt` | PRODUCT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/ee_smoke.yml` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/evidence/README.md` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/audit-closure.json` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/audit-closure.md` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/audit-history.json` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/audit-history.md` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/blueprint.json` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/ci-runs.json` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/claims.json` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/component-versions.json` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/content-to-sources.md` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/image-publication.json` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/input-provenance.json` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/local-validation.json` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/logs/actual_module_arguments.log` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/logs/ansible_syntax-backup_fabric.log` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/logs/ansible_syntax-configure.log` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/logs/ansible_syntax-configure_fabric.log` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/logs/ansible_syntax-diagnose_fabric.log` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/logs/ansible_syntax-ee_smoke.log` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/logs/ansible_syntax-facts_filters.log` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/logs/ansible_syntax-launch.log` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/logs/ansible_syntax-persist_fabric.log` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/logs/ansible_syntax-render.log` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/logs/ansible_syntax-restore_fabric.log` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/logs/ansible_syntax-show_version.log` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/logs/ansible_syntax-task_control.log` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/logs/ansible_syntax-verify_fabric.log` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/logs/ansible_synthetic_dataflow.log` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/logs/builder_definition.log` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/logs/navigator_host_inventory.log` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/logs/navigator_settings.log` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/logs/network-cleanup-followup.log` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/logs/predicate_unit_tests.log` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/logs/pytest.log` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/manifest.sha256` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/official-doc-audit.json` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/official-source-quotes.json` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/runtime-doc-contracts.json` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/source-review.md` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/sources.json` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/evidence/v6-changes.md` | EVID | [16](16-ci-and-acceptance.md) |
| `cheatsheets/ex457/v6/filter_plugins/controller_graph.py` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/filter_plugins/fabric.py` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/inventory/group_vars/fabric.yml` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/inventory/localhost.yml` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/inventory/network.yml` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/lab/frr/Dockerfile` | RETIRE | [17](17-cutover-and-removal.md) |
| `cheatsheets/ex457/v6/lab/frr/ex457-load` | RETIRE | [17](17-cutover-and-removal.md) |
| `cheatsheets/ex457/v6/lab/frr/ex457-start` | RETIRE | [17](17-cutover-and-removal.md) |
| `cheatsheets/ex457/v6/lab/frr/sshd_config` | RETIRE | [17](17-cutover-and-removal.md) |
| `cheatsheets/ex457/v6/model-upstream.yml` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/playbooks/backup_fabric.yml` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/playbooks/configure_fabric.yml` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/playbooks/diagnose_fabric.yml` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/playbooks/facts_filters.yml` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/playbooks/filter_plugins/fabric.py` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/playbooks/persist_fabric.yml` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/playbooks/render.yml` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/playbooks/restore_fabric.yml` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/playbooks/show_version.yml` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/playbooks/task_control.yml` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/playbooks/tasks/cleanup_trust.yml` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/playbooks/tasks/model.yml` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/playbooks/tasks/trust.yml` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/playbooks/tasks/verify.yml` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/playbooks/tasks/verify_remote.yml` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/playbooks/verify_fabric.yml` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/pytest.ini` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/requirements.txt` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/requirements.yml` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/roles/fabric/defaults/main.yml` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/roles/fabric/tasks/main.yml` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/templates/datacenter.clab.yml.j2` | RETIRE | [17](17-cutover-and-removal.md) |
| `cheatsheets/ex457/v6/templates/fabric.conf.j2` | PORT | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/tests/conftest.py` | REBIND | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/tests/fixture_device.py` | REBIND | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/tests/pyats_live.py` | REBIND | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/tests/requirements.txt` | REBIND | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/tests/test_audit6311.py` | REBIND | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/tests/test_automation.py` | REBIND | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/tests/test_ci_gate.py` | REBIND | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/tests/test_documentation_contract.py` | REBIND | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/tests/test_evidence.py` | REBIND | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/tests/test_fabric.py` | REBIND | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/tests/test_https.py` | REBIND | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/tests/test_properties.py` | REBIND | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/tests/test_state.py` | REBIND | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/tools/backup_store.py` | REBIND | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/tools/check_dataflow.py` | REBIND | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/tools/check_module_args.py` | REBIND | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/tools/check_previous_release.py` | REBIND | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/tools/contracts.py` | REBIND | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/tools/documentation_contract.py` | REBIND | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/tools/evidence_contract.py` | REBIND | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/tools/labctl.py` | REBIND | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/tools/package.py` | REBIND | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/tools/patch_libssh.py` | REBIND | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/tools/render_evidence.py` | REBIND | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/tools/state.py` | REBIND | [13](13-ex457-network-automation.md) |
| `cheatsheets/ex457/v6/tools/validate.py` | REBIND | [13](13-ex457-network-automation.md) |
| `ci/check_results.py` | CI | [16](16-ci-and-acceptance.md) |
| `ci/cleanup_network.py` | CI | [16](16-ci-and-acceptance.md) |
| `ci/dagger.json` | RETIRE | [16](16-ci-and-acceptance.md) |
| `ci/prepare_dind.py` | RETIRE | [16](16-ci-and-acceptance.md) |
| `ci/pyproject.toml` | CI | [16](16-ci-and-acceptance.md) |
| `ci/requirements.txt` | CI | [16](16-ci-and-acceptance.md) |
| `ci/run_ci.py` | CI | [16](16-ci-and-acceptance.md) |
| `ci/src/datacenter_ci/__init__.py` | RETIRE | [16](16-ci-and-acceptance.md) |
| `ci/src/datacenter_ci/main.py` | RETIRE | [16](16-ci-and-acceptance.md) |
| `docs/GETTING_STARTED.md` | DOC | [17](17-cutover-and-removal.md) |
| `docs/audit-6311-remediation.md` | DOC | [17](17-cutover-and-removal.md) |
| `integrations/deepops/.gitignore` | PORT | [09](09-deepops-kubernetes-mokka.md) |
| `integrations/deepops/README.md` | PORT | [09](09-deepops-kubernetes-mokka.md) |
| `integrations/deepops/config.example.json` | PORT | [09](09-deepops-kubernetes-mokka.md) |
| `integrations/deepops/files/nccl-rank.sh` | PORT | [09](09-deepops-kubernetes-mokka.md) |
| `integrations/deepops/galaxy.lock.json` | PORT | [09](09-deepops-kubernetes-mokka.md) |
| `integrations/deepops/install-galaxy-locked.sh` | PORT | [09](09-deepops-kubernetes-mokka.md) |
| `integrations/deepops/molecule-requirements.lock` | PORT | [09](09-deepops-kubernetes-mokka.md) |
| `integrations/deepops/prepare-nccl.yml` | PORT | [09](09-deepops-kubernetes-mokka.md) |
| `integrations/deepops/setup.sh` | PORT | [09](09-deepops-kubernetes-mokka.md) |
| `integrations/deepops/topology.json` | PORT | [09](09-deepops-kubernetes-mokka.md) |
| `integrations/deepops/upstream` | PORT | [09](09-deepops-kubernetes-mokka.md) |
| `integrations/k8s-test-infra/config.example.json` | PORT | [09](09-deepops-kubernetes-mokka.md) |
| `integrations/k8s-test-infra/kind.ci.yaml` | RETIRE | [09](09-deepops-kubernetes-mokka.md) |
| `integrations/k8s-test-infra/upstream` | PORT | [09](09-deepops-kubernetes-mokka.md) |
| `integrations/netbox/compose.ci.yml` | RETIRE | [07](07-netbox-and-services.md) |
| `integrations/netbox/config.example.json` | PORT | [07](07-netbox-and-services.md) |
| `integrations/netbox/manifest.example.json` | PORT | [07](07-netbox-and-services.md) |
| `integrations/netbox/seed-ci.sh` | PORT | [07](07-netbox-and-services.md) |
| `integrations/netbox/snapshot.example.json` | PORT | [07](07-netbox-and-services.md) |
| `inventory/localhost.ini` | PORT | [06](06-ansible-orchestration.md) |
| `license.md` | LEGAL | [01](01-repository-architecture.md) |
| `playbooks/deploy.yml` | PORT | [06](06-ansible-orchestration.md) |
| `playbooks/destroy.yml` | PORT | [06](06-ansible-orchestration.md) |
| `rust-toolchain.toml` | PIN | [18](18-sources-and-version-policy.md) |
| `simulator/.gitignore` | META | [17](17-cutover-and-removal.md) |
| `simulator/Cargo.lock` | PIN | [18](18-sources-and-version-policy.md) |
| `simulator/Cargo.toml` | PIN | [18](18-sources-and-version-policy.md) |
| `simulator/NOTICE.md` | LEGAL | [01](01-repository-architecture.md) |
| `simulator/README.md` | DOC | [17](17-cutover-and-removal.md) |
| `simulator/docs/api-coverage.md` | DOC | [17](17-cutover-and-removal.md) |
| `simulator/docs/audit-4121cb83-response.md` | DOC | [17](17-cutover-and-removal.md) |
| `simulator/docs/audit-675d35d-response.md` | DOC | [17](17-cutover-and-removal.md) |
| `simulator/docs/design.md` | DOC | [17](17-cutover-and-removal.md) |
| `simulator/docs/intended-state.md` | DOC | [17](17-cutover-and-removal.md) |
| `simulator/examples/gpu-spine-leaf.json` | REBIND | [16](16-ci-and-acceptance.md) |
| `simulator/examples/ibsim_fabric.rs` | REBIND | [16](16-ci-and-acceptance.md) |
| `simulator/examples/linux_fabric.rs` | REBIND | [16](16-ci-and-acceptance.md) |
| `simulator/examples/model_scale.rs` | REBIND | [16](16-ci-and-acceptance.md) |
| `simulator/examples/nccl_runtime.rs` | REBIND | [16](16-ci-and-acceptance.md) |
| `simulator/examples/rdma_loopback.rs` | REBIND | [16](16-ci-and-acceptance.md) |
| `simulator/licenses/nv-air-sdk-MIT.txt` | LEGAL | [01](01-repository-architecture.md) |
| `simulator/rust-toolchain.toml` | PIN | [18](18-sources-and-version-policy.md) |
| `simulator/src/api.rs` | CONTRACT | [01](01-repository-architecture.md) |
| `simulator/src/bounded_log.rs` | CONTRACT | [01](01-repository-architecture.md) |
| `simulator/src/collective.rs` | CONTRACT | [01](01-repository-architecture.md) |
| `simulator/src/command.rs` | CONTRACT | [01](01-repository-architecture.md) |
| `simulator/src/cuda.rs` | CONTRACT | [01](01-repository-architecture.md) |
| `simulator/src/deepops.rs` | PORT | [09](09-deepops-kubernetes-mokka.md) |
| `simulator/src/deepops_runtime.rs` | PORT | [09](09-deepops-kubernetes-mokka.md) |
| `simulator/src/doctor.rs` | CONTRACT | [01](01-repository-architecture.md) |
| `simulator/src/evidence.rs` | CONTRACT | [01](01-repository-architecture.md) |
| `simulator/src/fabric.rs` | CONTRACT | [01](01-repository-architecture.md) |
| `simulator/src/infiniband.rs` | CONTRACT | [01](01-repository-architecture.md) |
| `simulator/src/input.rs` | CONTRACT | [01](01-repository-architecture.md) |
| `simulator/src/lib.rs` | CONTRACT | [01](01-repository-architecture.md) |
| `simulator/src/limits.rs` | CONTRACT | [01](01-repository-architecture.md) |
| `simulator/src/linux.rs` | PORT | [05](05-patchbay-petgraph-networking.md) |
| `simulator/src/main.rs` | CONTRACT | [01](01-repository-architecture.md) |
| `simulator/src/manifest.rs` | CONTRACT | [01](01-repository-architecture.md) |
| `simulator/src/model.rs` | CONTRACT | [01](01-repository-architecture.md) |
| `simulator/src/mokka.rs` | PORT | [09](09-deepops-kubernetes-mokka.md) |
| `simulator/src/nccl_backend.rs` | CONTRACT | [01](01-repository-architecture.md) |
| `simulator/src/nccl_worker.rs` | CONTRACT | [01](01-repository-architecture.md) |
| `simulator/src/netbox.rs` | CONTRACT | [01](01-repository-architecture.md) |
| `simulator/src/process_group.rs` | CONTRACT | [01](01-repository-architecture.md) |
| `simulator/src/provenance.rs` | CONTRACT | [01](01-repository-architecture.md) |
| `simulator/src/rdma.rs` | CONTRACT | [01](01-repository-architecture.md) |
| `simulator/src/topology.rs` | PORT | [05](05-patchbay-petgraph-networking.md) |
| `simulator/src/vm.rs` | RETIRE | [09](09-deepops-kubernetes-mokka.md) |
| `simulator/tests/audit_4121.rs` | REBIND | [16](16-ci-and-acceptance.md) |
| `simulator/tests/audit_6311.rs` | REBIND | [16](16-ci-and-acceptance.md) |
| `simulator/tests/deepops.rs` | REBIND | [16](16-ci-and-acceptance.md) |
| `simulator/tests/intended_state.rs` | REBIND | [16](16-ci-and-acceptance.md) |
| `simulator/tests/mokka_cli.py` | REBIND | [16](16-ci-and-acceptance.md) |
| `simulator/tests/simulator.rs` | REBIND | [16](16-ci-and-acceptance.md) |
| `simulator/upstreams.json` | PIN | [18](18-sources-and-version-policy.md) |
| `simulator/vendor/nccl` | PIN | [01](01-repository-architecture.md) |
| `simulator/vendor/nccl-tests` | PIN | [01](01-repository-architecture.md) |
| `simulator/vendor/patchbay` | PIN | [01](01-repository-architecture.md) |
| `simulator/vendor/petgraph` | PIN | [01](01-repository-architecture.md) |
| `simulator/vendor/rust-ibverbs` | PIN | [01](01-repository-architecture.md) |
| `templates/datacenter.clab.yml.j2` | RETIRE | [17](17-cutover-and-removal.md) |
| `tests/datacenter_state.py` | REBIND | [16](16-ci-and-acceptance.md) |
| `tests/deploy_refusal.py` | REBIND | [16](16-ci-and-acceptance.md) |
| `vars/datacenter.yml` | PORT | [06](06-ansible-orchestration.md) |

## Generated and external state

Also inventory `build/*.clab.yml`, active `.state` directories, client/host SSH keys, known_hosts, Docker images/volumes/networks/labels, Docker/Kind registries, Dagger caches, per-run VM disks/cloud-init seeds/TAP devices, Incus profiles/projects/pools/volumes, kernel namespaces/veths/bridges/qdiscs/routes, NetBox databases/media/secrets, Kubernetes contexts/certificates/CRI images, Slurm state/munge credentials, native GPU/RDMA processes, Python virtual environments/collections, Cargo build artifacts and exported CI archives.

These are not all present in a fresh checkout and must be discovered from the actual host plus ownership journals, never fabricated from this list. Back up retained state, remove only positively owned obsolete runtime resources, and keep secrets out of the repository and CI artifacts. A missing generated topology file must not conceal orphaned resources.
