"""Ansible connection via the trainer's owned Incus journal."""
import importlib.util
import os
from pathlib import Path
from ansible.errors import AnsibleConnectionFailure
from ansible.plugins.connection import ConnectionBase

DOCUMENTATION = r'''
name: ncp_incus
short_description: Execute Ansible inside an explicitly owned Incus system container
description:
  - Uses the Incus REST API and a trainer-side ownership journal.
author: datacenter-simulator contributors
options:
  remote_addr:
    description: Instance name from the generated inventory.
    default: inventory_hostname
    vars:
      - name: ansible_host
'''

class Connection(ConnectionBase):
    transport = 'ncp_incus'
    has_pipelining = False

    def _connect(self):
        if not self._connected:
            spec = importlib.util.spec_from_file_location('ncp_transport', Path(__file__).resolve().parents[1]/'transport.py')
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            self.client = module.Incus(os.environ['NCP_JOURNAL'])
            self.node = self._play_context.remote_addr
            self.client.checked(self.node)
            self._connected = True
        return self

    def exec_command(self, cmd, in_data=None, sudoable=True):
        super().exec_command(cmd, in_data=in_data, sudoable=sudoable)
        self._connect()
        if in_data:
            raise AnsibleConnectionFailure('pipelining is unsupported; transfer the module first')
        return self.client.execute(self.node, ['/bin/sh', '-c', cmd])

    def put_file(self, in_path, out_path):
        super().put_file(in_path, out_path)
        self._connect()
        self.client.put(self.node, out_path, Path(in_path).read_bytes())

    def fetch_file(self, in_path, out_path):
        super().fetch_file(in_path, out_path)
        self._connect()
        Path(out_path).write_bytes(self.client.get(self.node, in_path))

    def close(self):
        self._connected = False
