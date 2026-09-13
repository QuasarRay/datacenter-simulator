"""Adversarial checks against the CI report gate with explicitly synthetic reports."""
import copy
import importlib.util
import json
from pathlib import Path
import sys
import xml.etree.ElementTree as ET
import zipfile
import hashlib
import pytest
from hypothesis import given, strategies as st
ROOT=Path(__file__).resolve().parents[1]
REPO=ROOT.parents[2]
sys.path.insert(0,str(REPO/'ci'))
from check_results import check
from run_ci import REQUIRED


def test_ci_gate_rejects_missing_failed_stale_and_unexecuted_evidence(tmp_path):
    names=REQUIRED+['package']
    report={'schema':1,'run_id':'synthetic-ci-gate-test','source_manifest_sha256':hashlib.sha256((ROOT/'evidence/manifest.sha256').read_bytes()).hexdigest(),'status':'PASS','required':REQUIRED,'checks':[{'name':n,'status':'PASS','returncode':0} for n in names]}
    # Construct JUnit identities only for this gate unit test; not runtime evidence.
    history=json.loads((ROOT/'evidence/audit-history.json').read_text())
    targets={t for r in history['findings'] for t in r['tests'] if not t.startswith('tests/pyats_live.py')}
    suite=ET.Element('testsuite')
    for target in sorted(targets):
        file,symbol=target.split('::');module=file.removesuffix('.py').replace('/','.')
        ET.SubElement(suite,'testcase',classname=module,name=symbol)
    ET.ElementTree(suite).write(tmp_path/'pytest.xml')
    with zipfile.ZipFile(tmp_path/'ex457_containerlab_cheatsheet_v6.zip','w') as archive:
        prefix='ex457-v6-project/cheatsheets/ex457/v6/'
        archive.write(ROOT/'evidence/manifest.sha256',prefix+'evidence/manifest.sha256')
        for line in (ROOT/'evidence/manifest.sha256').read_text().splitlines():
            _,name=line.split('  ',1);archive.write(ROOT/name,prefix+name)
    def write(value):(tmp_path/'results.json').write_text(json.dumps(value))
    write(report);assert check(tmp_path,report['run_id'],ROOT)
    for field,value in [('status','FAIL'),('run_id','old-run'),('source_manifest_sha256','0'*64),('checks',report['checks'][:-1])]:
        bad=copy.deepcopy(report);bad[field]=value;write(bad)
        with pytest.raises(AssertionError):check(tmp_path,report['run_id'],ROOT)
    write(report)
    ET.SubElement(suite[0],'skipped');ET.ElementTree(suite).write(tmp_path/'pytest.xml')
    with pytest.raises(AssertionError):check(tmp_path,report['run_id'],ROOT)
    suite[0].remove(suite[0][0]);suite.remove(suite[0]);ET.ElementTree(suite).write(tmp_path/'pytest.xml')
    with pytest.raises(AssertionError,match='not executed'):check(tmp_path,report['run_id'],ROOT)
