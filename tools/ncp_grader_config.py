"""Build the learner-readable proxy configuration; no private evidence paths."""
import json
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]

def configuration(python,proxy,socket):
    if not all(Path(p).is_absolute() for p in (python,proxy,socket)):
        raise ValueError('explicit absolute installation paths required')
    bank=json.loads((ROOT/'education/assessment/bank.json').read_text())
    return {'protocol':'ncp-grade-v1','program':str(python),'args':[str(proxy),str(socket)],
            'timeout_secs':480,'exercises':{s['name']:{'id':s['id'],'tier':4,
             'required_facets':(1<<len(s['facets']))-1} for s in bank}}
