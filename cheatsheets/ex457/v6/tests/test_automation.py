import ast
import copy
import hashlib
import json
import os
from pathlib import Path
import stat
import subprocess
import sys
import tempfile
import types
import urllib.request
import pytest
import yaml
from hypothesis import given, strategies as st
from jinja2 import Environment, sandbox, StrictUndefined
from controller_graph import graph_ok, EDGES
from state import secure_dir
ROOT=Path(__file__).resolve().parents[1]


def tasks(): return yaml.safe_load((ROOT/'controller/configure.yml').read_text())[0]['tasks']


def graph():
    edges_by_name={'Backup':{'success_nodes':['Change'],'failure_nodes':['Diagnose'],'always_nodes':[]}, 'Change':{'success_nodes':['Verify'],'failure_nodes':['Diagnose'],'always_nodes':[]}, 'Verify':{'success_nodes':[],'failure_nodes':['Diagnose'],'always_nodes':[]}, 'Diagnose':{'success_nodes':[],'failure_nodes':[],'always_nodes':[]}}
    ids={name:i+1 for i,name in enumerate(edges_by_name)}
    return [{'id':ids[name], 'identifier':name, 'summary_fields':{'unified_job_template':{'name':'EX457 '+name}},
             **{key:[ids[x] for x in targets] for key,targets in edges.items()}} for name,edges in edges_by_name.items()]


@given(st.sampled_from(['extra_node','diagnose_edge','wrong_template','missing_node','outside_edge']))
def test_exact_controller_graph(defect):
    nodes=graph(); assert graph_ok(nodes)
    if defect=='extra_node': nodes.append({'id':99,'identifier':'Rogue'})
    elif defect=='diagnose_edge': nodes[-1]['always_nodes']=[1]
    elif defect=='wrong_template': nodes[0]['summary_fields']['unified_job_template']['name']='Deploy production'
    elif defect=='missing_node': nodes.pop()
    else: nodes[0]['success_nodes']=[99]
    with pytest.raises((ValueError, KeyError)): graph_ok(nodes)


def test_machine_passphrase_and_privilege_contract():
    machine=next(x['awx.awx.credential'] for x in tasks() if x.get('name')=='Create Machine identity')
    template=Environment(undefined=StrictUndefined).from_string(machine['inputs']['ssh_key_unlock'])
    assert template.render(machine_key_passphrase='synthetic phrase',omit='omit')=='synthetic phrase'
    assert template.render(omit='omit')=='omit'
    prerequisite=next(x['ansible.builtin.assert'] for x in tasks() if x.get('name')=='Require privilege for initial custom credential type provisioning')
    env=Environment(); env.filters['bool']=bool
    expression=env.compile_expression(prerequisite['that'][1])
    for superuser,preprovisioned in [(True,False),(False,True),(False,False)]:
        allowed=expression(controller_identity={'json':{'results':[{'is_superuser':superuser}]}}, credential_type_preprovisioned=preprovisioned)
        assert bool(allowed)==(superuser or preprovisioned)


def test_real_ansible_trust_bootstrap(tmp_path):
    trust=tmp_path/'known_hosts'; trust.write_text('172.30.0.11 ssh-ed25519 fixture-public-only\n')
    # No SSH happens: this checks real network_cli variable evaluation and file lifecycle.
    play=[{'hosts':'spine1','gather_facts':False,'vars_files':[str(ROOT/'model-upstream.yml')],
           'tasks':[{'ansible.builtin.import_tasks':str(ROOT/'playbooks/tasks/model.yml')},
                    {'ansible.builtin.import_tasks':str(ROOT/'playbooks/tasks/trust.yml')},
                    {'ansible.builtin.assert':{'that':["ansible_libssh_config_file == ex457_ssh_config.path", "ansible_libssh_config_file | length > 0"]}},
                    {'ansible.builtin.stat':{'path':'{{ ansible_libssh_config_file }}'},'delegate_to':'localhost','register':'backend'},
                    {'ansible.builtin.assert':{'that':["backend.stat.isreg", "backend.stat.mode == '0600'"]}},
                    {'ansible.builtin.import_tasks':str(ROOT/'playbooks/tasks/cleanup_trust.yml')}]}]
    file=tmp_path/'bootstrap.yml'; file.write_text(yaml.safe_dump(play))
    secure_dir(ROOT/'.state')
    env=dict(os.environ,EX457_KNOWN_HOSTS=str(trust),ANSIBLE_FILTER_PLUGINS=str(ROOT/'filter_plugins'),ANSIBLE_LOCAL_TEMP=str(ROOT/'.state/ansible-tmp'))
    run=subprocess.run(['ansible-playbook','-i',str(ROOT/'inventory/network.yml'),str(file)],cwd=ROOT,env=env,text=True,capture_output=True)
    assert run.returncode==0,run.stdout+run.stderr
    assert 'failed=0' in run.stdout
    (ROOT/'.state/bootstrap.log').write_text(run.stdout+run.stderr)


