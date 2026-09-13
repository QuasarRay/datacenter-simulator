#!/usr/bin/env python3
"""Run shipped verifier tasks with synthetic command I/O under real Ansible.

Only device I/O and retry timing are replaced. Definitions, loops, registered
results, includes and assertions are copied from the shipped tasks on each run.
"""
import copy
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import yaml
ROOT = Path(__file__).resolve().parents[1]


def main():
    results=[]
    for scenario in ['good','undefined_remote_nodes','wrong_interface','extra_idle_peer','uninstalled_route']:
        with tempfile.TemporaryDirectory(prefix='dataflow-', dir=ROOT/'.state') as temp:
            folder=Path(temp)
            for name in ['model.yml','verify.yml','verify_remote.yml']:
                tasks=yaml.safe_load((ROOT/'playbooks/tasks'/name).read_text())
                for task in tasks:
                    if 'ansible.netcommon.cli_command' in task:
                        command=task.pop('ansible.netcommon.cli_command')['command']
                        task['ansible.builtin.command']={'argv':[sys.executable,str(ROOT/'tests/fixture_device.py'),'{{ inventory_hostname }}',command,scenario]}
                        if 'retries' in task:
                            task['retries']=1;task['delay']=0
                (folder/name).write_text(yaml.safe_dump(tasks,sort_keys=False))
            pre=[{'ansible.builtin.import_tasks':'model.yml'}]
            if scenario=='undefined_remote_nodes':
                pre.append({'ansible.builtin.set_fact':{'fabric_intent':"{{ fabric_intent | dict2items | rejectattr('key', 'equalto', 'remote_nodes') | items2dict }}"}})
            play=[{'name':'Synthetic verifier dataflow '+scenario,'hosts':'spine1','gather_facts':False,
                   'vars_files':[str(ROOT/'model-upstream.yml')],'pre_tasks':pre,
                   'tasks':[{'ansible.builtin.import_tasks':'verify.yml'}]}]
            (folder/'run.yml').write_text(yaml.safe_dump(play,sort_keys=False))
            env=dict(os.environ,ANSIBLE_FILTER_PLUGINS=str(ROOT/'filter_plugins'),ANSIBLE_LOCAL_TEMP=str(ROOT/'.state/ansible-tmp'))
            run=subprocess.run(['ansible-playbook','-i','spine1,','-c','local',str(folder/'run.yml')],cwd=ROOT,env=env,text=True,capture_output=True)
            expected_success=scenario=='good'
            output = run.stdout + run.stderr
            expected_error = {'undefined_remote_nodes': 'remote_nodes',
                              'wrong_interface': 'managed config mismatch: interfaces',
                              'extra_idle_peer': 'Wait for exact unfiltered peer set',
                              'uninstalled_route': 'no selected installed route'}
            passed = (run.returncode == 0) if expected_success else (run.returncode != 0 and expected_error[scenario] in output and 'ASN range/type' not in output)
            log=ROOT/'.state/dataflow-logs'/('dataflow-'+scenario+'.log');log.parent.mkdir(parents=True,exist_ok=True)
            log.write_text(run.stdout+run.stderr)
            results.append({'scenario':scenario,'expected':'success' if expected_success else 'failure','returncode':run.returncode,'status':'PASS' if passed else 'FAIL','log':str(log.relative_to(ROOT))})
    report={'check':'real_ansible_verifier_with_synthetic_device_io','status':'PASS' if all(r['status']=='PASS' for r in results) else 'FAIL','scenarios':results}
    print(json.dumps(report,indent=2))
    return 0 if report['status']=='PASS' else 1


if __name__=='__main__':
    (ROOT/'.state').mkdir(exist_ok=True)
    sys.exit(main())
