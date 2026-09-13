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
    mgmt, loops, used, addresses, networks, asns = set(), set(), set(), set(), set(), set()
    gateway = ipaddress.IPv4Address(dc['mgmt'].get('gateway', str(subnet.network_address + 1)))
    require(gateway in subnet and gateway not in (subnet.network_address, subnet.broadcast_address), 'invalid management gateway')
    adjacency = {n: set() for n in nodes}
    for name, node in nodes.items():
        require(re.fullmatch(r'[a-z][a-z0-9-]*', name), 'unsafe node name')
        require(node['role'] in ('spine', 'leaf'), 'this pack models routers only')
        ip = ipaddress.IPv4Address(node['mgmt_ip'])
        lo = ipaddress.IPv4Interface(node['loopback'])
        require(ip in subnet and ip not in (subnet.network_address, subnet.broadcast_address), 'mgmt subnet')
        require(ip not in mgmt, 'duplicate mgmt IP')
        require(ip != gateway, 'management gateway collision')
        require(lo.network.prefixlen == 32 and lo.ip not in loops, 'duplicate or non-/32 loopback')
        require(not (lo.ip.is_unspecified or lo.ip.is_multicast or lo.ip.is_loopback or lo.ip.is_reserved or lo.ip.is_link_local), 'loopback must be a usable unicast router ID')
        require(lo.ip not in subnet, 'loopback overlaps management')
        require(isinstance(node['asn'], int) and not isinstance(node['asn'], bool) and 1 <= node['asn'] <= 4294967295, 'ASN range/type')
        require(node['asn'] not in asns, 'unique ASNs required by this eBGP design')
        asns.add(node['asn'])
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
    result = {'interfaces': {'lo': [interface(node['loopback'])]}, 'peers': {}, 'peer_interfaces': {},
              'asn': node['asn'], 'router_id': str(ipaddress.IPv4Interface(node['loopback']).ip),
              'networks': [str(ipaddress.IPv4Network(node['loopback']))], 'ebgp_requires_policy': False, 'remote_nodes': []}
    for link in dc['links']:
        for i, end in enumerate(link['endpoints']):
            if end['node'] == name:
                peer = link['endpoints'][1-i]
                result['interfaces'][end['interface']] = [interface(end['address'])]
                result['peers'][str(ipaddress.IPv4Interface(peer['address']).ip)] = dc['nodes'][peer['node']]['asn']
                result['peer_interfaces'][str(ipaddress.IPv4Interface(peer['address']).ip)] = end['interface']
    result['remote_nodes'] = [dict(name=n, loopback=x['loopback'])
                              for n, x in sorted(dc['nodes'].items()) if n != name]
    return result


