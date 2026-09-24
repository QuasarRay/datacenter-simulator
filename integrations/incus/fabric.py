"""Generate an owned native fabric from NetBox; learner/operator entry is Python.

Repeated devices, ports, addresses, routes and guest configuration are compiled
from intent. No hand-maintained topology or per-node shell script is necessary.
"""
from dataclasses import dataclass
from functools import wraps
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

from .lab import Lab, ROOT

@dataclass(frozen=True)
class Fabric:
    binary: Path
    directory: Path

    def verify(self):
        ready=json.loads((self.directory/'READY.json').read_text())
        if ready['schema']!='ncp-native-plan-v1':raise ValueError('unknown compiled plan')
        expected={'snapshot.json','manifest.json','provisioning-vars.json','incus-config.json','fabric-config.json','native-plan.json'}
        if set(ready['sha256'])!=expected:raise ValueError('incomplete compiled bundle')
        for name,digest in ready['sha256'].items():
            if hashlib.sha256((self.directory/name).read_bytes()).hexdigest()!=digest:
                raise ValueError('compiled intent changed: '+name)
        native=json.loads((self.directory/'incus-config.json').read_text())
        if Path(native['manifest'])!=(self.directory/'manifest.json'):
            raise ValueError('manifest escaped its sealed plan directory')
        return ready

    @classmethod
    def compile(cls, recipe, *, binary, destination, snapshot=None):
        binary=Path(binary).resolve();directory=Path(destination).resolve()
        if directory.exists():raise ValueError('destination must be new')
        # Read-only acquisition is the default. Replay is an explicit development input.
        with tempfile.NamedTemporaryFile(mode='w',suffix='.json') as config:
            json.dump(recipe,config);config.flush()
            argv=[str(binary),'fabric-plan',config.name,str(directory)]
            if snapshot is not None:argv.append(str(Path(snapshot).resolve()))
            subprocess.run(argv,check=True,capture_output=True,timeout=180)
        directory.chmod(0o700)
        result=cls(binary,directory);result.verify();return result

    def deploy(self):
        self.verify()
        subprocess.run([str(self.binary),'incus-up',str(self.directory/'incus-config.json'),
                        str(self.directory/'state')],check=True,capture_output=True,timeout=600)
        return self

    def reconcile(self):
        self.verify()
        variables=json.loads((self.directory/'provisioning-vars.json').read_text())
        return Lab(self.directory/'state').apply(ROOT/'integrations/incus/provision.yml',variables)

    def destroy(self):
        # Owned lifecycle journal remains authoritative even if plan inputs changed.
        subprocess.run([str(self.binary),'incus-down',str(self.directory/'state')],
                       check=True,capture_output=True,timeout=600)

def native_fabric(recipe_function):
    """Turn a parameterized intent function into a compiler entry point.

    @native_fabric removes repeated acquisition/render/seal plumbing. The function
    returns ordinary typed JSON data; it cannot inject executable source into a guest.
    Generated bundles remain inspectable before .deploy(), with no extra approval flow.
    """
    @wraps(recipe_function)
    def compile_recipe(*args,binary,destination,snapshot=None,**kwargs):
        return Fabric.compile(recipe_function(*args,**kwargs),binary=binary,
                              destination=destination,snapshot=snapshot)
    return compile_recipe
