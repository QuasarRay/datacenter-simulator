"""Code-first trainer API. The public API contains no interactive command parsing."""
import hashlib
import json
import os
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DEEPOPS = 'dc80499ea34b3f36563ed039421076de15071517'

class Lab:
    def __init__(self, state):
        self.state = Path(state).resolve()
        self.journal = json.loads((self.state/'incus.json').read_text())
        if self.journal['phase'] != 'active':
            raise ValueError('expected an active Incus lab')

    def inventory(self):
        nodes = {node['name']: {'ansible_connection':'ncp_incus','ansible_python_interpreter':'/usr/bin/python3',
                              'ncp_declared_os':node['spec']['config'].get('user.ncp.os','cachyos'),
                              'ncp_hosts_file_src':str(self.state/'host-identities'/node['name'])}
                 for node in self.journal['plan']['nodes']}
        return {'all': {'hosts': nodes}}

    def apply(self, playbook, variables=None, timeout=7200):
        """Run reviewed automation against exactly this lab, recording identity and exit status."""
        playbook = Path(playbook).resolve()
        source = ROOT/'integrations/deepops/upstream'
        revision = subprocess.check_output(['git','-C',str(source),'rev-parse','HEAD'],text=True).strip()
        if revision != DEEPOPS:
            raise ValueError('DeepOps revision mismatch')
        if subprocess.check_output(['git','-C',str(source),'status','--porcelain','--untracked-files=no'],text=True).strip():
            raise ValueError('DeepOps tracked source is modified')
        # Identity is desired state, not a template fed by its previous resolver output.
        # The real DeepOps role supports an explicitly supplied custom hosts file.
        identities = self.state/'host-identities'
        identities.mkdir(exist_ok=True)
        for node in self.inventory()['all']['hosts']:
            if not node or any(c not in 'abcdefghijklmnopqrstuvwxyz0123456789-' for c in node):
                raise ValueError('invalid owned hostname')
            (identities/node).write_text('127.0.0.1 localhost\n127.0.1.1 '+node+'\n::1 localhost ip6-localhost ip6-loopback\n')
        inventory = self.state/'inventory.json'
        inventory.write_text(json.dumps(self.inventory(),indent=2)+'\n')
        extra = self.state/'variables.json'
        extra.write_text(json.dumps(variables or {},indent=2)+'\n')
        env = os.environ | {'NCP_JOURNAL':str(self.state/'incus.json'),
          'ANSIBLE_CONNECTION_PLUGINS':str(ROOT/'integrations/incus/connection_plugins'),
          'ANSIBLE_ROLES_PATH':str(source/'roles'), 'ANSIBLE_PIPELINING':'false',
          'ANSIBLE_NOCOLOR':'1', 'ANSIBLE_RETRY_FILES_ENABLED':'false'}
        result = subprocess.run(['ansible-playbook','-i',str(inventory),str(playbook),
          '--limit',','.join(self.inventory()['all']['hosts']), '-e','@'+str(extra)],
          env=env,capture_output=True,timeout=timeout)
        report = {'scope':'upstream-deepops-and-site-adapter','deepops_revision':revision,
          'playbook_sha256':hashlib.sha256(playbook.read_bytes()).hexdigest(),
          'inventory_sha256':hashlib.sha256(inventory.read_bytes()).hexdigest(),
          'variables_sha256':hashlib.sha256(extra.read_bytes()).hexdigest(),
          'returncode':result.returncode,'stdout':result.stdout.decode(errors='replace'),
          'stderr':result.stderr.decode(errors='replace'),'live_workload_validated':False}
        (self.state/'provisioning.json').write_text(json.dumps(report,indent=2)+'\n')
        result.check_returncode()
        return report

    def reconcile_hosts(self, timeout=7200):
        """Use the real pinned DeepOps hosts role, on every teaching lab."""
        return self.apply(ROOT/'integrations/incus/provision.yml', timeout=timeout)
