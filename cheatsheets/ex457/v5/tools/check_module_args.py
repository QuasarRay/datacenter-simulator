#!/usr/bin/env python3
"""Validate rendered collection module inputs using their actual argument_spec.

No module connects to a device/API: construction is intercepted before I/O.
This does not validate Controller's server-side nested credential schemas.
"""
import copy
import importlib
import json
import os
from pathlib import Path
import sys
import yaml
from jinja2.nativetypes import NativeEnvironment
from jinja2 import StrictUndefined
from ansible.module_utils.common.arg_spec import ArgumentSpecValidator
from ansible.utils.collection_loader._collection_finder import _AnsibleCollectionFinder
from ansible.plugins.filter.core import FilterModule

ROOT = Path(__file__).resolve().parents[1]
_AnsibleCollectionFinder(paths=os.environ.get('ANSIBLE_COLLECTIONS_PATH', str(ROOT/'collections')).split(os.pathsep))._install()
sys.path.insert(0, str(ROOT / 'filter_plugins'))
import fabric


class Captured(Exception):
    pass


def actual_spec(fqcn):
    ns, coll, name = fqcn.split('.')
    mod = importlib.import_module(f'ansible_collections.{ns}.{coll}.plugins.modules.{name}')
    result = {}
    def capture(*args, **kwargs):
        result.update(kwargs)
        if args:
            result['argument_spec'] = args[0]
        raise Captured()
    ctor = 'ControllerAPIModule' if ns == 'awx' else 'AnsibleModule'
    original = getattr(mod, ctor)
    setattr(mod, ctor, capture)
    try:
        try:
            mod.main()
        except Captured:
            pass
    finally:
        setattr(mod, ctor, original)
    assert 'argument_spec' in result, fqcn
    if ns == 'awx':
        helper = importlib.import_module('ansible_collections.awx.awx.plugins.module_utils.controller_api')
        result['argument_spec'] = {**helper.ControllerModule.AUTH_ARGSPEC, **result['argument_spec']}
    return {k:v for k,v in result.items() if k in ('argument_spec','mutually_exclusive','required_together','required_one_of','required_if','required_by')}


def tasks(value):
    if isinstance(value, list):
        for v in value:
            yield from tasks(v)
    elif isinstance(value, dict):
        if any(k.startswith(('awx.awx.', 'ansible.netcommon.', 'frr.frr.')) for k in value):
            yield value
        for key in ('tasks', 'pre_tasks', 'post_tasks', 'block', 'rescue', 'always'):
            yield from tasks(value.get(key, []))


def main():
    dc = yaml.safe_load((ROOT/'model-upstream.yml').read_text())['dc']
    want = fabric.intent(dc, 'spine1')
    ctx = dict(dc=dc, fabric_intent=want, inventory_hostname='spine1', ansible_host='172.30.0.11',
               organization_name='EX457 Lab', scm_url='https://github.com/QuasarRay/datacenter-simulator.git',
               scm_branch='main', project_prefix='cheatsheets/ex457/v5', ee_image='registry.example.net/ee@sha256:'+'a'*64,
               backup_url='https://backup.example.net/ex457', backup_directory='/tmp/ex457/backup', backup_run_id='123',
               fabric_removals='interface lo\n no ip address 192.0.2.1/32\nexit', fabric_candidate='router bgp 65001',
               machine_private_key='FIXTURE ONLY', scm_username='fixture', scm_token='fixture', registry_username='fixture',
               registry_password='fixture', known_hosts_text='FIXTURE ONLY', backup_token='fixture',
               remote_node=want['remote_nodes'][0], diagnostic_command='show version')
    env = NativeEnvironment(undefined=StrictUndefined)
    env.filters.update(FilterModule().filters())
    env.globals['lookup'] = lambda *a, **kw: 'fixture'
    def render(value, scope):
        if isinstance(value, str):
            return env.from_string(value).render(**scope)
        if isinstance(value, list):
            return [render(x,scope) for x in value]
        if isinstance(value, dict):
            return {k:render(v,scope) for k,v in value.items()}
        return value
    rows = []
    for file in sorted(list((ROOT/'playbooks').rglob('*.yml')) + list((ROOT/'roles').rglob('*.yml')) + list((ROOT/'controller').glob('*.yml'))):
        for task in tasks(yaml.safe_load(file.read_text())):
            fqcn = next(k for k in task if k.startswith(('awx.awx.','ansible.netcommon.','frr.frr.')))
            for item in render(task.get('loop',[None]), ctx):
                scope = dict(ctx)
                if item is not None:
                    scope[task.get('loop_control',{}).get('loop_var','item')] = item
                try:
                    spec = actual_spec(fqcn)
                    args = render(task[fqcn], scope)
                    errors = ArgumentSpecValidator(**spec).validate(args).error_messages
                    # Prove unsupported parameters are actually rejected.
                    mutation = ArgumentSpecValidator(**spec).validate({**args, 'ex457_invalid_argument': True}).error_messages
                    if not mutation:
                        errors.append('unknown-argument mutation was not rejected')
                    if fqcn == 'frr.frr.frr_facts':
                        valid = importlib.import_module('ansible_collections.frr.frr.plugins.modules.frr_facts').VALID_SUBSETS
                        for subset in args.get('gather_subset',[]):
                            if subset.lstrip('!') not in valid | {'all'}:
                                errors.append('invalid fact subset '+subset)
                except Exception as error:
                    errors = [str(error)]
                rows.append({'file':str(file.relative_to(ROOT)), 'task':task.get('name'), 'module':fqcn, 'status':'FAIL' if errors else 'PASS', 'errors':errors})
    result = {'check':'actual_collection_module_argument_specs', 'status':'PASS' if rows and all(r['status']=='PASS' for r in rows) else 'FAIL', 'cases':rows,
              'limit':'Schema validation only; Controller API payload acceptance and FRR transport remain runtime gates.'}
    print(json.dumps(result, indent=2))
    return 0 if result['status']=='PASS' else 1


if __name__ == '__main__':
    sys.exit(main())
