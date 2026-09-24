"""Read real native NetBox, compile its graph, realize it in Incus and send packets."""
import json
import os
from pathlib import Path
import subprocess
import sys
import urllib.error
import urllib.request

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT))
from integrations.incus.fabric import Fabric
from integrations.incus.transport import Incus

def qualify(fingerprint, token_file):
    output=ROOT/'artifacts/netbox-qualification';output.mkdir(parents=True,exist_ok=False)
    os.environ['NETBOX_TOKEN']=Path(token_file).read_text().strip()
    config=json.loads((ROOT/'integrations/netbox/native.example.json').read_text())
    config.update(image_fingerprint=fingerprint,project='ncp-netbox-ci')
    config['netbox'].update(api_url='http://127.0.0.1:8000/api/',allow_http=True)
    config['netbox']['resources']['memory']=512
    binary=ROOT/'simulator/target/debug/datacenter-simulator'
    fabric=None;tests=[];cleanup=None
    try:
        # A read-only token must not mutate even this disposable database.
        request=urllib.request.Request(config['netbox']['api_url']+'dcim/sites/',method='POST',
            headers={'Authorization':'Bearer '+os.environ['NETBOX_TOKEN'],'Content-Type':'application/json'},
            data=b'{"name":"forbidden","slug":"forbidden"}')
        try:urllib.request.urlopen(request,timeout=10)
        except urllib.error.HTTPError as error:
            if error.code!=403:raise
        else:raise RuntimeError('read-only NetBox token allowed mutation')
        tests.append('read-only-token-rejects-write')
        fabric=Fabric.compile(config,binary=binary,destination=output/'fabric')
        fabric.deploy();first=fabric.reconcile();second=fabric.reconcile()
        recaps=[line for line in second['stdout'].splitlines() if 'changed=' in line and 'unreachable=' in line]
        if len(recaps)!=5 or any('changed=0' not in line.replace(' ','') for line in recaps):
            raise RuntimeError('generated guest reconciliation is not idempotent')
        manifest=json.loads((fabric.directory/'manifest.json').read_text())
        lookup={spec['labels']['netbox.device_name']:node for node,spec in manifest['content']['nodes'].items()}
        guests=json.loads((fabric.directory/'provisioning-vars.json').read_text())
        source,target=lookup['compute-a'],lookup['compute-b']
        destination=next(address for endpoint,address in guests['ncp_addresses'].items() if endpoint.split(':')[0]==target)
        api=Incus(fabric.directory/'state/incus.json')
        code,out,err=api.execute(source,['ping','-c','3','-W','2',destination],timeout=15)
        (output/'payload.txt').write_bytes(out+err)
        if code:raise RuntimeError('NetBox-derived routed native payload failed')
        tests.append('netbox-to-incus-deepops-idempotent-native-payload')
        journal=json.loads((fabric.directory/'state/incus.json').read_text())
        source_spec=next(node for node in journal['plan']['nodes'] if node['name']==source)
        peer=next(device['host_name'] for device in source_spec['spec']['devices'].values() if device['type']=='nic')
        cable=next(i for i,c in enumerate(journal['plan']['cables']) if peer in c['peers'])
        def link(state):subprocess.run([str(binary),'incus-link',str(fabric.directory/'state'),str(cable),state],check=True,capture_output=True,timeout=30)
        link('down')
        if api.execute(source,['ping','-c','1','-W','1',destination],timeout=5)[0]==0:
            raise RuntimeError('payload escaped the selected cable')
        link('up')
        if api.execute(source,['ping','-c','3','-W','2',destination],timeout=15)[0]:
            raise RuntimeError('payload did not recover')
        tests.append('generated-topology-partition-and-recovery')
    finally:
        if fabric is not None and (fabric.directory/'state/incus.json').exists():
            try:fabric.destroy();cleanup=True
            except Exception:cleanup=False;raise
        (output/'qualification.json').write_text(json.dumps({'netbox_revision':'00791344e68213bde942218283dce03cc3941c30',
            'tests':tests,'required_tests':3,'complete':len(tests)==3 and cleanup is True,
            'owned_cleanup':cleanup,'tier':'emulated','hardware_or_proprietary_validation':False},indent=2)+'\n')

if __name__=='__main__':
    if os.geteuid()!=0 or len(sys.argv)!=3:raise SystemExit('requires the disposable CI host, image fingerprint and private token file')
    qualify(*sys.argv[1:])