def config(text):
    """Parse the deliberately small default-VRF IPv4 grammar, preserving scopes.

    FRR comments do not terminate a routing process. Unknown managed BGP
    statements are rejected instead of silently supplying evidence of intent.
    """
    result = {'interfaces': {}, 'peers': {}, 'asn': None, 'router_id': None,
              'networks': [], 'ebgp_requires_policy': True}
    context, iface = 'global', None
    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith('!'):
            continue
        if line == 'exit-address-family':
            require(context == 'af', 'address-family exit outside address-family')
            context = 'bgp'
        elif line in ('exit', 'end'):
            require(context != 'af', 'use exit-address-family before exiting BGP')
            context, iface = 'global', None
        elif line.startswith('interface '):
            require(context == 'global', 'interface nested in routing process')
            require(len(line.split()) == 2, 'unsupported VRF/interface syntax')
            context, iface = 'interface', line.split()[1]
        elif line.startswith('router bgp '):
            require(context == 'global', 'nested routing process')
            require(len(line.split()) == 3, 'unsupported BGP VRF/view')
            require(result['asn'] is None, 'multiple BGP instances')
            result['asn'] = int(line.split()[2]); context = 'bgp'
        elif line.startswith('router '):
            raise ValueError('unsupported routing process: ' + line)
        elif line.startswith('address-family '):
            require(context == 'bgp' and line == 'address-family ipv4 unicast', 'unsupported address-family or scope')
            context = 'af'
        elif line.startswith('ip address '):
            require(context == 'interface', 'address outside interface')
            if iface == 'eth0':
                continue
            parts = line.split()[2:]
            if '/' in parts[0]:
                value, suffix = parts[0], parts[1:]
            else:
                require(len(parts) >= 2, 'missing address mask')
                value, suffix = parts[0] + '/' + parts[1], parts[2:]
            require(suffix in ([], ['secondary']), 'unsupported IPv4 address suffix')
            result['interfaces'].setdefault(iface, []).append(interface(value))
        elif line.startswith('neighbor '):
            p = line.split()
            require(context == 'bgp' and len(p) == 4 and p[2] == 'remote-as', 'unsupported managed neighbor syntax/scope')
            ip = str(ipaddress.IPv4Address(p[1]))
            require(ip not in result['peers'], 'duplicate neighbor')
            result['peers'][ip] = int(p[3])
        elif line.startswith('bgp router-id '):
            require(context == 'bgp' and len(line.split()) == 3 and result['router_id'] is None, 'router-id scope/duplicate')
            result['router_id'] = str(ipaddress.IPv4Address(line.split()[2]))
        elif line in ('bgp ebgp-requires-policy', 'no bgp ebgp-requires-policy'):
            require(context == 'bgp', 'BGP policy outside process')
            result['ebgp_requires_policy'] = not line.startswith('no ')
        elif line.startswith('network '):
            p = line.split()
            require(context == 'af' and len(p) == 2, 'network outside IPv4 unicast BGP AF or unsupported syntax')
            result['networks'].append(str(ipaddress.IPv4Network(p[1])))
        elif context in ('bgp', 'af'):
            # Cosmetic/default operational output only. Policies, peer groups,
            # other AFs and route maps are outside this small managed grammar.
            require(line == 'bgp log-neighbor-changes', 'unsupported managed BGP statement: ' + line)
        elif line.startswith(('neighbor ', 'no neighbor ', 'bgp ', 'no router ', 'no network ', 'no ip address ')):
            raise ValueError('managed statement in wrong scope: ' + line)
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
    selected = [p for p in paths if p.get('bestpath', {}).get('overall') is True]
    require(bool(selected), 'no best BGP path')
    for path in selected:
        require(path.get('valid') is True, 'best BGP path is invalid')
        hops = path.get('nexthops', [])
        require(bool(hops) and all(n.get('ip') in desired['peers'] for n in hops), 'unmodeled BGP best-path nexthop')
    return True


def rib_ok(value, prefix, desired):
    data = obj(value)
    routes = data.get(prefix, [])
    require(isinstance(routes, list), 'route list missing')
    selected = [r for r in routes if r.get('selected') is True and r.get('installed') is True]
    require(bool(selected), 'no selected installed route')
    for route in selected:
        require(route.get('protocol') == 'bgp', 'selected installed route must be BGP')
        hops = [n for n in route.get('nexthops', []) if n.get('active') is True or n.get('fib') is True]
        require(bool(hops), 'no active installed nexthop')
        for hop in hops:
            require(hop.get('active') is True and hop.get('fib') is True, 'incomplete active/FIB nexthop')
            ip = hop.get('ip')
            require(ip in desired['peers'], 'unmodeled FIB nexthop')
            require(hop.get('interfaceName') == desired['peer_interfaces'][ip], 'wrong FIB egress')
    return True


def kernel_ok(value, prefix, desired):
    rows = obj(value)
    require(isinstance(rows, list) and bool(rows), 'kernel route missing')
    wanted = str(ipaddress.IPv4Network(prefix))
    for row in rows:
        observed = row.get('dst', '')
        if '/' not in observed:
            observed += '/32'
        require(observed == wanted and row.get('type', 'unicast') == 'unicast', 'kernel prefix/type mismatch')
        hops = row.get('nexthops') or [row]
        require(bool(hops), 'kernel nexthops missing')
        for hop in hops:
            ip = hop.get('gateway')
            require(ip in desired['peers'], 'unmodeled kernel nexthop')
            require(hop.get('dev') == desired['peer_interfaces'][ip], 'wrong kernel egress')
            require(not set(hop.get('flags', [])).intersection({'dead', 'linkdown'}), 'inactive kernel nexthop')
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
                'fabric_reconcile': reconcile, 'fabric_kernel_ok': kernel_ok}
