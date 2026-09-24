#!/usr/bin/env python3
"""Validate coverage identities, facets and tri-domain assessment obligations."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def validate(data):
    errors = []
    def require(condition, message):
        if not condition:
            errors.append(message)
    expected = {'AIO-W': [13, 5, 7, 6], 'AIO-G': [5, 3, 6, 5],
                'AIN': [3, 8, 4, 2, 5, 2], 'AII': [11, 2, 7, 14, 5]}
    wanted = {f'{s}-{i}.{j}' for s, counts in expected.items()
              for i, n in enumerate(counts, 1) for j in range(1, n + 1)}
    objectives = data['objectives']
    ids = [o['id'] for o in objectives]
    require(len(ids) == len(set(ids)), 'duplicate objective')
    require(set(ids) == wanted, f'objective mismatch: missing={wanted-set(ids)}, extra={set(ids)-wanted}')
    by_id = {o['id']: o for o in objectives}
    units = {u['id']: u for u in data['units']}
    require(len(units) == len(data['units']) == 20, 'expected 20 distinct units')
    for source, counts in expected.items():
        require(sum(data['sources'][source]['weights']) == 100, f'{source}: weights')
    for o in objectives:
        require(o['source'] in expected, f"{o['id']}: unknown source")
        require(o['facets'] and len(o['facets']) == len(set(o['facets'])), f"{o['id']}: empty/duplicate facets")
        u = units.get(o['unit'])
        require(u is not None, f"{o['id']}: missing unit")
        if u:
            require(o['id'] in u['objectives'], f"{o['id']}: missing unit mapping")
            require(o['taught_by'] == u['course'] and o['combined_by'] == u['project']
                    and o['assessed_by'] == u['exercises'], f"{o['id']}: broken learning chain")
    for level, u in enumerate(data['units'], 1):
        require(u['level'] == level, f"{u['id']}: nonincremental level")
        require(all(i in by_id for i in u['objectives']), f"{u['id']}: unknown objective")
        exams = {by_id[i]['exam'] for i in u['objectives'] if i in by_id}
        require(exams == {'AIO', 'AIN', 'AII'}, f"{u['id']}: missing source domain")
        require(u['prerequisites'] == ([] if level == 1 else [f'U{level-1:02}']), f"{u['id']}: prerequisite cycle or gap")
    return errors

if __name__ == '__main__':
    data = json.loads((ROOT/'education/ncp-metablueprint/blueprint.json').read_text())
    errors = validate(data)
    if errors:
        raise SystemExit('\n'.join(errors))
    print(f"PASS: {len(data['objectives'])} source objectives, "
          f"{sum(len(o['facets']) for o in data['objectives'])} facets, "
          f"{len(data['units'])} tri-domain units (specification coverage only)")
