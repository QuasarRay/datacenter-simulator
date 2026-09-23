#!/usr/bin/env python3
"""Run root deploy.yml against fake CLI tools inside a temporary checkout fragment.
No Docker/Containerlab operations occur. Exit zero confirms the deployed descriptor was preserved.
Usage: python deploy_probe.py /path/to/repo /path/to/ansible-playbook
"""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import yaml

repo=Path(sys.argv[1]).resolve()
ansible=Path(sys.argv[2]).resolve()
with tempfile.TemporaryDirectory(prefix='audit-deploy-') as temp:
    root=Path(temp)
    for dirname in ['playbooks','templates','vars','build','bin']:
        (root/dirname).mkdir()
    for relative in ['playbooks/deploy.yml','templates/datacenter.clab.yml.j2','vars/datacenter.yml']:
        shutil.copy2(repo/relative,root/relative)
    dc=yaml.safe_load((root/'vars/datacenter.yml').read_text())['dc']
    target=root/'build'/(dc['name']+'.clab.yml')
    original='name: '+dc['name']+'\n# ORIGINAL DEPLOYED TOPOLOGY\n'
    target.write_text(original)
    (root/'bin/containerlab').write_text('#!/bin/sh\nprintf "%s\\n" "$*" >> "$AUDIT_CALLS"\nexit 0\n')
    (root/'bin/docker').write_text('#!/bin/sh\nprintf "clab-existing-node\\n"\n')
    for tool in ['containerlab','docker']:
        (root/'bin'/tool).chmod(0o755)
    (root/'inventory.ini').write_text('[labhost]\nlocalhost ansible_connection=local\n')
    result=subprocess.run([str(ansible),'-i',str(root/'inventory.ini'),str(root/'playbooks/deploy.yml'),'-e','ansible_become=false'],
        cwd=root,env={**os.environ,'PATH':str(root/'bin')+':'+str(ansible.parent)+':'+os.environ['PATH'],
        'AUDIT_CALLS':str(root/'calls'),'ANSIBLE_NOCOLOR':'1'},capture_output=True,text=True,timeout=40)
    assert result.returncode!=0
    assert 'Lab resources already exist' in result.stdout
    assert target.read_text()==original
    calls=(root/'calls').read_text().splitlines() if (root/'calls').exists() else []
    assert calls==[]
    print(json.dumps({'exit':result.returncode,'existing_lab_refused':True,'deployed_topology_descriptor_overwritten':False,'fake_containerlab_calls':calls}))
