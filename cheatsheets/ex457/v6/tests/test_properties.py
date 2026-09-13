"""Independent mutation oracles for audit regressions; no device I/O mocked as live."""
import copy
import ipaddress
from pathlib import Path
import pytest
import yaml
from hypothesis import given, strategies as st
from jinja2 import Environment, FileSystemLoader
import fabric as f
from contracts import topology_ok, expected_topology, inventory_ok, containerlab_version
ROOT = Path(__file__).resolve().parents[1]
BASE = yaml.safe_load((ROOT/'model-upstream.yml').read_text())['dc']
ENV = Environment(loader=FileSystemLoader(ROOT/'templates'))


def desired():
    return f.intent(copy.deepcopy(BASE), 'spine1')


def candidate():
    return ENV.get_template('fabric.conf.j2').render(fabric_intent=desired())


@given(st.lists(st.integers(64512, 65534), min_size=4, max_size=4, unique=True), st.integers(1, 254))
def test_generated_model_oracle(asns, octet):
    dc = copy.deepcopy(BASE)
    for number, (name, node) in enumerate(dc['nodes'].items()):
        node['asn'] = asns[number]
        node['loopback'] = f'10.200.{octet}.{number+1}/32'
    assert f.model(dc)
    for name, node in dc['nodes'].items():
        out = f.intent(dc, name)
        # Compute adjacency independently from endpoint identity, not fabric.intent.
        pairs = [(a, b) for link in dc['links'] for a in link['endpoints'] for b in link['endpoints']
                 if a['node'] == name and b['node'] != name]
        assert out['peers'] == {str(ipaddress.ip_interface(b['address']).ip): dc['nodes'][b['node']]['asn'] for a, b in pairs}
        assert out['peer_interfaces'] == {str(ipaddress.ip_interface(b['address']).ip): a['interface'] for a, b in pairs}
        assert out['networks'] == [node['loopback']]
        assert {x['name'] for x in out['remote_nodes']} == set(dc['nodes'])-{name}
        assert f.config_ok(ENV.get_template('fabric.conf.j2').render(fabric_intent=out), out)


@given(st.sampled_from(['duplicate_as', 'gateway', 'zero_id', 'multicast_id', 'loopback_id', 'duplicate_endpoint', 'disconnect']))
def test_invalid_model_rejected(defect):
    dc = copy.deepcopy(BASE)
    if defect == 'duplicate_as': dc['nodes']['spine2']['asn'] = dc['nodes']['spine1']['asn']
    elif defect == 'gateway': dc['nodes']['spine1']['mgmt_ip'] = '172.30.0.1'
    elif defect.endswith('_id'): dc['nodes']['spine1']['loopback'] = {'zero_id':'0.0.0.0/32','multicast_id':'224.0.0.1/32','loopback_id':'127.0.0.1/32'}[defect]
    elif defect == 'duplicate_endpoint': dc['links'][1]['endpoints'][0]['interface'] = dc['links'][0]['endpoints'][0]['interface']
    else: dc['links'] = dc['links'][:2]
    with pytest.raises(ValueError): f.model(dc)


@given(st.sampled_from(['ipv6 unicast', 'ipv4 multicast', 'l2vpn evpn', 'ipv4 vpn']))
def test_wrong_address_family_rejected(family):
    with pytest.raises(ValueError): f.config_ok(candidate().replace('address-family ipv4 unicast','address-family '+family), desired())


@given(st.sampled_from(['neighbor PG peer-group', 'neighbor 192.0.2.4 peer-group PG', 'neighbor 10.0.0.1 route-map DENY in', 'address-family ipv6 unicast']))
def test_unsupported_bgp_rejected(statement):
    text = candidate().replace(' bgp router-id', ' '+statement+'\n bgp router-id')
    with pytest.raises(ValueError): f.config_ok(text, desired())


@given(st.sampled_from(['router rip\n network 10.255.0.1/32', 'network 10.255.0.1/32', 'neighbor 10.0.0.1 remote-as 65101', 'no neighbor 10.0.0.1']))
def test_out_of_scope_statements_rejected(statement):
    with pytest.raises(ValueError): f.config_ok(candidate()+'\n'+statement+'\n', desired())


@given(st.integers(1, 254), st.booleans())
def test_address_drift_rejected_and_removed(last, secondary):
    address = f'192.0.2.{last}/32'
    text = candidate().replace('interface lo', 'interface lo\n ip address '+address+(' secondary' if secondary else ''))
    with pytest.raises(ValueError): f.config_ok(text, desired())
    assert ' no ip address '+address in f.reconcile(text, desired())


