"""Compile the complete source-facet obligations. No evidence is fabricated."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def build(blueprint):
    objects = {o['id']: o for o in blueprint['objectives']}
    stations = []
    for unit in blueprint['units']:
        facets = [{'id': oid+'/'+facet, 'domain': {'AIO':0,'AIN':1,'AII':2}[objects[oid]['exam']],
                   'tier':4} for oid in unit['objectives'] for facet in objects[oid]['facets']]
        for exercise in unit['exercises']:
            stations.append({'id':len(stations),'name':exercise.lower(),'facets':facets})
    return stations

if __name__ == '__main__':
    import sys
    sys.path.insert(0, str(ROOT))
    from verification.pipeline import render_repository
    render_repository(ROOT)
