"""Prepare a delegated cgroup v2 subtree inside the disposable Dagger runner.

Adapted behavior from Moby hack/dind (Apache-2.0): move root processes to an
init child before enabling subtree controllers. Never run on a production host.
"""
import errno
import json
import os
from pathlib import Path
import time


def prepare(root=Path('/sys/fs/cgroup')):
    root=Path(root)
    if not (root/'cgroup.controllers').is_file():
        return {'cgroup_version':1,'action':'no v2 delegation required'}
    before={name:(root/name).read_text().strip() for name in ['cgroup.type','cgroup.controllers','cgroup.subtree_control']}
    controllers=before['cgroup.controllers'].split()
    if not {'cpu','memory'} <= set(controllers):
        raise RuntimeError('Dagger runner must delegate CPU and memory controllers: '+json.dumps(before))
    init=root/'ex457-ci-init';init.mkdir(exist_ok=True)
    for attempt in range(50):
        for pid in (root/'cgroup.procs').read_text().split():
            try:(init/'cgroup.procs').write_text(pid+'\n')
            except OSError as error:
                if error.errno!=errno.ESRCH:raise
        try:
            (root/'cgroup.subtree_control').write_text(' '.join('+'+c for c in controllers)+'\n')
            break
        except OSError as error:
            if error.errno!=errno.EBUSY or attempt==49:raise
            time.sleep(0.1)
    after=(root/'cgroup.subtree_control').read_text().split()
    if not {'cpu','memory'} <= set(after):raise RuntimeError('CPU/memory subtree delegation failed')
    return {'cgroup_version':2,'before':before,'enabled':after}


if __name__=='__main__':
    if os.environ.get('EX457_DAGGER_RUNNER')!='1':
        raise SystemExit('Run only inside the disposable Dagger runner')
    print(json.dumps(prepare(),indent=2))
