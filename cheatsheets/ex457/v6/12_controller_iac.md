# 12 — Controller configuration as code

## Prepare site inputs

For **AAP 2.6 product guidance**, use Red Hat's supported `infra.aap_configuration` collection from Automation Hub. Red Hat's current AAP 2.6 documentation states that `infra.aap_configuration` incorporates the older controller/hub/EDA configuration collections and **should be used in their place with AAP 2.6**. Do not infer the supported product CaC path from this repository's pinned AWX collection.

The shipped `controller/configure.yml` intentionally remains a **lab compatibility harness** built on pinned `awx.awx 24.6.1` modules. It is useful for exercising object schemas and the simulator's graph, but it is not evidence that `awx.awx` is Red Hat's recommended AAP 2.6 configuration-as-code interface. [check_module_args.py](tools/check_module_args.py) captures those pinned AWX module `argument_spec` values, validates rendered inputs and proves an unknown-argument mutation fails. That proves only the pinned client-side module contract; it does not emulate AAP server schemas, RBAC or supported-collection policy.

```bash
cp controller/site.example.yml controller/site.yml
ansible-vault create controller/secrets.yml
# Fill the secret names listed in site.example.yml; paste only public host keys
# into known_hosts_text. Keep private client keys in the encrypted file.
export CONTROLLER_HOST=https://aap.example.net
export CONTROLLER_OPTIONAL_API_URLPATTERN_PREFIX=/api/controller/
read -r -s -p 'Controller OAuth token: ' CONTROLLER_OAUTH_TOKEN
export CONTROLLER_OAUTH_TOKEN
ansible-playbook -i inventory/localhost.yml controller/configure.yml --ask-vault-pass
ansible-playbook -i inventory/localhost.yml controller/configure.yml --ask-vault-pass
ansible-playbook -i inventory/localhost.yml controller/launch.yml
unset CONTROLLER_OAUTH_TOKEN
```

The commands above run the **AWX compatibility harness**, not the recommended AAP 2.6 CaC collection. For exam/product practice, translate the same desired objects into the `infra.aap_configuration` roles available in the assigned AAP 2.6 environment and validate their documented role variables there. Do not invent role arguments from this AWX playbook. Replace example hosts, registry digest and storage endpoint with your lab values. Pin an accepted Git commit for repeatable study. Leave `project_prefix: cheatsheets/ex457/v6` for this repository.

[Red Hat AAP 2.6 configuration-as-code guidance](https://docs.redhat.com/en/documentation/red_hat_ansible_automation_platform/2.6/secure-ref_initial_configuration). [Pinned AWX 24.6.1 module implementations used only by this lab harness](https://github.com/ansible/awx/tree/24.6.1/awx_collection/plugins/modules).

## Acceptance

For the shipped harness, both passes must complete, with the second run converged except any documented server-side secret update behavior. Inspect memberships, playbook choices, EE digest, credential associations and workflow edges. Launch the workflow and verify core job statuses, exported backup hashes and actual strict-host-key behavior.

For **AAP 2.6 acceptance**, repeat the object-creation exercise with the supported collection installed in the assigned environment, using that collection's actual documentation and role argument specifications. A passing `awx.awx` client-side argument test is not a substitute. The target AAP Controller remains an external acceptance gate because local tests cannot prove its server API, RBAC, execution-node routing, credential injection or image-pull behavior.

[AAP 2.6 custom credentials](https://docs.redhat.com/en/documentation/red_hat_ansible_automation_platform/2.6/secure-assembly_controller_custom_credentials). [AAP 2.6 projects](https://docs.redhat.com/en/documentation/red_hat_ansible_automation_platform/2.6/develop-proc_controller_adding_a_project). [AAP 2.6 workflows](https://docs.redhat.com/en/documentation/red_hat_ansible_automation_platform/2.6/develop-assembly_ug_controller_workflows).

## v6 credential and graph contract

Set `credential_filename_namespace: tower` for the pinned AWX 24.6.1 harness, whose implementation is executed by the regression test. Do **not** generalize that namespace to AAP 2.6; confirm the actual supported collection and target API. Supply `machine_key_passphrase` in encrypted `secrets.yml` for an encrypted client key; otherwise omit it. A superuser must create the custom credential type initially, or set `credential_type_preprovisioned: true` after an administrator installs the reviewed type.

The play owns the four-node EX457 workflow, removes extra nodes, replaces all success/failure/always edges including Diagnose, and the launcher verifies the exact graph and all core node outcomes. Re-run configuration after injecting an extra node and a Diagnose outgoing edge, then prove the graph is clean. A structural module-spec test cannot substitute for server-side acceptance.

[AWX 24.6.1 injector source used by the compatibility regression](https://github.com/ansible/awx/blob/24.6.1/awx/main/models/credential/__init__.py).
