"""Independent model-to-artifact contracts; do not reuse the Jinja template."""
import ipaddress
from pathlib import Path
import re
import yaml


def need(value, message):
    if not value:
        raise ValueError(message)


def containerlab_version(text):
    matches = re.findall(r'(?im)^\s*(?:version\s*[:=]?\s*|containerlab\s+v?)(v?\d+\.\d+\.\d+(?:[-+][^\s]+)?)\s*$', text)
    need(len(matches) == 1 and matches[0].lstrip('v') == '0.79.0', 'requires exact Containerlab 0.79.0')
    return True


def expected_topology(dc, root):
    root = Path(root).resolve()
    return {'name': 'ex457-v6', 'mgmt': {'network': 'ex457-v6-mgmt', 'ipv4-subnet': dc['mgmt']['subnet'],
            'ipv4-gw': dc['mgmt'].get('gateway', str(ipaddress.IPv4Network(dc['mgmt']['subnet']).network_address + 1))},
            'topology': {'defaults': {'kind': 'linux', 'image': 'ex457-frr:v6', 'sysctls': {'net.ipv4.ip_forward': '1'}},
            'nodes': {name: {'mgmt-ipv4': node['mgmt_ip'], 'binds': [
                f'{root}/.state/authorized_keys:/run/ex457-auth/authorized_keys:ro',
                f'{root}/.state/hostkeys/{name}:/run/ex457-hostkeys:ro'], 'exec': ['/usr/local/sbin/ex457-load']}
                for name, node in dc['nodes'].items()},
            'links': [{'endpoints': [end['node']+':'+end['interface'] for end in link['endpoints']]} for link in dc['links']]}}


def topology_ok(value, dc, root, check_binds=False):
    data = yaml.safe_load(value) if isinstance(value, str) else value
    need(data == expected_topology(dc, root), 'topology differs from modeled endpoints, addresses, image, binds or lifecycle')
    if check_binds:
        for node in data['topology']['nodes'].values():
            for bind in node['binds']:
                source, target, mode = bind.rsplit(':', 2)
                path = Path(source)
                need(not path.is_symlink(), 'bind source is a symlink')
                need(path.is_dir() if target == '/run/ex457-hostkeys' else path.is_file(), 'missing actual bind source')
    return True


def inventory_ok(data, dc):
    if isinstance(data, str):
        data = yaml.safe_load(data)
    fabric = data['all']['children']['fabric']['children']
    need(set(fabric) == {'spines', 'leafs'}, 'inventory role groups mismatch')
    seen = {}
    for group, role in [('spines', 'spine'), ('leafs', 'leaf')]:
        hosts = fabric[group]['hosts']
        wanted = {name for name, node in dc['nodes'].items() if node['role'] == role}
        need(set(hosts) == wanted, 'inventory role membership mismatch')
        for name, host in hosts.items():
            need(host['ansible_host'] == dc['nodes'][name]['mgmt_ip'], 'inventory management address mismatch')
            need(name not in seen, 'duplicate inventory host')
            seen[name] = host
    need(set(seen) == set(dc['nodes']), 'inventory host set mismatch')
    return True


def effective_inventory_ok(data, dc):
    hosts = data['_meta']['hostvars']
    need(set(hosts) == set(dc['nodes']), 'effective inventory host set mismatch')
    for name, node in dc['nodes'].items():
        values = hosts[name]
        need(values.get('ansible_host') == node['mgmt_ip'], 'effective management address mismatch')
        need(values.get('ansible_connection') == 'ansible.netcommon.network_cli', 'effective transport mismatch')
        need(values.get('ansible_network_cli_ssh_type') == 'libssh', 'effective SSH backend mismatch')
        need(values.get('ansible_host_key_checking') is True and values.get('ansible_libssh_host_key_checking') is True, 'effective trust disabled')
        need('ansible_libssh_config_file' not in values, 'inventory must not dereference future trust task')
    return True
