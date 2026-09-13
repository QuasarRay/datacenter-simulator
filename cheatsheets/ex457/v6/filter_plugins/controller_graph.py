"""Validate the entire owned workflow graph before accepting job outcomes."""
EDGES = {'Backup': (['Change'], ['Diagnose']), 'Change': (['Verify'], ['Diagnose']),
         'Verify': ([], ['Diagnose']), 'Diagnose': ([], [])}


def graph_ok(nodes):
    if len(nodes) != 4 or {n['identifier'] for n in nodes} != set(EDGES):
        raise ValueError('workflow must contain exactly the four owned nodes')
    ids = {n['id']: n['identifier'] for n in nodes}
    if len(ids) != 4:
        raise ValueError('duplicate workflow node IDs')
    for node in nodes:
        name = node['identifier']
        for key, expected in [('success_nodes', EDGES[name][0]), ('failure_nodes', EDGES[name][1]), ('always_nodes', [])]:
            observed = [ids[i] for i in node[key]]
            if sorted(observed) != sorted(expected):
                raise ValueError('workflow edge mismatch: '+name+' '+key)
        if node['summary_fields']['unified_job_template']['name'] != 'EX457 '+name:
            raise ValueError('workflow node references wrong job template')
    return True


class FilterModule:
    def filters(self):
        return {'ex457_workflow_graph_ok': graph_ok}