@given(st.integers(0, 4), st.integers(0, 100), st.sampled_from(['received', 'packets received']))
def test_ping_truth_table(received, loss, spelling):
    text = f'3 packets transmitted, {received} {spelling}, {loss}% packet loss'
    if received == 3 and loss == 0: assert f.ping_ok(text)
    else:
        with pytest.raises(ValueError): f.ping_ok(text)


@given(st.sampled_from(['bgp', 'zebra', 'kernel']), st.sampled_from(['extra_hop', 'wrong_egress', 'missing_hops', 'inactive']))
def test_route_mutation_rejected(layer, defect):
    want = desired(); prefix = '10.255.1.1/32'; ip = '10.0.0.1'
    if layer == 'bgp':
        if defect == 'wrong_egress': defect = 'extra_hop'  # BGP JSON has no egress contract; RIB/kernel do.
        data = {'prefix':prefix,'paths':[{'valid':True,'bestpath':{'overall':True},'nexthops':[{'ip':ip}]}]}
        hops = data['paths'][0]['nexthops']
        if defect == 'inactive': data['paths'][0]['valid'] = False
        verify = f.bgp_ok
    elif layer == 'zebra':
        data = {prefix:[{'protocol':'bgp','selected':True,'installed':True,'nexthops':[{'ip':ip,'interfaceName':'eth1','active':True,'fib':True}]}]}
        hops = data[prefix][0]['nexthops']; verify = f.rib_ok
        if defect == 'wrong_egress': hops[0]['interfaceName'] = 'eth0'
        if defect == 'inactive': hops[0]['fib'] = False
    else:
        data = [{'dst':prefix,'nexthops':[{'gateway':ip,'dev':'eth1'}]}]
        hops = data[0]['nexthops']; verify = f.kernel_ok
        if defect == 'wrong_egress': hops[0]['dev'] = 'eth0'
        if defect == 'inactive': hops[0]['flags'] = ['linkdown']
    if defect == 'extra_hop': hops.append({'ip':'192.0.2.254','gateway':'192.0.2.254','interfaceName':'eth1','dev':'eth1','active':True,'fib':True})
    if defect == 'missing_hops': hops.clear()
    with pytest.raises(ValueError): verify(data, prefix, want)


@given(st.lists(st.sampled_from(['10.0.0.1','10.0.0.3']), min_size=1, max_size=2, unique=True))
def test_all_modeled_ecmp_hops_pass(ips):
    want = desired(); prefix = '10.255.1.1/32'
    assert f.rib_ok({prefix:[{'protocol':'bgp','selected':True,'installed':True,'nexthops':[{'ip':ip,'interfaceName':want['peer_interfaces'][ip],'active':True,'fib':True} for ip in ips]}]}, prefix, want)
    assert f.kernel_ok([{'dst':prefix,'nexthops':[{'gateway':ip,'dev':want['peer_interfaces'][ip]} for ip in ips]}], prefix, want)


@given(st.sampled_from(['endpoint', 'bind', 'gateway', 'extra_node', 'image', 'mgmt']))
def test_topology_mutations_rejected(defect):
    topology = expected_topology(BASE, ROOT)
    if defect == 'endpoint': topology['topology']['links'][0]['endpoints'][0] = 'spine1:eth9'
    elif defect == 'bind': topology['topology']['nodes']['spine1']['binds'][0] = '/tmp/untrusted:/run/ex457-auth/authorized_keys:ro'
    elif defect == 'gateway': topology['mgmt']['ipv4-gw'] = '172.30.0.11'
    elif defect == 'extra_node': topology['topology']['nodes']['rogue'] = {}
    elif defect == 'image': topology['topology']['defaults']['image'] = 'unreviewed:latest'
    else: topology['topology']['nodes']['spine1']['mgmt-ipv4'] = '172.30.0.12'
    with pytest.raises(ValueError): topology_ok(topology, BASE, ROOT)


def test_inventory_address_swap_rejected():
    inv = yaml.safe_load((ROOT/'inventory/network.yml').read_text())
    assert inventory_ok(inv, BASE)
    hosts = inv['all']['children']['fabric']['children']['spines']['hosts']
    hosts['spine1']['ansible_host'], hosts['spine2']['ansible_host'] = hosts['spine2']['ansible_host'], hosts['spine1']['ansible_host']
    with pytest.raises(ValueError): inventory_ok(inv, BASE)


@given(st.sampled_from(['1','-dev','-rc1','.1','x','+dirty']))
def test_exact_containerlab_version(suffix):
    assert containerlab_version('version: 0.79.0\ncommit: abc')
    with pytest.raises(ValueError): containerlab_version('version: 0.79.0'+suffix)
