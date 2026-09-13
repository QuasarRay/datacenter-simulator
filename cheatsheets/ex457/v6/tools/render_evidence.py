#!/usr/bin/env python3
"""Generate the human coverage/source matrices from their checked registries."""
import json
from pathlib import Path
import sys
ROOT=Path(__file__).resolve().parents[1]


def outputs():
    sources=json.loads((ROOT/'evidence/sources.json').read_text())
    blueprint=json.loads((ROOT/'evidence/blueprint.json').read_text())
    claims=json.loads((ROOT/'evidence/claims.json').read_text())
    def cell(s):
        return s.replace('|','\\|').replace('\n',' ')
    def links(ids):
        return ', '.join(f'[{i}]({sources[i]["url"]})' for i in ids)
    rows=['# 00 — Public EX457 coverage matrix','','Scope: '+blueprint['scope']+'.',
          '',f"**{len(blueprint['objectives'])}/{blueprint['leaf_count']} public leaf objectives mapped across {len(set(o['family'] for o in blueprint['objectives']))} families.** This is content and exercise coverage. Live mastery/acceptance remains to be demonstrated.",
          '',f"[Official public exam page]({sources['EXAM']['url']}). Objective labels below are brief paraphrases, in source order. Lifecycle persistence is separately exercised in chapters 10/20; multi-vendor preparation is in chapter 13.",
          '','| ID / family | Objective | Instruction | Exercise | Acceptance evidence | Official sources |','|---|---|---|---|---|---|']
    for o in blueprint['objectives']:
        rows.append('| '+' | '.join([o['id']+' / '+o['family'],o['objective'],f"[Study section]({o['section']})",cell(o['exercise']),cell(o['acceptance']),links(o['sources'])])+' |')
    matrix=['# Content to official sources','','Generated from claims.json and sources.json. The validator compares this exact table to those registries; it does not infer semantic truth from a `reviewed` flag. Source review notes identify scope. Official upstream/community documentation does not imply Red Hat product support.',
            '','| Claim | Exact section | Claim summary | Source and locator |','|---|---|---|---|']
    for c in claims:
        refs='<br>'.join(f'[{i}]({sources[i]["url"]}) — {cell(sources[i]["locator"])}' for i in c['sources'])
        matrix.append(f"| {c['id']} | [Section](../{c['section']}) | {cell(c['claim'])} | {refs} |")
    index=['# Source review record','','Retrieval date: 2026-09-13. Short notes/excerpts below were recorded during source review. They are not full-page snapshots. Source semantics and runtime behavior are different forms of evidence. Product links that redirected to unrelated landing pages were not used as substantive claim support.', '']
    for i,s in sources.items():
        index += [f'## {i}', '',f"[{s['title']}]({s['url']})",'',f"Authority: {s['authority']}. Version/scope: {s['version']}.", '',f"Locator: {s['locator']}.", '', 'Review note: '+s.get('review_note',s.get('excerpt','')), '']
    closure=json.loads((ROOT/'evidence/audit-closure.json').read_text())
    table=['# v4 finding disposition in v6', '', 'Generated from audit-closure.json. See audit-history.md for all available audit generations and executable test mappings.', '', '| ID | Severity | Finding | Disposition | Change | Evidence |', '|---|---|---|---|---|---|']
    for row in closure:
        table.append('| '+' | '.join(cell(str(row[k])) for k in ('id','severity','finding','status','remediation','evidence'))+' |')
    history=json.loads((ROOT/'evidence/audit-history.json').read_text())
    hist=['# All available historical audit findings', '', history['scope'], '', '| ID | Finding | Disposition | Tests | Remaining acceptance |', '|---|---|---|---|---|']
    for row in history['findings']:
        hist.append('| '+' | '.join([row['id'],cell(row['title']),row['status'],cell(', '.join(row['tests'])),cell(row['acceptance'])])+' |')
    return {'evidence/audit-history.md':'\n'.join(hist)+'\n','evidence/audit-closure.md':'\n'.join(table)+'\n','00_blueprint_map.md':'\n'.join(rows)+'\n','evidence/content-to-sources.md':'\n'.join(matrix)+'\n','evidence/source-review.md':'\n'.join(index)+'\n'}


if __name__=='__main__':
    check='--check' in sys.argv
    errors=[]
    for path,text in outputs().items():
        if check:
            if not (ROOT/path).is_file() or (ROOT/path).read_text()!=text:
                errors.append(path)
        else:
            (ROOT/path).write_text(text)
    if check and errors:
        raise SystemExit('Stale generated evidence: '+', '.join(errors))
    print('Evidence tables synchronized' if not errors else errors)
