# 12 — Controller configuration as code

## Prepare site inputs

`controller/configure.yml` uses the pinned `awx.awx` modules directly. The old `infra.aap_configuration` wrapper and its hand-written pseudo-schema are removed. [check_module_args.py](tools/check_module_args.py) captures the real module `argument_spec`, validates rendered inputs and proves an unknown-argument mutation fails. This does not emulate the Controller server's nested credential schemas or prove API compatibility.

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

Replace example hosts, registry digest and storage endpoint with your lab values. Until the v5 PR is merged, set `scm_branch` to its branch; then pin an accepted commit for repeatable study. Leave `project_prefix: cheatsheets/ex457/v5` for this repository. The API prefix variable is implemented in awx.awx 24.6.1; use `/api/` for AWX/direct Controller only when that matches your deployment.

[Pinned CaC module implementations](https://github.com/ansible/awx/tree/24.6.1/awx_collection/plugins/modules). [AAP 2.6 execution best practices](https://docs.redhat.com/en/documentation/red_hat_ansible_automation_platform/2.6/develop-assembly_controller_best_practices).

## Acceptance

Both CaC passes must complete, with the second run converged except any documented server-side secret update behavior. Inspect memberships, playbook choices, EE digest, credential associations and workflow edges. The collection modules create nodes before links, avoiding unresolved references. Launch the workflow and verify core job statuses, exported backup hashes and actual strict-host-key behavior. Use the assigned exam's installed supported collection if it differs; revalidate names/arguments against its docs.

[AWX workflows](https://docs.ansible.com/projects/awx/en/24.6.1/userguide/workflow_templates.html). [AWX custom credentials](https://docs.ansible.com/projects/awx/en/24.6.1/userguide/credential_types.html). [AWX projects](https://docs.ansible.com/projects/awx/en/24.6.1/userguide/projects.html).

