"""Prove selected concrete audit counterexamples distinguish v5 from v6."""
import copy
import importlib.util
import json
from pathlib import Path
import sys
import yaml
from jinja2 import Environment,FileSystemLoader
ROOT=Path(__file__).resolve().parents[1]

def load(path,name):
    spec=importlib.util.spec_from_file_location(name,path);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module

def main():
    old=load(ROOT.parent/'v5/filter_plugins/fabric.py','old_fabric');new=load(ROOT/'filter_plugins/fabric.py','new_fabric')
    dc=yaml.safe_load((ROOT/'model-upstream.yml').read_text())['dc']
    text=Environment(loader=FileSystemLoader(ROOT/'templates')).get_template('fabric.conf.j2').render(fabric_intent=new.intent(dc,'spine1'))
    tests={}
    for name,mutate in [('V5-F007',lambda d:d['nodes']['spine2'].update(asn=d['nodes']['spine1']['asn'])),('V5-F008',lambda d:d['nodes']['spine1'].update(mgmt_ip='172.30.0.1')),('V5-F009',lambda d:d['nodes']['spine1'].update(loopback='0.0.0.0/32'))]:
        data=copy.deepcopy(dc);mutate(data);tests[name]=lambda f,data=data:f.model(data)
    tests['V5-F004']=lambda f:f.config_ok(text.replace('address-family ipv4 unicast','address-family ipv6 unicast'),f.intent(dc,'spine1'))
    prefix='10.255.1.1/32'
    mixed={prefix:[{'protocol':'bgp','selected':True,'installed':True,'nexthops':[{'ip':'10.0.0.1','interfaceName':'eth1','active':True,'fib':True},{'ip':'192.0.2.1','interfaceName':'eth1','active':True,'fib':True}]}]}
    tests['V5-F005']=lambda f:f.rib_ok(mixed,prefix,f.intent(dc,'spine1'))
    wrong=copy.deepcopy(mixed);wrong[prefix][0]['nexthops']=wrong[prefix][0]['nexthops'][:1];wrong[prefix][0]['nexthops'][0]['interfaceName']='eth0'
    tests['V5-F006']=lambda f:f.rib_ok(wrong,prefix,f.intent(dc,'spine1'))
    rows=[]
    for name,probe in tests.items():
        outcomes={}
        for version,module in [('v5',old),('v6',new)]:
            try:probe(module);outcomes[version]='ACCEPTED_INVALID_STATE'
            except ValueError:outcomes[version]='REJECTED_INVALID_STATE'
        rows.append({'finding':name,**outcomes,'status':'PASS' if outcomes=={'v5':'ACCEPTED_INVALID_STATE','v6':'REJECTED_INVALID_STATE'} else 'FAIL'})
    print(json.dumps({'check':'historical_counterexamples','scope':'six concrete predicate counterexamples; not all historical findings','results':rows},indent=2))
    return 0 if all(row['status']=='PASS' for row in rows) else 1
if __name__=='__main__':raise SystemExit(main())
