#!/usr/bin/env python3
"""Offline release checks. JSON status labels describe only checks actually run."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import yaml
from jinja2 import Environment, FileSystemLoader, StrictUndefined
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'filter_plugins'))
from fabric import model, intent, config_ok
from render_evidence import outputs


def check_structure():
    errors=[]
    sources=json.loads((ROOT/'evidence/sources.json').read_text())
    bp=json.loads((ROOT/'evidence/blueprint.json').read_text())
    claims=json.loads((ROOT/'evidence/claims.json').read_text())
    closure=json.loads((ROOT/'evidence/audit-closure.json').read_text())
    expected={'DEV':3,'NAV':2,'GIT':3,'VAR':3,'TASK':2,'CTRL':5,'ROLE':2,'NET':3}
    ids={f'{p}-{i}' for p,n in expected.items() for i in range(1,n+1)}
    if {o['id'] for o in bp['objectives']}!=ids or len(bp['objectives'])!=23:
        errors.append('public blueprint IDs missing/duplicated')
    if len({o['family'] for o in bp['objectives']})!=8:
        errors.append('blueprint family count')
    def anchor_exists(target):
        file,_,anchor=target.partition('#')
        path=ROOT/file
        if not path.is_file():
            return False
        if not anchor:
            return True
        headings=re.findall(r'^#{1,6}\s+(.+?)\s*$',path.read_text(),re.M)
        slugs={re.sub(r'[^\w\- ]','',h.lower()).replace(' ','-') for h in headings}
        return anchor in slugs
    for row in bp['objectives']:
        if not row.get('exercise') or not row.get('acceptance') or not anchor_exists(row['section']):
            errors.append('objective target/acceptance: '+row['id'])
        if not row['sources'] or any(i not in sources for i in row['sources']):
            errors.append('objective sources: '+row['id'])
    for c in claims:
        if not anchor_exists(c['section']) or any(i not in sources for i in c['sources']):
            errors.append('claim target/source: '+c['id'])
    if len(closure)!=48 or {c['id'] for c in closure}!={f'V4-{i:03}' for i in range(1,49)}:
        errors.append('audit finding coverage')
    for row in closure:
        if not (ROOT/row['file']).is_file():
            errors.append('closure file missing: '+row['id'])
        if row['status']=='CLOSED_RUNTIME':
            errors.append('runtime closure without release transcript: '+row['id'])
    for file,text in outputs().items():
        if (ROOT/file).read_text()!=text:
            errors.append('generated evidence stale: '+file)
    for path in ROOT.rglob('*.md'):
        if any(p in path.parts for p in ('.state','collections','.venv','context')):
            continue
        for link in re.findall(r'(?<!!)\[[^\]]*\]\(([^)]+)\)',path.read_text()):
            if re.match(r'^[a-z]+:',link) or link.startswith('#'):
                continue
            file,_,anchor=link.partition('#')
            absolute=(path.parent/file).resolve()
            if not absolute.is_file():
                errors.append(f'broken link {path.relative_to(ROOT)} -> {link}')
            elif anchor and absolute.is_relative_to(ROOT) and not anchor_exists(str(absolute.relative_to(ROOT))+'#'+anchor):
                errors.append(f'broken anchor {link}')
    # Verify Controller's template paths name real supplied playbooks.
    cac=yaml.safe_load((ROOT/'controller/configure.yml').read_text())
    for task in cac[0]['tasks']:
        if 'awx.awx.job_template' in task:
            for item in task['loop']:
                if not (ROOT/'playbooks'/f"{item['file']}.yml").is_file():
                    errors.append('missing job-template playbook '+item['file'])
    return errors


def main():
    p=argparse.ArgumentParser();p.add_argument('--full',action='store_true');p.add_argument('--manifest',action='store_true');args=p.parse_args()
    state=ROOT/'.state';state.mkdir(exist_ok=True)
    env=dict(os.environ,ANSIBLE_LOCAL_TEMP=str(state/'ansible-tmp'))
    rows=[]
    def result(name,errors,detail=None):
        rows.append(dict(check=name,status='FAIL' if errors else 'PASS',errors=errors,detail=detail))
    try:
        result('structure_links_claims_and_blueprint',check_structure(),{'objectives':23,'families':8,'audit_findings':48})
    except Exception as error:
        result('structure_links_claims_and_blueprint',[str(error)])
    try:
        dc=yaml.safe_load((ROOT/'model-upstream.yml').read_text())['dc'];model(dc)
        tpl=Environment(loader=FileSystemLoader(ROOT/'templates'),undefined=StrictUndefined)
        for name in dc['nodes']:
            desired=intent(dc,name)
            config_ok(tpl.get_template('fabric.conf.j2').render(fabric_intent=desired),desired)
        topology=yaml.safe_load(tpl.get_template('datacenter.clab.yml.j2').render(dc=dc,pack_root=str(ROOT)))
        assert set(topology['topology']['nodes'])==set(dc['nodes'])
        assert len(topology['topology']['links'])==len(dc['links'])
        data=(ROOT/'model-upstream.yml').read_bytes()
        actual=hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest()
        expected=json.loads((ROOT/'evidence/input-provenance.json').read_text())['model_git_blob']
        assert actual==expected, 'pinned Git blob mismatch'
        result('model_template_and_git_blob',[],{'git_blob':actual})
    except Exception as error:
        result('model_template_and_git_blob',[str(error)])
    for file in [ROOT/'lab/frr/ex457-start',ROOT/'lab/frr/ex457-load']:
        r=subprocess.run(['sh','-n',str(file)],text=True,capture_output=True)
        result('shell_parse:'+file.name,[] if r.returncode==0 else [r.stderr])
    def command(name,argv,cwd=ROOT):
        if shutil.which(argv[0]) is None:
            rows.append(dict(check=name,status='SKIPPED',reason='required executable unavailable: '+argv[0]));return
        r=subprocess.run(argv,cwd=cwd,env=env,text=True,capture_output=True)
        log=state/(name.replace(':','-')+'.log');log.write_text(r.stdout+r.stderr)
        result(name,[] if r.returncode==0 else [f'exit {r.returncode}; see {log}'],{'command':argv,'log':str(log.relative_to(ROOT))})
    command('predicate_unit_tests',[sys.executable,'-m','unittest','discover','-s','tests','-v'])
    if args.full:
        for file in sorted(list((ROOT/'playbooks').glob('*.yml'))+list((ROOT/'controller').glob('*.yml'))+[ROOT/'ee_smoke.yml']):
            if file.name in ('site.example.yml','site.yml','secrets.yml'):
                continue
            command('ansible_syntax:'+file.stem,['ansible-playbook','--syntax-check',str(file)])
        command('actual_module_arguments',[sys.executable,'tools/check_module_args.py'])
        command('ansible_synthetic_dataflow',[sys.executable,'tools/check_dataflow.py'])
        command('navigator_host_inventory',['ansible-navigator','inventory','-i','inventory/network.yml','--list','--mode','stdout','--ee','false'])
        command('navigator_settings',['ansible-navigator','settings','--mode','stdout'])
        command('builder_definition',['ansible-builder','create','-f','execution-environment.yml','--context',str(state/'ee-context')],cwd=ROOT/'ee')
    if args.manifest:
        errors=[]
        for line in (ROOT/'evidence/manifest.sha256').read_text().splitlines():
            digest,path=line.split('  ',1)
            if not (ROOT/path).is_file() or hashlib.sha256((ROOT/path).read_bytes()).hexdigest()!=digest:
                errors.append(path)
        result('unsigned_manifest_integrity',errors)
    for name in ['docker_image_ssh','live_frr_negative_cases','kernel_and_ping','restart_and_restore','ee_build_push_pull','controller_jobs_cac_and_durable_backup','rhel_vm_reboot']:
        rows.append(dict(check=name,status='NOT_RUN',reason='requires external lab/runtime; see chapter 20'))
    failed=any(r['status']=='FAIL' for r in rows)
    skipped=any(r['status']=='SKIPPED' for r in rows)
    report={'overall':'FAILED_AVAILABLE_CHECKS' if failed else ('PARTIAL_TOOLS_MISSING_RUNTIME_GATED' if skipped else 'STATIC_AND_FIXTURE_VALIDATED_RUNTIME_GATED'),
            'mode':'full' if args.full else 'offline-basic','checks':rows,
            'limits':'No machine claim of source semantics, vendor support, live runtime success or unseen LMS blueprint coverage.'}
    print(json.dumps(report,indent=2))
    return 1 if failed else 0


if __name__=='__main__':
    sys.exit(main())
