"""Fail-closed lab model and FRR predicates, shared by Ansible and unit tests.

Scope: default VRF, IPv4 unicast, numeric eBGP peers, all non-eth0 IPv4
configuration. Unsupported managed syntax raises an error; it is never ignored.
"""
import ipaddress
import json
import re


def require(condition, message):
    if not condition:
        raise ValueError(message)


def obj(value):
    return json.loads(value) if isinstance(value, str) else value


def interface(value):
    return str(ipaddress.IPv4Interface(value))


def model(dc):
    nodes = dc['nodes']
    require(len(nodes) >= 2, 'at least two nodes required')
    subnet = ipaddress.IPv4Network(dc['mgmt']['subnet'])
    mgmt, loops, used, addresses, networks = set(), set(), set(), set(), set()
    adjacency = {n: set() for n in nodes}
    for name, node in nodes.items():
        require(re.fullmatch(r'[a-z][a-z0-9-]*', name), 'unsafe node name')
        require(node['role'] in ('spine', 'leaf'), 'this pack models routers only')
        ip = ipaddress.IPv4Address(node['mgmt_ip'])
        lo = ipaddress.IPv4Interface(node['loopback'])
        require(ip in subnet and ip not in (subnet.network_address, subnet.broadcast_address), 'mgmt subnet')
        require(ip not in mgmt, 'duplicate mgmt IP')
        require(lo.network.prefixlen == 32 and lo.ip not in loops, 'duplicate or non-/32 loopback')
        require(lo.ip not in subnet, 'loopback overlaps management')
        require(isinstance(node['asn'], int) and not isinstance(node['asn'], bool) and 1 <= node['asn'] <= 4294967295, 'ASN range/type')
        mgmt.add(ip); loops.add(lo.ip)
    for link in dc['links']:
        ends = link['endpoints']
        require(len(ends) == 2, 'link must have two endpoints')
        a, b = ends
        require(a['node'] != b['node'], 'self link')
        ips = []
        for end in ends:
            require(end['node'] in nodes, 'unknown endpoint')
            require(re.fullmatch(r'eth[1-9][0-9]*', end['interface']), 'managed interface must be eth1+')
            key = (end['node'], end['interface'])
            require(key not in used, 'duplicate interface')
            used.add(key)
            ip = ipaddress.IPv4Interface(end['address'])
            require(ip.network.prefixlen == 31, 'link must be /31')
            require(ip.ip not in addresses and ip.ip not in loops and ip.ip not in subnet, 'duplicate/overlapping link address')
            addresses.add(ip.ip); ips.append(ip)
        require(ips[0].network == ips[1].network, 'endpoints not same /31')
        require(ips[0].network not in networks, 'duplicate link subnet')
        require(nodes[a['node']]['role'] != nodes[b['node']]['role'], 'spine-leaf links required')
        require(nodes[a['node']]['asn'] != nodes[b['node']]['asn'], 'eBGP needs different ASNs')
        networks.add(ips[0].network)
        adjacency[a['node']].add(b['node']); adjacency[b['node']].add(a['node'])
    seen, pending = set(), [next(iter(nodes))]
    while pending:
        n = pending.pop()
        if n not in seen:
            seen.add(n); pending.extend(adjacency[n] - seen)
    require(seen == set(nodes), 'disconnected model')
    return True


def intent(dc, name):
    model(dc)
    node = dc['nodes'][name]
    result = {'interfaces': {'lo': [interface(node['loopback'])]}, 'peers': {},
              'asn': node['asn'], 'router_id': str(ipaddress.IPv4Interface(node['loopback']).ip),
              'networks': [str(ipaddress.IPv4Network(node['loopback']))], 'remote_nodes': []}
    for link in dc['links']:
        for i, end in enumerate(link['endpoints']):
            if end['node'] == name:
                peer = link['endpoints'][1-i]
                result['interfaces'][end['interface']] = [interface(end['address'])]
                result['peers'][str(ipaddress.IPv4Interface(peer['address']).ip)] = dc['nodes'][peer['node']]['asn']
    result['remote_nodes'] = [dict(name=n, loopback=x['loopback'])
                              for n, x in sorted(dc['nodes'].items()) if n != name]
    return result


