# 17 — Version and compatibility record

## Pins and boundaries

| Component | v5 contract | Evidence |
|---|---|---|
| Public exam | AAP 2.6 | Red Hat page, checked 2026-09-13 |
| Local static/fixture engine | ansible-core 2.19.3 | Actual command logs |
| netcommon / utils | 8.1.0 / 6.0.0 | Installed Galaxy manifests and module specs |
| FRR collection | 2.0.2 | Deprecated; published FRR 6.0 testing |
| Python libssh binding | 1.2.2 | Installed version; transport runtime gate |
| Navigator / Builder | 25.5.0 / 3.1.0 | Settings/definition validation |
| Controller CaC collection | awx.awx 24.6.1 | Actual argument specs; AAP API compatibility gate |
| Router runtime | FRR 10.7 containerlab flavor | Operator-inspected digest required |
| Containerlab | 0.79.0 | Preflight requires this lifecycle version |
| EE base | AAP 2.6 minimal RHEL 9 | Entitled pull; pin digest before acceptance |
| CachyOS | Rolling | Record installed package/kernel versions |
| RHEL network examples | RHEL 9 | Versioned Red Hat network chapters |

AAP 2.6 does not imply the local 2.19.3 test engine is the exam's bundled core. The product documentation currently distinguishes core 2.19 preview content; record the actual assigned EE version. Moving docs are dated and carry short evidence excerpts; a local digest is not fabricated from a tag. See component-lock.json for the exact files used during validation.

[EX457 public objectives](https://www.redhat.com/en/services/training/ex457-red-hat-certified-specialist-in-ansible-network-automation-exam). [AAP 2.6 execution best practices](https://docs.redhat.com/en/documentation/red_hat_ansible_automation_platform/2.6/develop-assembly_controller_best_practices). [FRR facts](https://docs.ansible.com/projects/ansible/9/collections/frr/frr/frr_facts_module.html). [RHEL Ethernet connections](https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/9/html/configuring_and_managing_networking/configuring-an-ethernet-connection_configuring-and-managing-networking).

