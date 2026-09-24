"""Disposable-host qualification of actual Incus, patchbay and DeepOps behavior.

Run only on the isolated CI VM or an explicitly prepared empty qualification host.
All lab mutations use owned journals. The only extra device is the explicitly
created foreign-member test veth, removed by its creating test.
"""
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT))
from integrations.incus.lab import Lab
from integrations.incus.transport import Incus

def command(*args,success=True):
    result=subprocess.run(list(map(str,args)),capture_output=True,text=True,timeout=300)
    if success and result.returncode:
        raise RuntimeError(f'{args[0]} failed: {result.stderr[-4000:]}')
    return result

def guest(api,node,*args,success=True):
    code,out,err=api.execute(node,list(args),timeout=30)
    if success and code:raise RuntimeError(f'{node}: {err.decode(errors="replace")}')
    return code,out.decode(errors='replace')

def qualify(fingerprint):
    workspace=ROOT/'artifacts/incus-qualification';workspace.mkdir(parents=True,exist_ok=False)
    binary=ROOT/'simulator/target/debug/datacenter-simulator'
    topology=json.loads((ROOT/'integrations/incus/topology.json').read_text())
    for node in topology['content']['nodes'].values():node['os']='ubuntu-24.04'
    manifest=workspace/'topology.json';manifest.write_text(json.dumps(topology))
    states=[];results=[]
    try:
        for suffix in ('a','b'):
            config={'project':'ncp-ci-'+suffix,'socket':'/var/lib/incus/unix.socket',
                    'storage_pool':'default','image_fingerprint':fingerprint,
                    'manifest':str(manifest),'max_memory_mib':8192}
            path=workspace/f'config-{suffix}.json';path.write_text(json.dumps(config))
            state=workspace/f'lab-{suffix}'
            states.append(state)
            command(binary,'incus-up',path,state)
            interfaces={};names=['control','compute-a','compute-b','services']
            for i,name in enumerate(names,1):
                interfaces[name]=[{'name':'data0','content':
                    f'[Match]\nName=data0\n[Network]\nAddress=10.66.{i}.2/30\nLinkLocalAddressing=no\nIPv6AcceptRA=no\n[Route]\nDestination=10.66.0.0/16\nGateway=10.66.{i}.1\n'}]
            interfaces['leaf']=[{'name':f'swp{i}','content':
                f'[Match]\nName=swp{i}\n[Network]\nAddress=10.66.{i}.1/30\nLinkLocalAddressing=no\nIPv6AcceptRA=no\n'} for i in range(1,5)]
            report=Lab(state).apply(ROOT/'integrations/incus/provision.yml',{'ncp_network_files':interfaces})
            if report['returncode']!=0:raise RuntimeError('DeepOps reconciliation failed')
            api=Incus(state/'incus.json')
            guest(api,'leaf','sysctl','-w','net.ipv4.ip_forward=1')
            guest(api,'compute-a','ping','-c','3','-W','2','10.66.3.2')
            second=Lab(state).apply(ROOT/'integrations/incus/provision.yml',{'ncp_network_files':interfaces})
            # Ansible recap: all owned hosts must be unchanged on reconciliation.
            recaps=[line for line in second['stdout'].splitlines() if 'changed=' in line and 'unreachable=' in line]
            if len(recaps)!=5 or any('changed=0' not in line.replace(' ', '') for line in recaps):
                raise RuntimeError('native reconciliation is not idempotent')
            results.append({'test':'native-deepops-and-payload','project':config['project'],'passed':True})
        first=json.loads((states[0]/'incus.json').read_text());second=json.loads((states[1]/'incus.json').read_text())
        if {c['name'] for c in first['plan']['cables']} & {c['name'] for c in second['plan']['cables']}:
            raise RuntimeError('concurrent labs share cable identities')
        api_a=Incus(states[0]/'incus.json');api_b=Incus(states[1]/'incus.json')
        peers=first['plan']['nodes']
        compute=next(n for n in peers if n['name']=='compute-a')
        peer=compute['spec']['devices']['data0']['host_name']
        index=next(i for i,c in enumerate(first['plan']['cables']) if peer in c['peers'])
        command(binary,'incus-link',states[0],index,'down')
        if guest(api_a,'compute-a','ping','-c','1','-W','1','10.66.3.2',success=False)[0]==0:
            raise RuntimeError('payload crossed the disabled cable')
        guest(api_b,'compute-a','ping','-c','2','-W','2','10.66.3.2')
        command(binary,'incus-link',states[0],index,'up')
        guest(api_a,'compute-a','ping','-c','3','-W','2','10.66.3.2')
        results.append({'test':'partition-recovery-and-concurrent-isolation','passed':True})
        # No other lab or host resource may be silently adopted during cleanup.
        cable=first['cables'][0]['bridge']
        command('ip','link','add','ncp-test-extra','type','veth','peer','name','ncp-test-peer')
        try:
            command('ip','link','set','ncp-test-extra','master',cable)
            refused=command(binary,'incus-down',states[0],success=False)
            if refused.returncode==0:raise RuntimeError('cleanup accepted a foreign bridge member')
            api_a.checked('compute-a')
        finally:command('ip','link','delete','ncp-test-extra')
        results.append({'test':'foreign-member-cleanup-refusal','passed':True})
    finally:
        cleanup=[]
        for state in reversed(states):
            if not (state/'incus.json').exists():
                continue # preflight failed before any owned resources existed
            outcome=command(binary,'incus-down',state,success=False)
            cleanup.append({'state':str(state),'returncode':outcome.returncode,'stderr':outcome.stderr})
            if outcome.returncode==0:command(binary,'incus-down',state) # idempotent completed teardown
        report={'tier':'emulated','image_fingerprint':fingerprint,'tests':results,'cleanup':cleanup,
                'hardware_or_proprietary_validation':False}
        (workspace/'qualification.json').write_text(json.dumps(report,indent=2)+'\n')
        if any(x['returncode'] for x in cleanup):raise RuntimeError('owned cleanup incomplete; inspect qualification artifacts')
    print(json.dumps(report,indent=2))

if __name__=='__main__':
    if os.geteuid()!=0 or len(sys.argv)!=2:raise SystemExit('requires the disposable CI host and one image fingerprint')
    qualify(sys.argv[1])
