"""Source-preserving blueprint adaptations; a complete map is not live mastery."""
import json
import re

from verification.dsl import require
from verification.io import strict_json
from verification.materials import record, sequence, text, unique


TRACKS = (
    ('NCP-DCIT', 'DCIT troubleshooting adaptation', 2),
    ('NCP-WCA', 'WCA and CBT Nuggets packet-analysis adaptation', 3),
    ('CKA', 'GPU cluster administration adaptation', 3),
    ('CKNE', 'GPU Kubernetes networking adaptation', 3),
    ('CNPE', 'GPU platform engineering adaptation', 3),
    ('EX200', 'CachyOS GPU-host administration adaptation', 3),
    ('EX294', 'GPU-host automation adaptation', 3),
    ('EX342', 'GPU-host diagnosis and troubleshooting adaptation', 3),
    ('EX457', 'GPU fabric automation adaptation', 3),
    ('NCP-CCDE', 'CCDE AI infrastructure, on-premises/cloud and CCNP DC design adaptation', 3),
    ('VCDX-VCF', 'GPU infrastructure architecture and virtualization adaptation', 3),
)


def compile_dcit(bp, lock, rows, units, obligations):
    record(lock, ['schema', 'source', 'objectives'])
    require(type(lock['schema']) is int and lock['schema'] == 1, 'DCIT source schema')
    record(lock['source'], ['name', 'version', 'url', 'reviewed', 'basis', 'weights'])
    require(lock['source']['version'] == '1.2' and lock['source']['weights'] == [25, 25, 15, 15, 20], 'DCIT source edition')
    for field in ['name', 'url', 'reviewed', 'basis']: text(lock['source'][field])
    source = sequence(lock['objectives'], 52)
    require(len(source) == len(rows) == 52, 'retain every DCIT leaf, including repeated subjects with different IDs')
    require(sum(len(item['concepts']) for item in source) == 84, 'retain all 84 reviewed DCIT concepts')
    require(len(units) == 12 and len(set(units)) == 12, 'DCIT unit inventory')
    wanted = {}
    for item in source:
        record(item, ['locator', 'page', 'concepts'])
        require(re.fullmatch(r'[1-5]\.[1-6](?:\.[a-g])?', item['locator']) is not None, 'invalid DCIT locator')
        require(type(item['page']) is int and 1 <= item['page'] <= 3, 'source page')
        require(item['locator'] not in wanted, 'duplicate source locator')
        unique(item['concepts'], nonempty=True)
        require(all(re.fullmatch('[a-z0-9-]+', concept) for concept in item['concepts']), 'unsafe concept identity')
        wanted[item['locator']] = item
    require({row.locator for row in rows} == set(wanted) and len({row.locator for row in rows}) == len(rows), 'omitted, duplicate or unknown DCIT adaptation')
    core = {unit['id']: unit for unit in bp['units']}
    domains = {o['id']: o['exam'] for o in bp['objectives']}
    expanded = []
    for row in rows:
        require(type(row.unit) is int and 1 <= row.unit <= 12 and row.core in core, 'DCIT learning/core reference')
        for field in [row.investigation, row.evidence, row.boundary]: text(field)
        references = core[row.core]['objectives']
        observed = {domains[oid] for oid in references}
        mask = sum(1 << i for i, domain in enumerate(('AIO', 'AIN', 'AII')) if domain in observed)
        obligations.check('DCIT/domains/' + row.locator, 'report_complete', 7, mask, 0)
        expanded.append(dict(wanted[row.locator], unit=f'D{row.unit:02}', core_unit=row.core,
                             nvidia_objectives=references, investigation=row.investigation,
                             required_evidence=row.evidence, fidelity_boundary=row.boundary,
                             status='specified-not-qualified'))
    require({row['unit'] for row in expanded} == {f'D{i:02}' for i in range(1, 13)}, 'empty DCIT unit')
    expected = {item['locator'] + '/' + concept for item in source for concept in item['concepts']}
    actual = {item['locator'] + '/' + concept for item in expanded for concept in item['concepts']}
    obligations.sets('DCIT/source', 'coverage', expected, actual)
    result = {'schema': 1, 'name': 'NCP-DCIT', 'independent': True, 'source': lock['source'],
              'source_leaves': len(source), 'source_facets': len(expected), 'live_mastery_enabled': False,
              'units': [{'id': f'D{i:02}', 'level': i, 'title': title,
                         'course': f'DC{i:02}', 'project': f'DP{i:02}',
                         'exercises': [f'DE{i:02}a', f'DE{i:02}b'], 'status': 'planned'}
                        for i, title in enumerate(units, 1)], 'objectives': expanded}
    return result


def render_extensions(root, bp, obligations, loader):
    authored = loader(root, 'education/authoring/dcit.py', 'ncp_dcit_authoring')
    lock = strict_json((root / 'verification/dcit-source-obligations.json').read_text())
    catalog = compile_dcit(bp, lock, authored.ROWS, authored.UNITS, obligations)
    # Every added user requirement stays visible until an explicitly reviewed
    # source and an implemented teaching/assessment chain are supplied.
    scope = {'schema': 1, 'nvidia_core': {'priority': 1, 'source_objectives': 113,
                                      'source_facets': 203, 'live_mastery_enabled': False},
             'extensions': [{'id': identity, 'description': description, 'priority': priority,
                             'status': 'specified-not-qualified' if identity == 'NCP-DCIT' else 'source-review-pending',
                             'live_mastery_enabled': False} for identity, description, priority in TRACKS],
             'release_complete': False}
    out = ['# NCP-DCIT adaptation obligations', '',
           'This independent extension preserves the three NVIDIA domains as a conjunctive core. It is not an official certification.', '',
           f"Source: [Cisco DCIT {lock['source']['version']}]({lock['source']['url']}), reviewed {lock['source']['reviewed']}. "
           f"The registry retains {catalog['source_leaves']} numbered leaves and {catalog['source_facets']} inline concepts. "
           'A parent heading is covered by its children. Entries are project-authored investigations, not exam questions.', '',
           'The ledger specifies work; it does not claim that these courses, projects, collectors or hardware gates are complete. '
           'No extension entry can grant live mastery. NCP-DCIT reuses the core three-domain evidence rule; one or two domains grant no unit.', '',
           'The units below are the delivery sequence. All NVIDIA courses precede extension projects. New skills must first gain a course, '
           'then project practice, then an independent exercise. Source-specific mechanisms stay visible even when the NVIDIA adaptation differs.', '']
    for unit in catalog['units']:
        out += [f"## {unit['id']} · {unit['title']}", '',
                f"Planned chain: {unit['course']} → {unit['project']} → {', '.join(unit['exercises'])}.", '',
                '| Source locator / concepts | GPU-datacenter investigation | Required observations | Boundary and NVIDIA core |',
                '|---|---|---|---|']
        for row in catalog['objectives']:
            if row['unit'] != unit['id']: continue
            values = [f"p{row['page']} `{row['locator']}` · " + ', '.join(row['concepts']), row['investigation'],
                      row['required_evidence'], row['fidelity_boundary'] + f" Core: {row['core_unit']}."]
            out.append('| ' + ' | '.join(value.replace('|', '\\|') for value in values) + ' |')
        out.append('')
    practical = loader(root, 'education/authoring/dcit_practical.py', 'ncp_dcit_practical')
    return {'NCP-DCIT.json': json.dumps(catalog, indent=2) + '\n',
            'NCP-DCIT.md': '\n'.join(out), 'scope.json': json.dumps(scope, indent=2) + '\n',
            **practical.render(obligations)}
