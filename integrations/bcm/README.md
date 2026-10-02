# Real NVIDIA Base Command Manager 11 integration

This integration installs **actual NVIDIA Base Command Manager 11** on a real
supported Linux head node by invoking NVIDIA/Bright Computing's official
`brightcomputing.installer110.head_node` Ansible role. It does not emulate
CMDaemon, fabricate BCM API responses, or substitute another dashboard for
Base View.

BCM 11's supported add-on method installs the BCM head-node software onto an
already-running supported distribution. NVIDIA documents the official
`brightcomputing.installer110` collection as the recommended automation path.
This repository therefore treats the BCM head node as a full machine or full VM,
not an Incus system container. The rest of the datacenter-simulator can continue
to use Incus system containers; the exception exists because fidelity is more
important than pretending BCM supports a deployment shape NVIDIA does not claim.

## What is installed

`install.yml` performs three real operations:

1. invokes `brightcomputing.installer110.head_node` against the target;
2. installs the real BCM 11 `base-view` package from the repositories configured
   by the NVIDIA installer;
3. requires `cmd.service` to be running and probes the HTTPS Base View endpoint
   on port 8081.

Base View is therefore the GUI shipped by BCM itself. No local mock UI exists.

## Prerequisites

Use a fresh full machine or full VM running a BCM-11-supported Linux distribution.
The repository baseline is Ubuntu 24.04. Give the head node both an external and
a management interface and enough storage for BCM plus its generated compute-node
software image.

Obtain a BCM entitlement/product key through NVIDIA and download the BCM ISO that
matches the target distribution. The ISO and product key are licensed inputs and
are deliberately not downloaded or redistributed by this repository.

On the Ansible control node, install the pinned Ansible Core release and
collections. The explicit `ansible.posix`, `community.crypto`,
`community.general`, and `community.mysql` entries are intentional: the
current installer110 role calls modules from those collections while its
published Galaxy metadata declares no collection dependencies. `jmespath`
is pinned because installer110 also uses `community.general.json_query`.

The current installer110 artifact also publishes
`requires_ansible: ">=8.3"`. Ansible interprets that metadata field against
the 2.x `ansible-core` version, so it emits a compatibility warning even with
a syntactically valid tested core. CI records the upstream metadata and tests
this integration against the explicit core pin instead of hiding that warning.

```bash
python -m pip install -r integrations/bcm/requirements-control-node.txt
ansible-galaxy collection install -r integrations/bcm/requirements.yml
```

Copy the examples and fill in site-specific values:

```bash
cp integrations/bcm/inventory.example.ini integrations/bcm/inventory.ini
cp integrations/bcm/group_vars/head_node/credentials.example.yml \
   integrations/bcm/group_vars/head_node/credentials.yml
ansible-vault encrypt integrations/bcm/group_vars/head_node/credentials.yml
```

Edit `group_vars/head_node/settings.yml` for the target interfaces, management
network, DNS, certificate subject, and the path where the licensed BCM ISO has
already been staged on the target head node.

Run the actual installation:

```bash
ansible-playbook \
  -i integrations/bcm/inventory.ini \
  --ask-vault-pass \
  integrations/bcm/install.yml
```

A successful run ends only after the official installer has converged, the real
`base-view` package is present, `cmd.service` is active, TCP/8081 is listening,
and an HTTPS request reaches `/base-view`.

## Base View

After installation, browse to:

```text
https://<bcm-head-node>:8081/base-view
```

The default BCM certificate may not yet be trusted by the browser. Replace it
with site PKI before treating the endpoint as production hardened.

## Fidelity boundary

This change supplies a real BCM control plane and its real GUI. It does **not**
claim that Incus/KWOK workers have become BCM-provisioned physical compute nodes.
PXE boot, BMC/Redfish control, GPU/NVSwitch hardware, InfiniBand payload traffic
and physical provisioning remain separate integration/qualification work.

A CI syntax check can prove that the automation is structurally valid; only a
licensed run on a supported head node can prove that BCM and Base View actually
installed and started for a particular release/media combination.

Official references:

- NVIDIA BCM 11 Installation Manual:
  https://docs.nvidia.com/base-command-manager/manuals/11/installation-manual.pdf
- NVIDIA BCM documentation:
  https://docs.nvidia.com/base-command-manager/
- Official installer collection:
  https://galaxy.ansible.com/ui/repo/published/brightcomputing/installer110/
