import copy
import json
from pathlib import Path
import pytest
from hypothesis import given, strategies as st
from evidence_contract import manifest_ok, manifest_text, claims_ok, history_ok
from render_evidence import outputs
ROOT=Path(__file__).resolve().parents[1]


def test_nonempty_manifest_exact_coverage(tmp_path):
    (tmp_path/'README.md').write_text('content')
    text=manifest_text(tmp_path);assert manifest_ok(tmp_path,text)
    for mutation in ['',text+text,text.replace('README.md','../README.md'),text.replace('README.md','absent.md')]:
        with pytest.raises(ValueError):manifest_ok(tmp_path,mutation)
    (tmp_path/'added.py').write_text('raise SystemExit(1)')
    with pytest.raises(ValueError):manifest_ok(tmp_path,text)


@given(st.integers(0,41))
def test_nonempty_complete_claims(index):
    claims=json.loads((ROOT/'evidence/claims.json').read_text());sources=json.loads((ROOT/'evidence/sources.json').read_text())
    assert claims_ok(claims,sources)
    for mutation in [[],claims[:index]+claims[index+1:],claims+[claims[index]]]:
        with pytest.raises(ValueError):claims_ok(mutation,sources)
    claims[index]['sources']=[]
    with pytest.raises(ValueError):claims_ok(claims,sources)


def test_human_evidence_matches_registry():
    for file,text in outputs().items():assert (ROOT/file).read_text()==text,file


def test_all_available_audit_ids_and_test_targets():
    history=json.loads((ROOT/'evidence/audit-history.json').read_text());assert history_ok(history)
    for row in history['findings']:
        for target in row['tests']:
            file,_,symbol=target.partition('::')
            assert (ROOT/file).is_file(),target
            assert symbol and (('def '+symbol+'(') in (ROOT/file).read_text() or ('class '+symbol+'(') in (ROOT/file).read_text() or (symbol+' = BackupLifecycle.TestCase') in (ROOT/file).read_text()),target
    history['findings'].pop()
    with pytest.raises(ValueError):history_ok(history)


def test_public_blueprint_exact_objectives():
    from validate import check_structure
    assert check_structure()==[]
