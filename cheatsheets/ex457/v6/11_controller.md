# 11 — Controller objects and execution

## Create and inspect

Create Machine, Source Control and Container Registry credentials, a Git Project, inventory groups, a published EE and four Job Templates: Backup, Change, Verify, Diagnose. The complete CaC exercise in chapter 12 creates the same study objects for repeatable practice.

Use AAP's Automation Execution views to inspect each object. Sync the Project and verify its selected revision and discovered playbooks. Assign the EE and Machine plus Trust/Backup credentials to every template. Match inventory groups to the local graph. Launch a version/verification job only after the execution node can route to `172.30.0.0/24` and pull the EE.

The trust credential renders a file of verified server public keys into the job and injects its path as `EX457_KNOWN_HOSTS`. Machine credentials supply client identity. They serve different purposes. Regenerate the public trust input when you intentionally rotate host keys. Red Hat documents that machine credentials are used by `network_cli`, that Source Control credentials are used by projects, and that custom credential types can inject file-based content. The exact Trust/Backup credential schema in this repository remains a pack-specific implementation that must be accepted on the target Controller.

[AAP 2.6 credentials](https://docs.redhat.com/en/documentation/red_hat_ansible_automation_platform/2.6/secure-assembly_controller_credentials). [AAP 2.6 source-control credentials](https://docs.redhat.com/en/documentation/red_hat_ansible_automation_platform/2.6/secure-ref_controller_credential_source_control). [AAP 2.6 projects](https://docs.redhat.com/en/documentation/red_hat_ansible_automation_platform/2.6/develop-proc_controller_adding_a_project). [AAP 2.6 custom credentials](https://docs.redhat.com/en/documentation/red_hat_ansible_automation_platform/2.6/secure-assembly_controller_custom_credentials).

## Workflow gate

```mermaid
flowchart TD
  B[Backup] -->|success| C[Change]
  C -->|success| V[Verify]
  B -->|failure| D[Diagnose]
  C -->|failure| D
  V -->|failure| D
```

Red Hat documents workflow nodes with success, failure and always relationships. The graph above is this pack's exam-practice policy, not a vendor-prescribed topology. Record every node's job ID/status. A handled failure can leave the workflow successful, so `controller/launch.yml` also asserts successful Backup, Change and Verify node jobs. A missing node or incomplete paginated response fails. This workflow gates control-plane practice; the lab-host namespace/persistence gates remain explicit chapter-20 requirements.

[AAP 2.6 workflows](https://docs.redhat.com/en/documentation/red_hat_ansible_automation_platform/2.6/develop-assembly_ug_controller_workflows). [AAP 2.6 job templates](https://docs.redhat.com/en/documentation/red_hat_ansible_automation_platform/2.6/develop-proc_controller_create_job_template).
