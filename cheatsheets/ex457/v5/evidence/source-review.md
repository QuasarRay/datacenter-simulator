# Source review record

Retrieval date: 2026-09-13. Short notes/excerpts below were recorded during source review. They are not full-page snapshots. Source semantics and runtime behavior are different forms of evidence. Product links that redirected to unrelated landing pages were not used as substantive claim support.

## EXAM

[EX457 public objectives](https://www.redhat.com/en/services/training/ex457-red-hat-certified-specialist-in-ansible-network-automation-exam)

Authority: Red Hat exam page. Version/scope: Public AAP 2.6 blueprint.

Locator: Exam Objectives; Performance Rule; Recommended Preparation.

Review note: This exam is based on Red Hat Ansible Automation Platform 2.6.

## INV

[Ansible inventory](https://docs.ansible.com/projects/ansible-core/2.19/inventory_guide/intro_inventory.html)

Authority: Official Ansible documentation. Version/scope: ansible-core 2.19.

Locator: Inventory basics; group variables; behavioral inventory parameters.

Review note: Groups allow you to reference multiple associated hosts to target for your automation.

## PLAY

[Ansible playbooks](https://docs.ansible.com/projects/ansible-core/2.19/playbook_guide/playbooks_intro.html)

Authority: Official Ansible documentation. Version/scope: ansible-core 2.19.

Locator: Playbook syntax; Running playbooks; Verifying playbooks.

Review note: A playbook consists of one or more ‘plays’ in an ordered list.

## VARS

[Ansible variables](https://docs.ansible.com/projects/ansible-core/2.19/playbook_guide/playbooks_variables.html)

Authority: Official Ansible documentation. Version/scope: ansible-core 2.19.

Locator: Registering variables; precedence; dictionary variables.

Review note: You can create a variable from the output of an Ansible task with the task keyword register.

## FILTER

[Ansible filters](https://docs.ansible.com/projects/ansible/latest/playbook_guide/playbooks_filters.html)

Authority: Official Ansible documentation. Version/scope: current, retrieved 2026-09-13.

Locator: Transforming dictionaries; set theory; JSON and YAML.

Review note: filters execute on the control node and transform data locally.

## LOOP

[Ansible loops](https://docs.ansible.com/projects/ansible-core/2.19/playbook_guide/playbooks_loops.html)

Authority: Official Ansible documentation. Version/scope: ansible-core 2.19.

Locator: loop_control; retrying a task until a condition is met.

Review note: Retrying a task until a condition is met

## BLOCK

[Ansible blocks](https://docs.ansible.com/projects/ansible/latest/playbook_guide/playbooks_blocks.html)

Authority: Official Ansible documentation. Version/scope: current, retrieved 2026-09-13.

Locator: Handling errors with blocks.

Review note: Handling errors with blocks

## REUSE

[Reusing Ansible artifacts](https://docs.ansible.com/projects/ansible-core/2.19/playbook_guide/playbooks_reuse.html)

Authority: Official Ansible documentation. Version/scope: ansible-core 2.19.

Locator: Includes versus imports; reusing roles.

Review note: Comparing includes and imports: dynamic and static reuse

## NETCLI

[network_cli connection](https://docs.ansible.com/projects/ansible/latest/collections/ansible/netcommon/network_cli_connection.html)

Authority: Official Ansible collection docs. Version/scope: Current page 8.6.2; execution pinned separately to 8.1.0.

Locator: ssh_type; host_key_checking; host_key_auto_add.

Review note: The python package that will be used by the network_cli connection plugin to create a SSH connection to remote host.

## LIBSSH

[libssh connection](https://docs.ansible.com/projects/ansible/latest/collections/ansible/netcommon/libssh_connection.html)

Authority: Official Ansible collection docs. Version/scope: config_file added in 5.1.0; local args/source checked at 8.1.0.

Locator: config_file; ssh_common_args; host_key_checking.

Review note: Alternate SSH config file location

## LIBSSH-SRC

[libssh pinned implementation](https://github.com/ansible-collections/ansible.netcommon/blob/v8.1.0/plugins/connection/libssh.py)

Authority: Official upstream Ansible source. Version/scope: v8.1.0.

Locator: ssh_connect_kwargs config_file assignment.

Review note: ssh_connect_kwargs["config_file"] = self.get_option("config_file")

## CONFIG

[cli_config module](https://docs.ansible.com/projects/ansible/latest/collections/ansible/netcommon/cli_config_module.html)

Authority: Official Ansible collection docs. Version/scope: Pinned local argument_spec: 8.1.0.

Locator: config; diff_match; platform-dependent capabilities.

Review note: Push text based configuration to network devices over network_cli

## BACKUP

[cli_backup module](https://docs.ansible.com/projects/ansible/latest/collections/ansible/netcommon/cli_backup_module.html)

Authority: Official Ansible collection docs. Version/scope: Pinned local argument_spec: 8.1.0.

Locator: dir_path; filename.

Review note: Back up device configuration from network devices over network_cli

## FACTS

[FRR facts](https://docs.ansible.com/projects/ansible/9/collections/frr/frr/frr_facts_module.html)

Authority: Official Ansible collection docs. Version/scope: frr.frr 2.0.2; deprecated.

Locator: gather_subset; Notes; Return Values.

Review note: Tested against FRR 6.0.

## NAV

[Navigator settings](https://docs.ansible.com/projects/navigator/settings/)

Authority: Official Ansible project docs. Version/scope: Current docs; local Navigator 25.5.0 validation.

Locator: execution-environment; container-options; mode.

Review note: Execution environment settings

## BUILDER

[Builder definition](https://docs.ansible.com/projects/builder/en/stable/definition/)

Authority: Official Ansible project docs. Version/scope: Builder 3 schema; local 3.1.0.

Locator: version 3; images; dependencies.

Review note: Execution environment definition

## AAP

[AAP 2.6 execution best practices](https://docs.redhat.com/en/documentation/red_hat_ansible_automation_platform/2.6/develop-assembly_controller_best_practices)

Authority: Red Hat product documentation. Version/scope: AAP 2.6.

Locator: Use source control; file and directory structure.

Review note: Automation controller does not support interactive features.

## CREDS

[AWX credentials](https://docs.ansible.com/projects/awx/en/24.6.1/userguide/credentials.html)

Authority: Official Ansible AWX community docs. Version/scope: 24.6.1; product compatibility remains gated.

Locator: Machine; Source Control; Container Registry.

Review note: Credentials

## CUSTOM

[AWX custom credentials](https://docs.ansible.com/projects/awx/en/24.6.1/userguide/credential_types.html)

Authority: Official Ansible AWX community docs. Version/scope: 24.6.1.

Locator: Create a New Credential Type; temporary files.

Review note: The absolute file path to the generated file will be stored in an environment variable named MY_CLOUD_INI_FILE.

## PROJECT

[AWX projects](https://docs.ansible.com/projects/awx/en/24.6.1/userguide/projects.html)

Authority: Official Ansible AWX community docs. Version/scope: 24.6.1.

Locator: SCM projects and synchronization.

Review note: Projects

## WORKFLOW

[AWX workflows](https://docs.ansible.com/projects/awx/en/24.6.1/userguide/workflow_templates.html)

Authority: Official Ansible AWX community docs. Version/scope: 24.6.1.

Locator: Workflow nodes; success/failure transitions.

Review note: Workflow Job Templates

## AWX-SRC

[Pinned CaC module implementations](https://github.com/ansible/awx/tree/24.6.1/awx_collection/plugins/modules)

Authority: Official upstream Ansible AWX source. Version/scope: Installed Galaxy awx.awx 24.6.1; SHA-256 records in component-lock.json.

Locator: main() argument_spec; workflow_job_template_node; project; credential_type.

Review note: argument_spec

## FRR-BGP

[FRR BGP](https://docs.frrouting.org/en/stable-10.7/bgp.html)

Authority: Official FRRouting documentation. Version/scope: stable-10.7.

Locator: ASN and Router ID; route selection; show bgp; ebgp-requires-policy.

Review note: Enable a BGP protocol process with the specified ASN.

## FRR-ZEBRA

[FRR Zebra](https://docs.frrouting.org/en/stable-10.7/zebra.html)

Authority: Official FRRouting documentation. Version/scope: stable-10.7.

Locator: Interface commands; show ip route.

Review note: Zebra

## FRR-SAVE

[FRR integrated configuration](https://docs.frrouting.org/en/stable-10.7/vtysh.html)

Authority: Official FRRouting documentation. Version/scope: stable-10.7.

Locator: Integrated configuration file; configuration saving.

Review note: vtysh -b must also be executed after restarting any daemon.

## FRR-IMAGE

[FRR Containerlab image source](https://github.com/FRRouting/frr/blob/1286d6389626a77648ed844b176c2324466c1371/docker/containerlab/Dockerfile)

Authority: Official upstream FRRouting source. Version/scope: 1286d6389626a77648ed844b176c2324466c1371.

Locator: FROM and docker-start COPY.

Review note: This is the released FRR image with an SSH server added.

## CLAB

[Containerlab Linux kind](https://containerlab.dev/manual/kinds/linux/)

Authority: Official Containerlab documentation. Version/scope: Current docs; required local version 0.79.0.

Locator: Interfaces; lifecycle; restart.

Review note: Linux container

## RESTART

[Containerlab restart command](https://containerlab.dev/cmd/restart/)

Authority: Official Containerlab documentation. Version/scope: Current docs; required local version 0.79.0.

Locator: Description: lifecycle stop/start and interface restoration; Limitations.

Review note: Containerlab restores parked data interfaces. The pack wrapper separately reloads integrated FRR configuration.

## SSH

[OpenSSH sshd](https://man.openbsd.org/sshd)

Authority: Official OpenSSH manual. Version/scope: Current manual; Alpine image gate remains required.

Locator: Authentication; account access.

Review note: sshd (OpenSSH Daemon)

## CACHY

[CachyOS post-install](https://wiki.cachyos.org/configuration/post_install_setup/)

Authority: Official CachyOS documentation. Version/scope: Rolling release; reviewed 2026-09-13.

Locator: Configuring Firewall (ufw); Changing the Default Shell.

Review note: UFW is enabled by default after installation.

## CACHY-VM

[CachyOS QEMU/VMM](https://wiki.cachyos.org/virtualization/qemu_and_vmm_setup/)

Authority: Official CachyOS documentation. Version/scope: Rolling release; reviewed 2026-09-13.

Locator: Setup; libvirt network autostart; routed traffic.

Review note: sudo virsh net-autostart default

## CACHY-SCOPE

[CachyOS wiki scope](https://wiki.cachyos.org/cachyos_basic/navigation-guide/)

Authority: Official CachyOS documentation. Version/scope: Current wiki.

Locator: Reminder.

Review note: Feel free to refer to Arch Wiki articles when needed

## RHEL-IP

[RHEL Ethernet connections](https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/9/html/configuring_and_managing_networking/configuring-an-ethernet-connection_configuring-and-managing-networking)

Authority: Red Hat product documentation. Version/scope: RHEL 9.

Locator: 2.1 nmcli; 2.8 network RHEL system role.

Review note: Configuring an Ethernet connection by using nmcli

## RHEL-BR

[RHEL network bridges](https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/9/html/configuring_and_managing_networking/configuring-a-network-bridge_configuring-and-managing-networking)

Authority: Red Hat product documentation. Version/scope: RHEL 9.

Locator: 6.1 Configuring a network bridge by using nmcli.

Review note: Configuring a network bridge by using nmcli

## RHEL-ROUTE

[RHEL static routes](https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/9/html/configuring_and_managing_networking/configuring-static-routes_configuring-and-managing-networking)

Authority: Red Hat product documentation. Version/scope: RHEL 9.

Locator: 28.2 nmcli route syntax; 28.3 static route procedure.

Review note: Configuring a static route by using nmcli

## VSCODE

[VS Code Source Control](https://code.visualstudio.com/docs/sourcecontrol/overview)

Authority: Official Microsoft documentation. Version/scope: Current VS Code docs.

Locator: Source Control view; staging and committing.

Review note: Source control in VS Code

## DEVTOOLS

[AAP VS Code setup](https://docs.redhat.com/en/documentation/red_hat_ansible_automation_platform/2.6/install-proc_devtools_install_vsc)

Authority: Red Hat product documentation. Version/scope: AAP 2.6.

Locator: Install and configure VS Code.

Review note: Install and configure VS Code

## GIT

[Git tutorial](https://git-scm.com/docs/gittutorial)

Authority: Official Git documentation. Version/scope: Current Git tutorial.

Locator: Making changes; branches; collaboration.

Review note: Git tracks content not files

## MODEL

[Pinned simulator model](https://github.com/QuasarRay/datacenter-simulator/blob/2003f8bdd88423c77c528785f7ffa70f398a23e0/vars/datacenter.yml)

Authority: User repository source. Version/scope: Commit 2003f8b; blob 0f89fe231473254e3aa4bba6fe87b5285ee69a25.

Locator: dc.nodes and dc.links.

Review note: gpu-dc

## NAV-FAQ

[Navigator FAQ: SSH keys and project mounts](https://docs.ansible.com/projects/navigator/faq/)

Authority: Official Ansible Navigator documentation. Version/scope: Current docs; local Navigator 25.5.0.

Locator: SSH keys; Where should ansible.cfg go with an execution environment.

Review note: Navigator exposes an active SSH agent to the EE. Keep ansible.cfg and required project files adjacent to the root playbook.

