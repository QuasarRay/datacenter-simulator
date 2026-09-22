#!/usr/bin/env python3
"""Wire explicit host trust into pinned netcommon; no global SSH configuration.

ansible-pylibssh 1.2.2 ignores config_file, but supports knownhosts.
Reject an unknown plugin source rather than silently patching a newer release.
This patches an Ansible deployment dependency, not the Rust simulator.
"""
import hashlib
from pathlib import Path
import sys

ORIGINAL = "4c5f06798950539fd9619431e1bd73798fd21e5735413792bda59e6b77cdbe36"
OPTION = """      known_hosts:
        description: Explicit known_hosts file for the pinned pylibssh binding.
        type: path
        vars:
          - name: ansible_libssh_known_hosts
"""
BEFORE = '            if self.get_option("config_file"):'
AFTER = '\n'.join([
    '            if self.get_option("known_hosts"):',
    '                ssh_connect_kwargs["knownhosts"] = self.get_option("known_hosts")',
    '', BEFORE,
])

def patch(root):
    file = Path(root) / "ansible_collections/ansible/netcommon/plugins/connection/libssh.py"
    data = file.read_text()
    if OPTION in data and AFTER in data:
        original = data.replace(OPTION, "", 1).replace(AFTER, BEFORE, 1)
        if hashlib.sha256(original.encode()).hexdigest() != ORIGINAL:
            raise RuntimeError("patched libssh dependency has unexpected edits")
        return
    if hashlib.sha256(data.encode()).hexdigest() != ORIGINAL:
        raise RuntimeError("libssh dependency differs from pinned netcommon 8.1.0")
    if data.count("      config_file:") != 1 or data.count(BEFORE) != 1:
        raise RuntimeError("libssh patch anchors differ")
    file.write_text(data.replace("      config_file:", OPTION + "      config_file:", 1).replace(BEFORE, AFTER, 1))

if __name__ == "__main__":
    patch(sys.argv[1])
