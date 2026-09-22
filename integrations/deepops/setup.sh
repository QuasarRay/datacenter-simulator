#!/usr/bin/env bash
# SPDX-License-Identifier: RPL-1.5
# Run as a regular user on a dedicated Linux provisioning machine.
set -euo pipefail
repo=$(git rev-parse --show-toplevel)
git -C "$repo" submodule update --init --recursive
cd "$repo/integrations/deepops/upstream"
./scripts/setup.sh </dev/null
"$repo/integrations/deepops/install-galaxy-locked.sh" /opt/deepops/env/bin/ansible-galaxy "$PWD/roles/galaxy" "$PWD/collections" "$repo/integrations/deepops/.galaxy-cache"
