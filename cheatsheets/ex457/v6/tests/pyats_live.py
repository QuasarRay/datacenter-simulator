"""Destructive acceptance tests ONLY for the isolated ex457-v6 Incus lab.

A missing Incus daemon or an unprepared lab is an error, never a skip/pass.
Raw outputs stay in private .state; CI exports an allowlist of sanitized logs.
"""
import hashlib
import ipaddress
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import uuid
import yaml
from pyats import aetest
ROOT=Path(__file__).resolve().parents[1]
sys.path[:0]=[str(ROOT/'tools'),str(ROOT/'filter_plugins')]
from fabric import intent, config_ok, peers_ok, bgp_ok, rib_ok, kernel_ok, ping_ok
from backup_store import verify as verify_backup
from state import recorded_run, secure_dir
DC=yaml.safe_load((ROOT/'model-upstream.yml').read_text())['dc']
STATE=secure_dir(ROOT/'.state')
LOGS=secure_dir(STATE/'live-logs')
OFFICIAL_SOURCES=json.loads((ROOT/'evidence/official-source-quotes.json').read_text())['sources']
RUNTIME_DOCS={row['id']:row for row in json.loads((ROOT/'evidence/runtime-doc-contracts.json').read_text())['contracts']}
COUNTER=0


def doc_contract(contract_id,observer):
    row=RUNTIME_DOCS[contract_id]
    assert row['observer']==observer,(contract_id,row['observer'],observer)
    assert row['source_id'] in OFFICIAL_SOURCES,(contract_id,row['source_id'])
    return row


def cmd(argv, expected=0, env=None):
    global COUNTER
    COUNTER+=1
    run=subprocess.run(list(map(str,argv)),cwd=ROOT,env={**os.environ,**(env or {})},text=True,capture_output=True,timeout=420)
    output=run.stdout+run.stderr
    (LOGS/f'{COUNTER:03}-{Path(str(argv[0])).name}.log').write_text(output)
    if expected==0 and run.returncode: raise AssertionError(f'{argv[0]} exit {run.returncode}: {output[-6000:]}')
    if expected=='failure' and run.returncode==0: raise AssertionError('Injected defect was accepted: '+str(argv))
    return run


