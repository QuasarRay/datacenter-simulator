import copy
import json
from pathlib import Path
import tempfile
import pytest
import yaml
from hypothesis import given, settings, strategies as st
from hypothesis.stateful import RuleBasedStateMachine, rule, invariant
from jinja2 import Environment, FileSystemLoader
import backup_store as bs
from fabric import intent
from state import recorded_run, secure_dir
ROOT = Path(__file__).resolve().parents[1]
DC = yaml.safe_load((ROOT/'model-upstream.yml').read_text())['dc']
ENV = Environment(loader=FileSystemLoader(ROOT/'templates'))


def capture(base, name):
    folder = bs.reserve(base, name)
    for node in DC['nodes']:
        (folder/(node+'.cfg')).write_text(ENV.get_template('fabric.conf.j2').render(fabric_intent=intent(DC,node)))
    bs.seal(folder)
    return folder


class BackupLifecycle(RuleBasedStateMachine):
    def __init__(self):
        super().__init__(); self.temp = tempfile.TemporaryDirectory(); self.base = Path(self.temp.name)/'.state/backups'; self.runs = {}

    @rule(run_id=st.integers(0, 5))
    def create_or_reuse(self, run_id):
        name = str(run_id)
        if name in self.runs:
            before = {p.name:p.read_bytes() for p in (self.base/name).iterdir()}
            with pytest.raises(FileExistsError): bs.reserve(self.base,name)
            assert before == {p.name:p.read_bytes() for p in (self.base/name).iterdir()}
        else:
            capture(self.base, name); self.runs[name] = True

    @rule(run_id=st.integers(0, 5), appended=st.binary(min_size=1,max_size=20))
    def corrupt(self, run_id, appended):
        name = str(run_id)
        if name in self.runs:
            with (self.base/name/'spine1.cfg').open('ab') as stream: stream.write(appended)
            self.runs[name] = False

    @invariant()
    def verify_never_accepts_corruption(self):
        for name, valid in self.runs.items():
            if valid: assert bs.verify(self.base/name)['run_id'] == name
            else:
                with pytest.raises(ValueError, match='checksum'): bs.verify(self.base/name)
            assert (self.base/name).stat().st_mode & 0o777 == 0o700
            for file in (self.base/name).iterdir(): assert file.stat().st_mode & 0o777 == 0o600

    def teardown(self): self.temp.cleanup()


TestBackupLifecycle = BackupLifecycle.TestCase
TestBackupLifecycle.settings = settings(max_examples=35, stateful_step_count=12, derandomize=True, deadline=None)


def test_state_permissions_repaired(tmp_path):
    path=tmp_path/'.state'; path.mkdir(mode=0o755); secure_dir(path)
    folder=capture(path/'backups','permissions')
    assert path.stat().st_mode & 0o777 == 0o700
    assert (path/'backups').stat().st_mode & 0o777 == 0o700
    assert all(p.stat().st_mode & 0o777 == 0o600 for p in folder.iterdir())


def test_backup_index_and_partial_capture_rejected(tmp_path):
    folder=capture(tmp_path/'backups','index')
    assert bs.verify(folder)['nodes'].keys() == DC['nodes'].keys()
    (folder/'manifest.json').unlink()
    with pytest.raises(ValueError): bs.verify(folder)


@given(st.sampled_from(['FAIL', 'KeyboardInterrupt']))
def test_fresh_runtime_failure_replaces_success(failure):
    with tempfile.TemporaryDirectory() as temp:
        state=Path(temp)
        with recorded_run(ROOT,'dataplane',state) as good: good['results'].append({'probe':'synthetic'})
        first=json.loads((state/'dataplane.json').read_text())
        with pytest.raises(BaseException):
            with recorded_run(ROOT,'dataplane',state):
                raise (KeyboardInterrupt() if failure == 'KeyboardInterrupt' else ValueError('injected'))
        last=json.loads((state/'dataplane.json').read_text())
        assert first['status']=='PASS' and last['status']=='FAIL'
        assert first['run_id'] != last['run_id'] and last['results']==[]
        assert len(list((state/'runtime').glob('*.json'))) == 2


def test_export_conditional_write_and_readback(tmp_path, monkeypatch):
    folder=capture(tmp_path/'backups','export'); objects={}; calls=[]
    def service(base, run, filename, token, method='GET', body=None, cafile=None):
        calls.append((method,filename))
        if method=='PUT':
            if filename in objects: raise FileExistsError('conditional write')
            objects[filename]=body
        return objects[filename]
    monkeypatch.setattr(bs,'request',service)
    bs.export(folder,'https://fixture.invalid','test-token')
    assert calls[-2:] == [('PUT','manifest.json'),('GET','manifest.json')]
    with pytest.raises(FileExistsError): bs.export(folder,'https://fixture.invalid','test-token')
    restored=tmp_path/'retrieval/backups'
    bs.download(restored,'export','https://fixture.invalid','test-token')
    for p in folder.iterdir(): assert p.read_bytes()==(restored/'export'/p.name).read_bytes()
    # This service is synthetic. The live HTTPS protocol test separately checks headers/TLS.
