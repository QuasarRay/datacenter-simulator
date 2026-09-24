"""Check learning links, exact facets, examples and deterministic generation."""
import json
from pathlib import Path
import sys

ROOT=Path(__file__).resolve().parents[1]

def validate(base):
    blueprint=json.loads((base/'blueprint.json').read_text())
    curriculum=json.loads((base/'curriculum.json').read_text())
    objects={o['id']:o for o in blueprint['objectives']}
    all_courses={f'C{i:02}' for i in range(1,21)}
    taught=set();combined=set();seen=set();errors=[]
    def check(value,message):
        if not value:errors.append(message)
    check(len(curriculum)==20,'expected twenty learning units')
    for level,entry in enumerate(curriculum,1):
        u=blueprint['units'][level-1];c=entry['course'];p=entry['project']
        check(c['level']==p['level']==level,'nonmonotonic course/project level')
        check(c['id']==u['course'] and p['id']==u['project'],'blueprint learning identity mismatch')
        check(all_courses<=set(p['prerequisites']),f"{p['id']} must assume every course")
        taught.update(c['skills'])
        check(set(p['skills'])<=taught,f"{p['id']} requires an untaught skill")
        combined.update(p['skills'])
        expected={oid+'/'+f for oid in u['objectives'] for f in objects[oid]['facets']}
        check(len(entry['exercises'])==2,'each unit needs both incidents')
        for j,e in enumerate(entry['exercises']):
            check(e['level']==2*level-1+j,'nonmonotonic exercise level')
            check(e['id']==u['exercises'][j],'exercise identity mismatch')
            check(set(e['skills'])<=taught&combined,f"{e['id']} lacks a teaching/project antecedent")
            check(set(e['facets'])==expected,f"{e['id']} omits a facet")
            check((base/e['prompt']).is_file(),f"missing {e['prompt']}")
            seen.update(e['facets'])
        for path in [c['document'],c['example'],p['document']]:
            check((base/path).is_file(),f'missing {path}')
        if (base/c['document']).exists():
            text=(base/c['document']).read_text()
            check(all(f'`{fid}`' in text for fid in expected),f"{c['id']} missing facet row")
            check('```mermaid' in text and '```python' in text,f"{c['id']} missing diagram/code")
    check({o['id']+'/'+f for o in objects.values() for f in o['facets']}<=seen,'uncovered source facet')
    return errors

if __name__=='__main__':
    errors=validate(ROOT/'education/ncp-metablueprint')
    if errors:raise SystemExit('\n'.join(errors))
    print('PASS: 20 course/project pairs, 40 incidents, complete facet/skill links; semantic and live qualification remain separate')