def guest(node,*args,expected=0):
    import importlib.util
    transport_path = Path(__file__).resolve().parents[4]/'integrations/incus/transport.py'
    spec = importlib.util.spec_from_file_location('incus_transport', transport_path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    client = module.Incus(os.environ['NCP_JOURNAL'])
    code, out, err = client.execute(node, list(args))
    assert code == expected, (code, err)
    return subprocess.CompletedProcess(args, code, out.decode(), err.decode())


def cli(node,*commands):
    args=['vtysh']
    for command in commands:args+=['-c',command]
    return guest(node,*args).stdout


def play(name,*args,expected=0,env=None):
    return cmd(['ansible-playbook','-i','inventory/network.yml',f'playbooks/{name}.yml',*args],expected,env)


class LabSetup(aetest.CommonSetup):
    @aetest.subsection
    def require_exact_testbed(self):
        journal=json.loads(Path(os.environ['NCP_JOURNAL']).read_text())
        running=[n['name'] for n in journal['plan']['nodes']]
        assert set(running)==set(DC['nodes'])
        assert journal['phase']=='active'
        for node in running: guest(node, 'true')


class Transport(aetest.Testcase):
    @aetest.test
    def ssh_policy_and_real_libssh(self):
        doc_contract('R001','Transport.ssh_policy_and_real_libssh')
        for node in DC['nodes']:
            policy=guest(node,'/usr/sbin/sshd','-T').stdout
            for directive in ['passwordauthentication no','kbdinteractiveauthentication no','pubkeyauthentication yes','permitrootlogin no']:
                assert directive in policy
            account=guest(node,'awk','-F:', '$1 == "ansible" { if ($2 ~ /^[!*]/) print "LOCKED"; else print "UNLOCKED" }','/etc/shadow').stdout.strip()
            assert account == 'UNLOCKED', 'missing or locked public-key account'
            assert '10.7.1' in cli(node,'show version')
        play('show_version')
        wrong=STATE/'wrong-hostkey';cmd(['ssh-keygen','-q','-t','ed25519','-N','','-f',wrong])
        trust=STATE/'wrong-known-hosts';key=wrong.with_suffix('.pub').read_text().strip()
        trust.write_text('\n'.join(n['mgmt_ip']+' '+key for n in DC['nodes'].values())+'\n')
        rejected=play('show_version','--limit','spine1',expected='failure',env={'EX457_KNOWN_HOSTS':str(trust)})
        assert re.search(r'host.?key|known.host|authenticity|fingerprint',rejected.stdout+rejected.stderr,re.I),'failure must identify host trust'
        play('show_version','--limit','spine1')


class Convergence(aetest.Testcase):
    @aetest.test
    def configure_and_idempotence(self):
        play('configure_fabric')
        second=play('configure_fabric').stdout
        recaps=re.findall(r'^\S+\s+: ok=\d+\s+changed=(\d+).*failed=(\d+)',second,re.M)
        assert len(recaps)==len(DC['nodes']) and all(changed=='0' and failed=='0' for changed,failed in recaps),'second run must converge without changes'

    @aetest.test
    def documented_frr_config_facts(self):
        doc_contract('R002','Convergence.documented_frr_config_facts')
        output=play('facts_filters').stdout
        assert 'ansible_net_config' in output,'config gather_subset must expose ansible_net_config on the live pinned lab'
        assert 'ansible_net_version' in output,'live facts must include the device software version'

    @aetest.test
    def independent_addresses_and_routes(self):
        doc_contract('R003','Convergence.independent_addresses_and_routes')
        with recorded_run(ROOT,'live-independent',STATE) as record:
            for name,node in DC['nodes'].items():
                want=intent(DC,name)
                raw=json.loads(guest(name,'ip','-j','-4','address','show').stdout)
                observed={r['ifname']:[f"{a['local']}/{a['prefixlen']}" for a in r.get('addr_info',[]) if a['family']=='inet' and a['local']!='127.0.0.1'] for r in raw if r['ifname']!='eth0'}
                observed={k:sorted(v) for k,v in observed.items() if v}
                expected={'lo':[node['loopback']]}
                for link in DC['links']:
                    for end in link['endpoints']:
                        if end['node']==name:expected[end['interface']]=[end['address']]
                assert observed==expected,(name,observed,expected)
                config_ok(cli(name,'show running-config'),want)
                peers_ok(cli(name,'show bgp ipv4 unicast summary json'),want)
                for remote,other in DC['nodes'].items():
                    if remote==name:continue
                    prefix=other['loopback']
                    bgp_ok(cli(name,'show bgp ipv4 unicast '+prefix+' json'),prefix,want)
                    rib_ok(cli(name,'show ip route '+prefix+' json'),prefix,want)
                    kernel_ok(guest(name,'ip','-j','-4','route','show','exact',prefix).stdout,prefix,want)
                    ping_ok(guest(name,'ping','-n','-c','3','-W','2','-I',node['loopback'].split('/')[0],prefix.split('/')[0]).stdout)
                    record['results'].append({'node':name,'remote':remote,'checks':['address','BGP','Zebra','kernel','sourced-ping']})
            assert len(record['results'])==12


class NegativeCases(aetest.Testcase):
    @aetest.test
    def extra_peer_and_address_fail_then_reconcile(self):
        cli('spine1','configure terminal','router bgp 65001','neighbor 192.0.2.250 remote-as 65250','end')
        try:
            failed=play('verify_fabric','--limit','spine1',expected='failure')
            assert 'Wait for exact unfiltered peer set' in failed.stdout
        finally:play('configure_fabric')
        cli('spine1','configure terminal','interface lo','ip address 192.0.2.249/32','end')
        try:
            failed=play('verify_fabric','--limit','spine1',expected='failure')
            assert 'managed config mismatch: interfaces' in failed.stdout
        finally:play('configure_fabric')

    @aetest.test
    def total_ping_loss_is_a_failed_fresh_run(self):
        cmd([sys.executable,'tools/labctl.py','dataplane'])
        success=json.loads((STATE/'dataplane.json').read_text())
        rule=['OUTPUT','-p','icmp','--icmp-type','echo-request','-j','DROP']
        guest('spine1','iptables','-I',*rule)
        try:
            cmd([sys.executable,'tools/labctl.py','dataplane'],expected='failure')
            failed=json.loads((STATE/'dataplane.json').read_text())
            assert failed['status']=='FAIL' and failed['run_id']!=success['run_id']
        finally:guest('spine1','iptables','-D',*rule)
        cmd([sys.executable,'tools/labctl.py','dataplane'])


class Persistence(aetest.Testcase):
    @aetest.test
    def private_backup_restore_save_restart(self):
        doc_contract('R004','Persistence.private_backup_restore_save_restart')
        run_id='ci_'+uuid.uuid4().hex
        play('backup_fabric','-e','backup_run_id='+run_id)
        directory=STATE/'backups'/run_id
        index=verify_backup(directory)
        before={n:hashlib.sha256((directory/(n+'.cfg')).read_bytes()).hexdigest() for n in DC['nodes']}
        failed=play('backup_fabric','-e','backup_run_id='+run_id,expected='failure')
        assert 'FileExistsError' in failed.stdout
        assert before=={n:hashlib.sha256((directory/(n+'.cfg')).read_bytes()).hexdigest() for n in DC['nodes']}
        cli('spine1','configure terminal','interface lo','ip address 192.0.2.248/32','end')
        play('restore_fabric','-e','restore_run_id='+run_id)
        play('persist_fabric')
        saved={n:hashlib.sha256(guest(n,'cat','/etc/frr/frr.conf').stdout.encode()).hexdigest() for n in DC['nodes']}
        cmd([sys.executable,'tools/labctl.py','restart'])
        play('verify_fabric')
        cmd([sys.executable,'tools/labctl.py','dataplane'])
        cmd([sys.executable,'tools/labctl.py','saved'])
        assert saved=={n:hashlib.sha256(guest(n,'cat','/etc/frr/frr.conf').stdout.encode()).hexdigest() for n in DC['nodes']}
        assert set(index['nodes'])==set(DC['nodes'])


if __name__=='__main__':
    result=aetest.main()
    raise SystemExit(0 if str(result).lower()=='passed' else 1)
