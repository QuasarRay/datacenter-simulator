#!/usr/bin/env python3
"""D006-D008 regressions against the real CLI with isolated fake external tools.
No Kubernetes deployment or GPU execution occurs. Build --features mokka first.
"""
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
repo=Path(sys.argv[1]).resolve()
def report(case, **values):
    print(json.dumps({'case':case, **values}), flush=True)

def mokka(case):
    with tempfile.TemporaryDirectory(prefix='audit-mokka-') as name:
        root=Path(name); bindir=root/'bin'; bindir.mkdir()
        config=json.loads((repo/'integrations/k8s-test-infra/config.example.json').read_text())
        config['manifest']=str(repo/'integrations/netbox/manifest.example.json')
        config['upstream']=str(repo/'integrations/k8s-test-infra/upstream')
        config['image']+='@sha256:'+'a'*64
        config['nodes']={'nb1': {'kubernetes_node':'simulator-worker','profile':'t4','gpu_count':2}}
        (root/'config.json').write_text(json.dumps(config))
        capacity={f'nvidia.com/{case}':'7'} if case in ('mig-1g.5gb', 'gpu.shared') else {}
        node={'metadata':{'name':'simulator-worker','labels':{'kubernetes.io/hostname':'simulator-worker','simulator.quasarray.io/mokka':'true'}},
              'status':{'conditions':[{'type':'Ready','status':'True'}],'capacity':capacity}}
        (root/'node.json').write_text(json.dumps(node))
        pods={'items':[{'metadata':{'name':'test-pod'},'spec':{'nodeName':'simulator-worker','containers':[{'name':'node-agent','image':config['image']}]},
                       'status':{'containerStatuses':[{'name':'node-agent','ready':True,'imageID':'containerd://sha256:fixture'}]}}]}
        (root/'pods.json').write_text(json.dumps(pods))
        kubectl='''#!/bin/sh
printf '%s\\n' "$*" >> "$AUDIT_FIXTURES/kubectl-calls"
if [ "$AUDIT_CASE" = descendant ]; then
  sleep 30 &
  echo $! > "$AUDIT_FIXTURES/descendant.pid"
  printf '{"metadata":{"name":"simulator-gpus"},"status":{"phase":"Active"}}'
  exit 0
fi
case "$*" in
  *"get namespace "*) printf '{"metadata":{"name":"simulator-gpus"},"status":{"phase":"Active"}}';;
  *"get node "*) cat "$AUDIT_FIXTURES/node.json";;
  *"wait --for=condition=Ready pod "*) exit 0;;
  *"get pods "*) cat "$AUDIT_FIXTURES/pods.json";;
  *"exec test-pod "*)
    if [ "$AUDIT_CASE" = report_failure ]; then mkdir "$AUDIT_FIXTURES/state/result.json"; fi
    printf 'GPU-1, T4\\nGPU-2, T4\\n';;
  *) exit 98;;
esac
'''
        helm='''#!/bin/sh
printf '%s\\n' "$*" >> "$AUDIT_FIXTURES/helm-calls"
case "$*" in
  "list "*"--selector "*) printf '[{"name":"sim-gpu-0"}]';;
  "list "*) printf '[]';;
  "install "*) touch "$AUDIT_FIXTURES/installed";;
  "uninstall "*) touch "$AUDIT_FIXTURES/uninstalled";;
  *) exit 97;;
esac
'''
        for tool,script in [('kubectl',kubectl),('helm',helm)]:
            (bindir/tool).write_text(script); (bindir/tool).chmod(0o755)
        proc=subprocess.run([str(repo/'simulator/target/debug/datacenter-simulator'),'mokka-apply',str(root/'config.json'),str(root/'state')],
            env={**os.environ,'PATH':str(bindir)+':'+os.environ['PATH'],'AUDIT_FIXTURES':str(root),'AUDIT_CASE':case},
            capture_output=True,text=True,timeout=15)
        installed=(root/'installed').exists(); uninstalled=(root/'uninstalled').exists()
        state=json.loads((root/'state/batch-state.json').read_text()) if (root/'state/batch-state.json').exists() else None
        values={'exit':proc.returncode,'installed':installed,'uninstalled':uninstalled,'state':state['status'] if state else None,'stderr':proc.stderr.strip()}
        if case in ('mig-1g.5gb', 'gpu.shared'):
            assert proc.returncode!=0 and not installed
            values['advertised_capacity']=capacity
        elif case=='report_failure':
            assert proc.returncode!=0 and installed and uninstalled and state['status']=='rolled-back'
        else:
            pid=int((root/'descendant.pid').read_text())
            # A killed orphan may briefly remain a zombie until init reaps it.
            def running():
                try: return Path(f'/proc/{pid}/stat').read_text().split()[2] != 'Z'
                except FileNotFoundError: return False
            end=time.monotonic()+2
            while running() and time.monotonic()<end: time.sleep(.01)
            alive=running()
            try: assert proc.returncode!=0 and not alive
            finally:
                if alive:
                    try: os.kill(pid,signal.SIGKILL)
                    except ProcessLookupError: pass
            values['descendant_survived_cli_exit']=alive
        report('mokka_'+case,**values)

for case in ['mig-1g.5gb','gpu.shared','report_failure','descendant']:
    mokka(case)