def config(text):
    result = {'interfaces': {}, 'peers': {}, 'asn': None, 'router_id': None, 'networks': []}
    context = None
    for raw in text.splitlines():
        line = raw.strip()
        if line.startswith('interface '):
            require(len(line.split()) == 2, 'unsupported VRF/interface syntax')
            context = line.split()[1]
        elif line.startswith('router bgp '):
            require(len(line.split()) == 3, 'unsupported BGP VRF/view')
            require(result['asn'] is None, 'multiple BGP instances')
            result['asn'] = int(line.split()[2]); context = None
        elif line in ('!', 'exit', 'end'):
            context = None
        elif line.startswith('ip address '):
            require(context is not None, 'address outside interface')
            if context == 'eth0':
                continue
            parts = line.split()[2:]
            if '/' in parts[0]:
                value, suffix = parts[0], parts[1:]
            else:
                require(len(parts) >= 2, 'missing address mask')
                value, suffix = parts[0] + '/' + parts[1], parts[2:]
            require(suffix in ([], ['secondary']), 'unsupported IPv4 address suffix')
            result['interfaces'].setdefault(context, []).append(interface(value))
        elif line.startswith('neighbor ') and ' remote-as ' in line:
            p = line.split()
            require(len(p) == 4, 'unsupported remote-as syntax')
            ip = str(ipaddress.IPv4Address(p[1]))
            require(ip not in result['peers'], 'duplicate neighbor')
            result['peers'][ip] = int(p[3])
        elif line.startswith('bgp router-id '):
            result['router_id'] = str(ipaddress.IPv4Address(line.split()[2]))
        elif line.startswith('network '):
            p = line.split()
            require(len(p) == 2, 'unsupported network statement')
            result['networks'].append(str(ipaddress.IPv4Network(p[1])))
    for key in result['interfaces']:
        result['interfaces'][key].sort()
    result['networks'].sort()
    return result


def config_ok(text, desired):
    observed = config(text)
    for key in observed:
        require(observed[key] == desired[key], 'managed config mismatch: ' + key)
    return True


def peers_ok(value, desired):
    data = obj(value)
    peers = data.get('peers', data.get('ipv4Unicast', {}).get('peers'))
    require(isinstance(peers, dict), 'missing peers object')
    require(set(peers) == set(desired['peers']), 'peer set mismatch')
    for ip, asn in desired['peers'].items():
        require(peers[ip].get('state') == 'Established', 'peer not Established: ' + ip)
        require(int(peers[ip].get('remoteAs', -1)) == asn, 'remote ASN mismatch: ' + ip)
    return True


def peers_ready(value, desired):
    try:
        return peers_ok(value, desired)
    except (ValueError, TypeError, KeyError):
        return False


def bgp_ok(value, prefix, desired):
    data = obj(value)
    require(data.get('prefix') == prefix, 'BGP prefix mismatch')
    paths = data.get('paths', [])
    good = [p for p in paths if p.get('valid') is True and p.get('bestpath', {}).get('overall') is True
            and any(n.get('ip') in desired['peers'] for n in p.get('nexthops', []))]
    require(bool(good), 'no valid best BGP path via modeled peer')
    return True


def rib_ok(value, prefix, desired):
    data = obj(value)
    routes = data.get(prefix, [])
    require(isinstance(routes, list), 'route list missing')
    good = [r for r in routes if r.get('protocol') == 'bgp' and r.get('selected') is True
            and r.get('installed') is True and any(n.get('ip') in desired['peers']
            and n.get('active') is True and n.get('fib') is True for n in r.get('nexthops', []))]
    require(bool(good), 'no selected installed BGP route with active FIB nexthop')
    return True


def ping_ok(text):
    # Require one complete summary and exact counts; malformed/localized output fails.
    matches = re.findall(r'(?m)^\s*(\d+) packets transmitted,\s*(\d+) (?:packets )?received,\s*(?:\+\d+ errors,\s*)?(\d+(?:\.\d+)?)% packet loss(?:,.*)?\s*$', text)
    require(len(matches) == 1, 'missing or ambiguous ping summary')
    sent, received, loss = matches[0]
    require(int(sent) == 3 and int(received) == 3 and float(loss) == 0.0, 'three probes and zero loss required')
    return True


def reconcile(text, desired):
    old = config(text)
    lines = []
    # Address removals are explicit and limited to the documented managed scope.
    for iface, values in sorted(old['interfaces'].items()):
        stale = sorted(set(values) - set(desired['interfaces'].get(iface, [])))
        if stale:
            lines += ['interface ' + iface] + [' no ip address ' + a for a in stale] + ['exit']
    if old['asn'] is not None and old['asn'] != desired['asn']:
        lines += ['no router bgp ' + str(old['asn'])]
    elif old['asn'] is not None:
        stale = [ip for ip, asn in old['peers'].items() if desired['peers'].get(ip) != asn]
        stale_net = set(old['networks']) - set(desired['networks'])
        if stale or stale_net:
            lines += ['router bgp ' + str(old['asn'])]
            lines += [' no neighbor ' + ip for ip in sorted(stale)]
            if stale_net:
                lines += [' address-family ipv4 unicast'] + ['  no network ' + n for n in sorted(stale_net)] + [' exit-address-family']
            lines += ['exit']
    return '\n'.join(lines)


class FilterModule:
    def filters(self):
        return {'fabric_model': model, 'fabric_intent': intent, 'fabric_config': config,
                'fabric_config_ok': config_ok, 'fabric_peers_ok': peers_ok,
                'fabric_peers_ready': peers_ready, 'fabric_bgp_ok': bgp_ok,
                'fabric_rib_ok': rib_ok, 'fabric_ping_ok': ping_ok,
                'fabric_reconcile': reconcile}
