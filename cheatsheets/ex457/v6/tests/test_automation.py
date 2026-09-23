import ast
import copy
import hashlib
import importlib.util
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
    # Network SSH and wrong-key rejection are separate live CI gates.
    play=[{'hosts':'spine1','gather_facts':False,'vars_files':[str(ROOT/'model-upstream.yml')],
           'tasks':[{'ansible.builtin.import_tasks':str(ROOT/'playbooks/tasks/model.yml')},
                    {'ansible.builtin.import_tasks':str(ROOT/'playbooks/tasks/trust.yml')},
                    {'ansible.builtin.assert':{'that':["ansible_libssh_known_hosts == ex457_trust_path", "ansible_libssh_known_hosts | length > 0"]}},
                    {'ansible.builtin.stat':{'path':'{{ ansible_libssh_known_hosts }}'},'delegate_to':'localhost','register':'backend'},
                    {'ansible.builtin.assert':{'that':["backend.stat.isreg"]}},
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


def test_simulator_profile_images_match_published_digest():
    repository=ROOT.parents[2]
    dc=yaml.safe_load((repository/'vars/datacenter.yml').read_text())['dc']
    publication=json.loads((ROOT/'evidence/image-publication.json').read_text())['replacement']
    expected=publication['repository']+'@'+publication['index_digest']
    assert {dc['profiles'][n['role']]['image'] for n in dc['nodes'].values()}=={expected}
    assert publication['entrypoint_sha256']=='57c192078ba6b8be0f43aee3f08bef43581d17d877a6b0ef145cfd4c87dcb8e5'


def controller_wiring(rows):
    """Check declared dependencies; actual Controller API acceptance is separate."""
    def one(module, name=None):
        found=[r[module] for r in rows if module in r and (name is None or r[module].get('name')==name)]
        assert len(found)==1,(module,name)
        return found[0]
    project=one('awx.awx.project')
    credential=one('awx.awx.credential',project.get('credential'))
    assert credential['credential_type']=='Source Control'
    ee=one('awx.awx.execution_environment')
    registry=one('awx.awx.credential',ee.get('credential'))
    assert registry['credential_type']=='Container Registry'
    job=one('awx.awx.job_template')
    assert job.get('execution_environment')==ee['name']
    assert job.get('project')==project['name']
    parent=one('awx.awx.group','fabric')
    assert set(parent['children'])=={'spines','leafs'}
    connection=parent['variables']
    assert connection['ansible_connection']=='ansible.netcommon.network_cli'
    assert connection['ansible_network_os']=='frr.frr.frr'
    assert connection['ansible_network_cli_ssh_type']=='libssh'
    assert connection['ansible_host_key_checking'] is True and connection['ansible_libssh_host_key_checking'] is True
    role_task=next(r for r in rows if r.get('awx.awx.group',{}).get('name')=='{{ item.group }}')
    assert {(x['group'],x['role']) for x in role_task['loop']}=={('spines','spine'),('leafs','leaf')}


@given(st.sampled_from(['missing_ee','job_without_ee','project_without_scm','missing_role_group','missing_connection_vars']))
def test_controller_declared_dependencies(defect):
    rows=tasks();controller_wiring(rows)
    if defect=='missing_ee':rows=[r for r in rows if 'awx.awx.execution_environment' not in r]
    elif defect=='job_without_ee':next(r['awx.awx.job_template'] for r in rows if 'awx.awx.job_template' in r).pop('execution_environment')
    elif defect=='project_without_scm':next(r['awx.awx.project'] for r in rows if 'awx.awx.project' in r).pop('credential')
    elif defect=='missing_role_group':next(r for r in rows if r.get('awx.awx.group',{}).get('name')=='{{ item.group }}')['loop'].pop()
    else:next(r['awx.awx.group'] for r in rows if r.get('awx.awx.group',{}).get('name')=='fabric').pop('variables')
    with pytest.raises((AssertionError,KeyError,StopIteration)):controller_wiring(rows)


def test_installed_collection_argument_specs():
    run=subprocess.run([sys.executable,'tools/check_module_args.py'],cwd=ROOT,text=True,capture_output=True)
    assert run.returncode==0,run.stdout+run.stderr
    report=json.loads(run.stdout)
    assert report['status']=='PASS'


@given(st.integers(1,2048),st.sampled_from(['k','m','g']),st.booleans())
def test_simulator_resource_units_and_quota(number,unit,binary):
    from fractions import Fraction
    file=ROOT.parents[2]/'tests/datacenter_state.py'
    spec=importlib.util.spec_from_file_location('ex457_root_state',file)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    power={'k':1,'m':2,'g':3}[unit]
    suffix=unit+('i' if binary else '')+'b'
    expected=number*(1024 if binary else 1000)**power
    assert module.parse_memory(str(number)+suffix.upper())==expected
    assert module.parse_memory('256Mb')==256000000
    assert module.parse_memory('256MiB')==268435456
    assert module.cpu_limit({'NanoCpus':0,'CpuQuota':number*25000,'CpuPeriod':100000})==Fraction(number,4)
    for bad in [{},{'CpuQuota':-1,'CpuPeriod':100000},{'CpuQuota':50000,'CpuPeriod':0}]:
        with pytest.raises(ValueError):module.cpu_limit(bad)


@given(st.sampled_from(['name','label','subnet','attached','driver']))
def test_network_cleanup_refuses_unowned_or_active_network(defect):
    file=ROOT.parents[2]/'ci/cleanup_network.py'
    spec=importlib.util.spec_from_file_location('ex457_network_cleanup',file)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    network={'Id':'synthetic-id','Name':'gpu-dc-mgmt','Driver':'bridge','Labels':{'containerlab':''},'Containers':{},'IPAM':{'Config':[{'Subnet':'172.30.0.0/24'}]}}
    assert module.validate(network,'gpu-dc-mgmt','172.30.0.0/24')=='synthetic-id'
    if defect=='name':network['Name']='other-lab'
    elif defect=='label':network['Labels']={}
    elif defect=='subnet':network['IPAM']['Config'][0]['Subnet']='192.0.2.0/24'
    elif defect=='attached':network['Containers']={'active':{'Name':'other-lab-node'}}
    else:network['Driver']='overlay'
    with pytest.raises(ValueError):module.validate(network,'gpu-dc-mgmt','172.30.0.0/24')


def test_libssh_dependency_patch_is_exact_and_idempotent(tmp_path):
    from patch_libssh import patch, ORIGINAL, OPTION, AFTER, BEFORE
    from ansible.plugins.loader import connection_loader, init_plugin_loader
    init_plugin_loader()
    data=Path(connection_loader.find_plugin("ansible.netcommon.libssh")).read_text()
    original=data.replace(OPTION, '', 1).replace(AFTER, BEFORE, 1)
    assert hashlib.sha256(original.encode()).hexdigest()==ORIGINAL
    target=tmp_path/'ansible_collections/ansible/netcommon/plugins/connection/libssh.py'
    target.parent.mkdir(parents=True); target.write_text(original)
    patch(tmp_path); once=target.read_bytes(); patch(tmp_path)
    assert target.read_bytes()==once
    target.write_text(target.read_text()+'\n# unexpected modification\n')
    with pytest.raises(RuntimeError): patch(tmp_path)


def test_libssh_real_host_trust_accepts_pinned_key_and_rejects_wrong_key(tmp_path):
    import shutil
    import socket
    import time
    if os.geteuid()!=0 or not shutil.which('sshd'):
        pytest.skip('loopback SSH regression needs root and openssh-server')
    capabilities=int(next(line.split()[1] for line in Path('/proc/self/status').read_text().splitlines() if line.startswith('CapEff:')),16)
    if not capabilities & (1 << 18):
        if os.environ.get('EX457_DAGGER_RUNNER'):
            pytest.fail('mandatory live SSH regression requires CAP_SYS_CHROOT in the CI container')
        pytest.skip('openssh privilege separation requires CAP_SYS_CHROOT')
    Path('/run/sshd').mkdir(exist_ok=True)
    for name in ['server', 'client', 'wrong']:
        subprocess.run(['ssh-keygen','-q','-t','ed25519','-N','','-f',str(tmp_path/name)],check=True)
    with socket.socket() as s:
        s.bind(('127.0.0.1',0)); port=s.getsockname()[1]
    config=tmp_path/'sshd_config'
    config.write_text(f'''ListenAddress 127.0.0.1
Port {port}
HostKey {tmp_path/'server'}
PidFile {tmp_path/'sshd.pid'}
AuthorizedKeysFile {tmp_path/'client.pub'}
StrictModes no
PasswordAuthentication no
KbdInteractiveAuthentication no
PermitRootLogin prohibit-password
UsePAM no
''')
    log=(tmp_path/'sshd.log').open('w')
    server=subprocess.Popen([shutil.which('sshd'),'-D','-e','-f',str(config)],stdout=log,stderr=log)
    try:
        for _ in range(100):
            if server.poll() is not None: pytest.fail((tmp_path/'sshd.log').read_text())
            try:
                with socket.create_connection(('127.0.0.1',port),timeout=.1): break
            except OSError: time.sleep(.02)
        trust=tmp_path/'known_hosts'
        command=['ansible','all','-i','127.0.0.1,','-c','ansible.netcommon.libssh','-u','root',
                 '--private-key',str(tmp_path/'client'),'-m','ansible.builtin.raw','-a','printf trusted',
                 '-e',json.dumps({'ansible_port':port,'ansible_libssh_known_hosts':str(trust),
                                 'ansible_host_key_checking':True,'ansible_libssh_host_key_auto_add':False})]
        for key,success in [('server',True),('wrong',False)]:
            trust.write_text(f'[127.0.0.1]:{port} '+(tmp_path/(key+'.pub')).read_text())
            run=subprocess.run(command,capture_output=True,text=True,timeout=30)
            assert (run.returncode==0)==success,run.stdout+run.stderr
            if success: assert 'trusted' in run.stdout
            else: assert 'Host key' in run.stdout+run.stderr
    finally:
        server.terminate(); server.wait(timeout=5); log.close()
