"""Audit 6311 counterexamples. Fixtures are not live datacenter evidence."""
import copy
import io
from pathlib import Path
import time
import pytest
import yaml
from jinja2 import Environment, FileSystemLoader
import fabric
import backup_store as bs

ROOT=Path(__file__).resolve().parents[1]

@pytest.mark.parametrize('statement', ['shutdown', 'ip access-group BLOCK in', 'ip ospf area 0', 'no ip forwarding'])
def test_unmodeled_interface_state_is_not_certified(statement):
    dc=yaml.safe_load((ROOT/'model-upstream.yml').read_text())['dc']
    desired=fabric.intent(dc,'spine1')
    text=Environment(loader=FileSystemLoader(ROOT/'templates')).get_template('fabric.conf.j2').render(fabric_intent=desired)
    with pytest.raises(ValueError): fabric.config_ok(text.replace('interface eth1','interface eth1\n '+statement),desired)
    with pytest.raises(ValueError): fabric.config_ok(text+'\nip route 203.0.113.0/24 Null0\n',desired)

@pytest.mark.parametrize('prefix', ['224.0.0', '127.0.0', '0.0.0', '169.254.0', '240.0.0'])
def test_link_endpoints_require_usable_unicast(prefix):
    dc=yaml.safe_load((ROOT/'model-upstream.yml').read_text())['dc']
    for index in range(2): dc['links'][0]['endpoints'][index]['address']=f'{prefix}.{index}/31'
    with pytest.raises(ValueError): fabric.model(dc)


def test_read_bounds_and_total_deadline_apply_during_transfer():
    source=io.BytesIO(b'x'*100)
    with pytest.raises(ValueError,match='byte limit'): bs.bounded_read(source,10)
    assert source.tell()==11
    assert bs.bounded_read(io.BytesIO(b'x'*10),10)==b'x'*10
    before=time.monotonic()
    with pytest.raises(TimeoutError):
        with bs.deadline(.05): time.sleep(2)
    assert time.monotonic()-before<1


def test_http_rejects_oversized_header_and_oversized_body(monkeypatch):
    class Response(io.BytesIO):
        status=200
        headers={}
    class Opener:
        response=None
        def open(self,*args,**kwargs): return self.response
    opener=Opener()
    monkeypatch.setattr(bs.urllib.request,'build_opener',lambda *args:opener)
    for advertised in ('100', None):
        opener.response=Response(b'x'*100)
        opener.response.headers={} if advertised is None else {'Content-Length':advertised}
        with pytest.raises(ValueError): bs.request('https://fixture.invalid','run','host.cfg','fixture',limit=10)


def test_refused_redeploy_preserves_committed_topology():
    import shutil
    import subprocess
    import sys
    repo=ROOT.parents[2]
    subprocess.run([sys.executable,str(repo/'tests/deploy_refusal.py'),str(repo),shutil.which('ansible-playbook')],check=True,timeout=45)
