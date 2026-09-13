"""Fail-closed provenance rules for Red Hat/Ansible documentation mappings.

This module does not pretend that string validation proves semantic entailment.  The reviewed
mapping is explicit in evidence/official-doc-audit.json; these checks make provenance,
coverage, product authority and the reviewed mapping mechanically immutable.
"""
from copy import deepcopy
from pathlib import Path
from urllib.parse import urlparse

CLAIM_IDS={f'C{i:03d}' for i in range(1,43)}
ALLOWED_AUTHORITIES={'Ansible','Red Hat'}
ALLOWED_HOSTS={'docs.ansible.com','docs.redhat.com','www.redhat.com','redhat.com'}
CLASSIFICATIONS={'OFFICIAL_DOCS','MIXED','REPOSITORY_RUNTIME','OUTSIDE_ANSIBLE_RED_HAT_SCOPE'}
VERDICTS={'VERIFIED','VERIFIED_WITH_BOUNDARY','CORRECTED','OUT_OF_SCOPE'}

# This is the human-reviewed claim -> authoritative-source mapping.  Duplicating it here is
# deliberate: mutating the JSON registry alone must not silently redefine what "verified" means.
REQUIRED_SOURCES={
 'C003':{'ANS-NETCLI'},'C004':{'ANS-INV'},'C005':{'ANS-NAV'},'C006':{'RH-EX457'},
 'C007':{'ANS-PLAY'},'C008':{'ANS-REGISTER'},'C009':{'ANS-FRR-FACTS'},'C010':{'ANS-LOOP','ANS-BLOCK'},
 'C012':{'ANS-REUSE'},'C013':{'ANS-BUILDER','ANS-NAV'},'C014':{'ANS-CLI-CONFIG'},
 'C018':{'ANS-CLI-CONFIG'},'C020':{'ANS-CLI-BACKUP'},
 'C021':{'ANS-CLI-BACKUP','RH-AAP-CUSTOM'},
 'C023':{'RH-AAP-CREDS','RH-AAP-CUSTOM','RH-AAP-PROJECT','RH-AAP-JOB','RH-AAP-SCM'},
 'C024':{'RH-AAP-WORKFLOW'},'C025':{'RH-AAP-CAC'},
 'C026':{'RH-AAP-CUSTOM','RH-AAP-PROJECT','RH-AAP-WORKFLOW'},
 'C027':{'RH-EX457','ANS-NETCLI'},'C028':{'RH-EX457'},
 'C029':{'ANS-BLOCK','ANS-NETCLI','RH-AAP-WORKFLOW'},
 'C030':{'ANS-INV','ANS-NETCLI','RH-AAP-WORKFLOW'},
 'C031':{'RH-EX457','ANS-FRR-FACTS','RH-AAP-CAC','RH-RHEL-ETH'},
 'C034':{'RH-RHEL-ETH','RH-RHEL-ROUTE'},'C035':{'RH-RHEL-BRIDGE'},
 'C036':{'ANS-NETCLI'},'C037':{'RH-EX457'},'C042':{'RH-AAP-CAC'},
}
RED_HAT_PRODUCT_CLAIMS={'C006','C021','C023','C024','C025','C026','C027','C028','C031','C034','C035','C037','C042'}
RUNTIME_IDS={'R001','R002','R003','R004'}


def _fail(message):
    raise ValueError(message)


def official_sources_ok(data):
    if data.get('schema')!=1 or data.get('retrieved_on')!='2026-09-13':
        _fail('official source registry schema/retrieval date')
    sources=data.get('sources')
    if not isinstance(sources,dict) or not sources:
        _fail('official source registry empty')
    for source_id,source in sources.items():
        if source.get('authority') not in ALLOWED_AUTHORITIES:
            _fail('unapproved authority: '+source_id)
        url=source.get('url','')
        parsed=urlparse(url)
        if parsed.scheme!='https' or parsed.hostname not in ALLOWED_HOSTS:
            _fail('unapproved official URL: '+source_id)
        for field in ('version','locator'):
            if not str(source.get(field,'')).strip():
                _fail(f'{source_id} missing {field}')
        quotes=source.get('quotes')
        if not isinstance(quotes,list) or not quotes:
            _fail(source_id+' missing direct quote')
        for quote in quotes:
            if not isinstance(quote,str) or not quote.strip() or len(quote.split())>30:
                _fail(source_id+' quote must be short and nonempty')
    return True


def audit_ok(claims,audit,source_data):
    official_sources_ok(source_data)
    claim_ids={row.get('id') for row in claims}
    if claim_ids!=CLAIM_IDS or len(claims)!=42:
        _fail('claims.json coverage changed')
    rows=audit.get('claims') if audit.get('schema')==1 else None
    if not isinstance(rows,list) or len(rows)!=42:
        _fail('official audit must contain exactly 42 claims')
    by_id={row.get('id'):row for row in rows}
    if set(by_id)!=CLAIM_IDS or len(by_id)!=len(rows):
        _fail('official audit claim IDs missing or duplicated')
    sources=source_data['sources']
    for claim_id,row in by_id.items():
        if row.get('classification') not in CLASSIFICATIONS or row.get('verdict') not in VERDICTS:
            _fail('invalid classification/verdict: '+claim_id)
        mapped=row.get('official_sources')
        repo=row.get('repository_evidence')
        if not isinstance(mapped,list) or not isinstance(repo,list):
            _fail('bad evidence lists: '+claim_id)
        if any(source_id not in sources for source_id in mapped):
            _fail('unknown official source: '+claim_id)
        expected=REQUIRED_SOURCES.get(claim_id,set())
        if set(mapped)!=expected:
            _fail('reviewed official mapping changed: '+claim_id)
        if row['classification'] in {'REPOSITORY_RUNTIME','OUTSIDE_ANSIBLE_RED_HAT_SCOPE'} and not repo:
            _fail('non-vendor claim lacks repository/runtime evidence: '+claim_id)
        if claim_id in RED_HAT_PRODUCT_CLAIMS and mapped and not any(sources[s]['authority']=='Red Hat' for s in mapped):
            _fail('Red Hat product claim lost Red Hat authority: '+claim_id)
        if not str(row.get('note','')).strip() and row['verdict']!='VERIFIED':
            _fail('bounded claim needs review note: '+claim_id)
    if by_id['C025']['verdict']!='CORRECTED':
        _fail('AAP 2.6 CaC correction lost')
    return True


def runtime_contracts_ok(runtime,audit,source_data,pyats_text=None):
    official_sources_ok(source_data)
    rows=runtime.get('contracts') if runtime.get('schema')==1 else None
    if not isinstance(rows,list) or {r.get('id') for r in rows}!=RUNTIME_IDS or len(rows)!=4:
        _fail('runtime documentation contracts changed')
    audit_by={r['id']:r for r in audit['claims']}
    seen=set()
    for row in rows:
        claim=row.get('claim_id');source=row.get('source_id');observer=row.get('observer')
        if claim not in audit_by or source not in source_data['sources']:
            _fail('runtime contract references unknown claim/source')
        if source not in audit_by[claim]['official_sources']:
            _fail('runtime contract source is not audited for claim '+claim)
        if not observer or observer in seen:
            _fail('runtime observer missing/duplicate')
        seen.add(observer)
        if not row.get('documented_proposition') or not row.get('runtime_oracle'):
            _fail('runtime contract lacks proposition/oracle')
        if pyats_text is not None:
            method=observer.split('.')[-1]
            if f'def {method}(' not in pyats_text:
                _fail('runtime observer not implemented in pyATS: '+observer)
    return True
