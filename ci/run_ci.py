"""Run both real labs in order; export fresh results even when a required gate fails."""
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import time
ROOT=Path(__file__).resolve().parents[1]
PACK=ROOT/'cheatsheets/ex457/v6'
ARTIFACTS=ROOT/'artifacts'
FRR='quay.io/frrouting/frr@sha256:da3009abda75bd7cace6b2a9cbe28887cb2044ab1451103031340162704539fc'
REQUIRED=['historical-counterexamples','property-regressions','release-contracts','docker-ready','legacy-deploy','legacy-pyats','legacy-destroy','v6-prepare','v6-image','v6-deploy','v6-pyats','v6-destroy']


def main():
    ARTIFACTS.mkdir(exist_ok=True)
    logs=ARTIFACTS/'logs';logs.mkdir(exist_ok=True)
    rows=[];daemon=None
    report={'schema':1,'run_id':os.environ.get('CI_RUN_ID','local'), 'started_at':datetime.now(timezone.utc).isoformat(),
            'status':'RUNNING','required':REQUIRED,'checks':rows,'external_gates':{'aap_controller':'NOT_RUN','entitled_ee':'NOT_RUN','cachyos_host':'NOT_RUN','rhel_vm_reboot':'NOT_RUN'}}
    target=ARTIFACTS/'results.json'
    def save():target.write_text(json.dumps(report,indent=2)+'\n')
    def run(name,argv,cwd=PACK,required=True,extra_env=None):
        print('+ '+name,flush=True)
        with (logs/(name+'.log')).open('w') as log:
            result=subprocess.run(argv,cwd=cwd,env={**os.environ,**(extra_env or {})},stdout=log,stderr=subprocess.STDOUT,text=True,timeout=1800)
        rows.append({'name':name,'status':'PASS' if result.returncode==0 else 'FAIL','returncode':result.returncode,'log':'logs/'+name+'.log'})
        save()
        if required and result.returncode:
            print((logs/(name+'.log')).read_text(errors='replace')[-12000:],flush=True)
            raise RuntimeError('gate failed: '+name)
        return result.returncode
    save()
    try:
        run('historical-counterexamples',[sys.executable,'tools/check_previous_release.py'])
        run('property-regressions',[sys.executable,'-m','pytest','-q','--junitxml='+str(ARTIFACTS/'pytest.xml')])
        run('release-contracts',[sys.executable,'tools/validate.py','--full','--manifest'])
        with (logs/'dockerd.log').open('w') as stream:
            daemon=subprocess.Popen(['dockerd','--host=unix:///var/run/docker.sock','--storage-driver=vfs'],stdout=stream,stderr=subprocess.STDOUT)
        for attempt in range(90):
            if subprocess.run(['docker','info'],capture_output=True).returncode==0:break
            if daemon.poll() is not None:raise RuntimeError('nested Docker daemon stopped')
            time.sleep(1)
        else:raise RuntimeError('nested Docker readiness timeout')
        run('docker-ready',['docker','version'])
        run('legacy-deploy',['ansible-playbook','-i','inventory/localhost.ini','playbooks/deploy.yml'],ROOT)
        run('legacy-pyats',[sys.executable,'tests/datacenter_state.py'],ROOT)
        run('legacy-destroy',['ansible-playbook','-i','inventory/localhost.ini','playbooks/destroy.yml'],ROOT)
        key=PACK/'.state/client';key.parent.mkdir(mode=0o700,exist_ok=True)
        run('client-key',['ssh-keygen','-q','-t','ed25519','-N','','-f',str(key)])
        os.environ['ANSIBLE_PRIVATE_KEY_FILE']=str(key)
        run('v6-prepare',[sys.executable,'tools/labctl.py','prepare','--public-key',str(key)+'.pub'])
        run('v6-image',[sys.executable,'tools/labctl.py','build-image','--base',FRR])
        run('v6-deploy',[sys.executable,'tools/labctl.py','deploy'])
        run('v6-pyats',[sys.executable,'tests/pyats_live.py'])
    except BaseException as error:
        report['error_type']=type(error).__name__
        report['error']=str(error)[:2000]
    finally:
        if daemon is not None:
            try:
                run('docker-diagnostics',['docker','ps','-a'],required=False)
                if (PACK/'.state/datacenter.clab.yml').exists():
                    run('v6-destroy',[sys.executable,'tools/labctl.py','destroy'],required=False)
                # Handles a partially deployed legacy lab, including failures before normal teardown.
                if not any(r['name']=='legacy-destroy' and r['status']=='PASS' for r in rows):
                    run('legacy-cleanup',['ansible-playbook','-i','inventory/localhost.ini','playbooks/destroy.yml'],ROOT,required=False)
            except Exception as error:report['cleanup_error']=str(error)[:1000]
            if daemon.poll() is None:
                daemon.send_signal(signal.SIGTERM)
                try:daemon.wait(timeout=15)
                except subprocess.TimeoutExpired:daemon.kill();daemon.wait()
        passed={r['name'] for r in rows if r['status']=='PASS'}
        report['status']='PASS' if set(REQUIRED)<=passed and all(r['status']=='PASS' for r in rows) and 'error' not in report else 'FAIL'
        report['finished_at']=datetime.now(timezone.utc).isoformat()
        save()
        # Deliberate allowlist: never export .state wholesale (keys/backups/tokens).
        for source in (PACK/'.state/live-logs').glob('*.log'):
            shutil.copy2(source,logs/('live-'+source.name))
        for source in (PACK/'.state/runtime').glob('*.json'):
            shutil.copy2(source,ARTIFACTS/source.name)
        for name in ['image-inspect.json','base-image.txt']:
            source=PACK/'.state'/name
            if source.exists():shutil.copy2(source,ARTIFACTS/name)
        if report['status']=='PASS':
            run('package',[sys.executable,'tools/package.py','--output',str(ARTIFACTS/'ex457_containerlab_cheatsheet_v6.zip')])
        print(json.dumps(report,indent=2),flush=True)
    # The Dagger ci() function reads results.json and rejects FAIL; returning here
    # preserves diagnostics for export even on a failed test run.
    return 0


if __name__=='__main__':raise SystemExit(main())
