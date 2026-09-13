# 13 — Transfer to other network platforms

## Preparation scope

Red Hat's preparation guidance calls for familiarity with VNFs from multiple vendors, including Cisco, Juniper and Arista. FRR practice alone does not establish that familiarity. Repeat inventory, facts, Jinja rendering, netcommon commands, backup and verification on the vendor platforms available in your assigned lab.

Keep the desired-state model and acceptance criteria. Replace `ansible_network_os`, platform-specific facts, prompt/privilege handling, templates and output parsing according to the installed vendor collection. Verify whether the transport is `network_cli`, NETCONF or HTTPAPI and whether candidate commit/rollback semantics apply. Do not paste FRR `no` commands into another vendor's configuration.

[EX457 public objectives](https://www.redhat.com/en/services/training/ex457-red-hat-certified-specialist-in-ansible-network-automation-exam). [network_cli connection](https://docs.ansible.com/projects/ansible/latest/collections/ansible/netcommon/network_cli_connection.html).

