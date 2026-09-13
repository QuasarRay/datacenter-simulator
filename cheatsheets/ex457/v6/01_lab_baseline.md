# 01 — Lab baseline and key trust

## Bootstrap

Prerequisites: a Linux lab host with Docker, Containerlab **0.79.0**, Python 3.11+ and OpenSSH tools; at least four router containers worth of memory. Use a dedicated lab host/VM. The management subnet is `172.30.0.0/24`; check that it does not overlap an existing VPN or container network.

```bash
python3 -m venv .venv
source .venv/bin/activate
python -m pip install -r requirements.txt
ansible-galaxy collection install -r requirements.yml -p collections
export ANSIBLE_COLLECTIONS_PATH="$PWD/collections"
ansible-inventory --graph
```

FRR base selection is a required runtime input. The v4 tag `quay.io/frrouting/frr:containerlab-10.7.1` is a **candidate to inspect**, not an asserted registry artifact. Pull/inspect a published containerlab-flavor FRR 10.7 image, record its RepoDigest and version, then provide its `name@sha256:...` reference. `labctl` rejects an unpinned base. The upstream Containerlab Dockerfile establishes the image flavor, not a registry digest.

[Containerlab Linux kind](https://containerlab.dev/manual/kinds/linux/). [FRR Containerlab image source](https://github.com/FRRouting/frr/blob/1286d6389626a77648ed844b176c2324466c1371/docker/containerlab/Dockerfile).

## Build and deploy

```bash
read -r -p 'Inspected FRR containerlab image digest: ' FRR_BASE
python tools/labctl.py build-image --base "$FRR_BASE"
mkdir -p .state/client
ssh-keygen -t ed25519 -f .state/client/id_ed25519
python tools/labctl.py prepare --public-key .state/client/id_ed25519.pub
python tools/labctl.py preflight
python tools/labctl.py deploy
export ANSIBLE_PRIVATE_KEY_FILE="$PWD/.state/client/id_ed25519"
# Use an active SSH agent; start one first if this shell has none.
ssh-add "$ANSIBLE_PRIVATE_KEY_FILE"
ssh -tt -i "$ANSIBLE_PRIVATE_KEY_FILE"   -o UserKnownHostsFile="$PWD/.state/known_hosts"   -o StrictHostKeyChecking=yes ansible@172.30.0.11
# Inside vtysh: show version ; exit
```

For a passphrase-protected client key, load it into an active SSH agent before Ansible runs; use `eval "$(ssh-agent -s)"` to start an agent in a shell that has none.

`prepare` creates directories before files, generates each host key only once, exports public trust directly from those keys, and renders topology. Preflight checks bind-file existence, key modes, public/private correspondence, trust, image presence and Containerlab version. Protect and retain `.state/hostkeys` between redeploys.

The Dockerfile uses Alpine `passwd -d ansible` to remove the lock, while sshd disables password, empty-password and keyboard-interactive authentication. This is a lab adapter: membership in `frrvty` grants configuration access. Prove account status and key-only login using chapter 20.

[OpenSSH sshd](https://man.openbsd.org/sshd). [FRR integrated configuration](https://docs.frrouting.org/en/stable-10.7/vtysh.html).

## Backend trust

The network playbooks select **libssh**, set host-key checking explicitly, and create a temporary SSH config through `ansible_libssh_config_file`. That config points `UserKnownHostsFile` to the verified public trust file. Default local trust is `.state/known_hosts`; Controller supplies an absolute path in `EX457_KNOWN_HOSTS` through a custom credential.

Manual OpenSSH login and `network_cli` are separate smoke tests. No OpenSSH common-args setting is used as evidence of network_cli trust. A missing trust file fails before connection. Test that a deliberately wrong server key is rejected. Temporary backend config is removed after resetting the persistent connection.

[network_cli connection](https://docs.ansible.com/projects/ansible/latest/collections/ansible/netcommon/network_cli_connection.html). [libssh connection](https://docs.ansible.com/projects/ansible/latest/collections/ansible/netcommon/libssh_connection.html). [libssh pinned implementation](https://github.com/ansible-collections/ansible.netcommon/blob/v8.1.0/plugins/connection/libssh.py).

