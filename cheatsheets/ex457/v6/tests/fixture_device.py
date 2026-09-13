"""Synthetic command responses for task dataflow testing; never a live capture."""
import json
from pathlib import Path
import sys
import yaml
from jinja2 import Environment, FileSystemLoader
ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT/'filter_plugins'))
from fabric import intent
dc = yaml.safe_load((ROOT/'model-upstream.yml').read_text())['dc']
node, command, scenario = sys.argv[1:]
want = intent(dc, node)
peer = next(iter(want['peers']))
if command == 'show running-config':
    env=Environment(loader=FileSystemLoader(ROOT/'templates'))
    text=env.get_template('fabric.conf.j2').render(fabric_intent=want)
    if scenario == 'wrong_interface':
        text=text.replace('interface eth1','interface eth9')
    print(text)
elif command == 'show bgp ipv4 unicast summary json':
    peers={ip:{'state':'Established','remoteAs':asn} for ip,asn in want['peers'].items()}
    if scenario == 'extra_idle_peer':
        peers['192.0.2.99']={'state':'Idle','remoteAs':65200}
    print(json.dumps({'peers':peers}))
elif command.startswith('show bgp ipv4 unicast '):
    prefix=command.split()[-2]
    print(json.dumps({'prefix':prefix,'paths':[{'valid':True,'bestpath':{'overall':True},'nexthops':[{'ip':peer}]}]}))
elif command.startswith('show ip route '):
    prefix=command.split()[-2]
    print(json.dumps({prefix:[{'protocol':'bgp','selected':True,'installed':scenario!='uninstalled_route','nexthops':[{'ip':peer,'interfaceName':want['peer_interfaces'][peer],'active':True,'fib':True}]}]}))
else:
    raise ValueError('fixture command not modeled: '+command)
