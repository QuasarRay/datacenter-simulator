import copy
import json
from pathlib import Path
import sys
from hypothesis import given, strategies as st

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'tools'))
from documentation_contract import audit_ok, official_sources_ok, runtime_contracts_ok, REQUIRED_SOURCES


def load():
    claims=json.loads((ROOT/'evidence/claims.json').read_text())
    audit=json.loads((ROOT/'evidence/official-doc-audit.json').read_text())
    sources=json.loads((ROOT/'evidence/official-source-quotes.json').read_text())
    runtime=json.loads((ROOT/'evidence/runtime-doc-contracts.json').read_text())
    return claims,audit,sources,runtime


def test_reviewed_official_documentation_contract():
    claims,audit,sources,runtime=load()
    assert audit_ok(claims,audit,sources)
    assert runtime_contracts_ok(runtime,audit,sources,(ROOT/'tests/pyats_live.py').read_text())


@given(st.sampled_from(['quotes','locator','version','url']))
def test_official_source_provenance_mutations_fail_closed(field):
    _,_,sources,_=load()
    broken=copy.deepcopy(sources)
    source=broken['sources']['RH-AAP-CAC']
    source[field]=[] if field=='quotes' else ('https://example.com/not-official' if field=='url' else '')
    try:
        official_sources_ok(broken)
    except ValueError:
        return
    raise AssertionError('mutated official-source provenance was accepted')


@given(st.sampled_from(sorted(REQUIRED_SOURCES)))
def test_reviewed_claim_source_mapping_cannot_be_silently_redefined(claim_id):
    claims,audit,sources,_=load()
    broken=copy.deepcopy(audit)
    row=next(r for r in broken['claims'] if r['id']==claim_id)
    row['official_sources']=row['official_sources'][1:]
    try:
        audit_ok(claims,broken,sources)
    except ValueError:
        return
    raise AssertionError('reviewed claim/source mapping mutation was accepted')


@given(st.sampled_from(['C023','C024','C025','C026','C031','C037','C042']))
def test_aap_product_claim_cannot_be_downgraded_to_ansible_only_source(claim_id):
    claims,audit,sources,_=load()
    broken=copy.deepcopy(audit)
    row=next(r for r in broken['claims'] if r['id']==claim_id)
    row['official_sources']=['ANS-PLAY']
    try:
        audit_ok(claims,broken,sources)
    except ValueError:
        return
    raise AssertionError('AAP product claim lost Red Hat product authority')


@given(st.sampled_from(['R001','R002','R003','R004']))
def test_runtime_documentation_mapping_mutations_fail_closed(contract_id):
    _,audit,sources,runtime=load()
    broken=copy.deepcopy(runtime)
    row=next(r for r in broken['contracts'] if r['id']==contract_id)
    row['source_id']='ANS-PLAY'
    try:
        runtime_contracts_ok(broken,audit,sources,(ROOT/'tests/pyats_live.py').read_text())
    except ValueError:
        return
    raise AssertionError('runtime documentation mapping mutation was accepted')