def test_invalid_public_key_rejected_before_state_writes(tmp_path, monkeypatch):
    import labctl
    key=tmp_path/'invalid.pub';key.write_text('ssh-ed25519 NOT-BASE64\n')
    target=tmp_path/'state';monkeypatch.setattr(labctl,'STATE',target)
    with pytest.raises(RuntimeError):labctl.prepare(key)
    assert not target.exists()


def test_pinned_awx_injector(tmp_path):
    cache=secure_dir(ROOT/'.state/upstream')/'awx-credential-24.6.1.py'
    expected='3f5655abda1b2a5d3a24d1c7e3e42fce3884c4b6c7007bccf2ea9d2b60d8ee3d'
    if not cache.exists():
        with urllib.request.urlopen('https://raw.githubusercontent.com/ansible/awx/24.6.1/awx/main/models/credential/__init__.py',timeout=30) as response: cache.write_bytes(response.read())
    assert hashlib.sha256(cache.read_bytes()).hexdigest()==expected,'upstream source identity changed'
    module=ast.parse(cache.read_text());method=next(n for n in ast.walk(module) if isinstance(n,ast.FunctionDef) and n.name=='inject_credential')
    ns={'sandbox':sandbox,'tempfile':tempfile,'os':os,'stat':stat,'to_container_path':lambda p,base:'/runner/'+str(Path(p).relative_to(base)),'ValidationError':ValueError}
    exec(compile(ast.fix_missing_locations(ast.Module(body=[method],type_ignores=[])),str(cache),'exec'),ns)
    spec=next(x['awx.awx.credential_type'] for x in tasks() if 'awx.awx.credential_type' in x)
    def render(value):
        if isinstance(value,str):return Environment(undefined=StrictUndefined).from_string(value).render(credential_filename_namespace='tower')
        if isinstance(value,dict):return {k:render(v) for k,v in value.items()}
        if isinstance(value,list):return [render(v) for v in value]
        return value
    spec=render(spec)
    class Field:
        def validate_env_var_allowed(self,value):assert value.startswith('EX457_')
    values={'known_hosts':'SYNTHETIC PUBLIC TRUST\n','backup_token':'SYNTHETIC TOKEN'}
    credential=types.SimpleNamespace(inputs=values,dynamic_input_fields=[],get_input=lambda key:values[key])
    target=types.SimpleNamespace(injectors=spec['injectors'],managed=False,inputs=spec['inputs'],secret_fields=['backup_token'],_meta=types.SimpleNamespace(get_field=lambda name:Field()))
    (tmp_path/'env').mkdir();env={};safe={}
    ns['inject_credential'](target,credential,env,safe,[],str(tmp_path))
    assert env['EX457_KNOWN_HOSTS'].startswith('/runner/env/') and safe['EX457_BACKUP_TOKEN']=='**********'
    actual=tmp_path/Path(env['EX457_KNOWN_HOSTS']).relative_to('/runner')
    assert actual.read_text()==values['known_hosts']
    target.injectors['env']['EX457_KNOWN_HOSTS']='{{ awx.filename }}'
    with pytest.raises(Exception,match='awx'):ns['inject_credential'](target,credential,{}, {}, [],str(tmp_path))


def test_pyats_failure_has_nonzero_process_exit(tmp_path):
    probe=tmp_path/'probe.py'
    source="from pyats import aetest\nclass Failure(aetest.Testcase):\n @aetest.test\n def injected(self):self.failed('deliberate regression probe')\n"
    # Exercise exactly the process-exit block shipped in the live suite.
    block=(ROOT/'tests/pyats_live.py').read_text().split("if __name__=='__main__':",1)[1]
    probe.write_text(source+"if __name__=='__main__':"+block)
    result=subprocess.run([sys.executable,str(probe)],capture_output=True,text=True)
    assert result.returncode!=0 and 'deliberate regression probe' in result.stdout
