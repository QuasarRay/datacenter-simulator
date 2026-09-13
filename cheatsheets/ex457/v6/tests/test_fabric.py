import copy
import json
from pathlib import Path
import sys
import unittest
import yaml
from jinja2 import Environment, FileSystemLoader, StrictUndefined

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'filter_plugins'))
import fabric as f


class FabricTests(unittest.TestCase):
    def setUp(self):
        self.dc = yaml.safe_load((ROOT / 'model-upstream.yml').read_text())['dc']
        self.want = f.intent(self.dc, 'spine1')
        self.env = Environment(loader=FileSystemLoader(ROOT / 'templates'), undefined=StrictUndefined)
        self.text = self.env.get_template('fabric.conf.j2').render(fabric_intent=self.want)
        self.peers = {'peers': {ip: {'state': 'Established', 'remoteAs': asn} for ip, asn in self.want['peers'].items()}}
        self.prefix = '10.255.1.1/32'
        self.bgp = {'prefix': self.prefix, 'paths': [{'valid': True, 'bestpath': {'overall': True}, 'nexthops': [{'ip': '10.0.0.1'}]}]}
        self.rib = {self.prefix: [{'protocol': 'bgp', 'selected': True, 'installed': True, 'nexthops': [{'ip': '10.0.0.1', 'interfaceName': 'eth1', 'active': True, 'fib': True}]}]}

    def test_four_router_model_and_template_roundtrip(self):
        for name in self.dc['nodes']:
            desired = f.intent(self.dc, name)
            text = self.env.get_template('fabric.conf.j2').render(fabric_intent=desired)
            self.assertTrue(f.config_ok(text, desired))
            self.assertEqual(len(desired['remote_nodes']), 3)
            self.assertNotIn(name, [n['name'] for n in desired['remote_nodes']])

    def test_duplicate_management(self):
        self.dc['nodes']['leaf1']['mgmt_ip'] = '172.30.0.11'
        self.assertRaises(ValueError, f.model, self.dc)

    def test_integer_subclass_is_valid_but_boolean_asn_is_not(self):
        class TaggedInteger(int):
            pass
        self.dc['nodes']['spine1']['asn'] = TaggedInteger(65001)
        self.assertTrue(f.model(self.dc))
        self.dc['nodes']['spine1']['asn'] = True
        self.assertRaises(ValueError, f.model, self.dc)

    def test_duplicate_loopback(self):
        self.dc['nodes']['leaf1']['loopback'] = '10.255.0.1/32'
        self.assertRaises(ValueError, f.model, self.dc)

    def test_bad_prefix_length(self):
        self.dc['links'][0]['endpoints'][0]['address'] = '10.0.0.0/30'
        self.assertRaises(ValueError, f.model, self.dc)

    def test_different_link_subnets(self):
        self.dc['links'][0]['endpoints'][0]['address'] = '10.0.99.0/31'
        self.assertRaises(ValueError, f.model, self.dc)

    def test_duplicate_interface(self):
        self.dc['links'][1]['endpoints'][0]['interface'] = 'eth1'
        self.assertRaises(ValueError, f.model, self.dc)

    def test_disconnected_model(self):
        self.dc['links'] = self.dc['links'][:2]
        self.assertRaises(ValueError, f.model, self.dc)

    def test_management_overlap(self):
        self.dc['nodes']['leaf1']['loopback'] = '172.30.0.100/32'
        self.assertRaises(ValueError, f.model, self.dc)

    def test_exact_peers_pass(self):
        self.assertTrue(f.peers_ok(self.peers, self.want))

    def test_extra_peer_any_state_fails(self):
        for state in ['Established', 'Active', 'Idle']:
            with self.subTest(state=state):
                p = copy.deepcopy(self.peers)
                p['peers']['192.0.2.1'] = {'state': state, 'remoteAs': 65099}
                self.assertRaises(ValueError, f.peers_ok, p, self.want)

    def test_missing_peer_fails(self):
        self.peers['peers'].pop('10.0.0.1')
        self.assertFalse(f.peers_ready(self.peers, self.want))

    def test_wrong_asn_fails(self):
        self.peers['peers']['10.0.0.1']['remoteAs'] = 65199
        self.assertRaises(ValueError, f.peers_ok, self.peers, self.want)

    def test_idle_peer_fails(self):
        self.peers['peers']['10.0.0.1']['state'] = 'Idle'
        self.assertFalse(f.peers_ready(self.peers, self.want))

    def test_missing_summary_schema_fails(self):
        self.assertFalse(f.peers_ready({'unexpected': {}}, self.want))

    def test_address_interface_swap_fails(self):
        text = self.text.replace('interface eth1', 'interface TMP').replace('interface eth2', 'interface eth1').replace('interface TMP', 'interface eth2')
        self.assertRaises(ValueError, f.config_ok, text, self.want)

    def test_extra_loopback_address_fails(self):
        text = self.text.replace('interface lo', 'interface lo\n ip address 192.0.2.3/32 secondary')
        self.assertRaises(ValueError, f.config_ok, text, self.want)

    def test_netmask_normalized(self):
        text = self.text.replace('10.0.0.0/31', '10.0.0.0 255.255.255.254')
        self.assertTrue(f.config_ok(text, self.want))

    def test_unknown_address_suffix_fails_closed(self):
        self.assertRaises(ValueError, f.config, self.text.replace('10.0.0.0/31', '10.0.0.0/31 invented'))

    def test_wrong_router_id_fails(self):
        self.assertRaises(ValueError, f.config_ok, self.text.replace('bgp router-id 10.255.0.1', 'bgp router-id 192.0.2.1'), self.want)

    def test_extra_network_fails(self):
        self.assertRaises(ValueError, f.config_ok, self.text + '\nnetwork 192.0.2.1/32\n', self.want)

    def test_valid_best_path(self):
        self.assertTrue(f.bgp_ok(self.bgp, self.prefix, self.want))

    def test_nonempty_wrong_prefix_fails(self):
        self.bgp['prefix'] = '192.0.2.1/32'
        self.assertRaises(ValueError, f.bgp_ok, self.bgp, self.prefix, self.want)

    def test_invalid_path_fails(self):
        self.bgp['paths'][0]['valid'] = False
        self.assertRaises(ValueError, f.bgp_ok, self.bgp, self.prefix, self.want)

    def test_nonbest_path_fails(self):
        self.bgp['paths'][0]['bestpath']['overall'] = False
        self.assertRaises(ValueError, f.bgp_ok, self.bgp, self.prefix, self.want)

    def test_unmodeled_nexthop_fails(self):
        self.bgp['paths'][0]['nexthops'][0]['ip'] = '192.0.2.1'
        self.assertRaises(ValueError, f.bgp_ok, self.bgp, self.prefix, self.want)

    def test_selected_installed_route(self):
        self.assertTrue(f.rib_ok(self.rib, self.prefix, self.want))

    def test_uninstalled_route_fails(self):
        self.rib[self.prefix][0]['installed'] = False
        self.assertRaises(ValueError, f.rib_ok, self.rib, self.prefix, self.want)

    def test_nonfib_nexthop_fails(self):
        self.rib[self.prefix][0]['nexthops'][0]['fib'] = False
        self.assertRaises(ValueError, f.rib_ok, self.rib, self.prefix, self.want)

    def test_missing_route_fails(self):
        self.assertRaises(ValueError, f.rib_ok, {}, self.prefix, self.want)

    def test_zero_loss_passes(self):
        for text in ['3 packets transmitted, 3 received, 0% packet loss, time 2000ms', '3 packets transmitted, 3 packets received, 0.0% packet loss']:
            self.assertTrue(f.ping_ok(text))

    def test_packet_loss_truth_table(self):
        for received, loss in [(2, '33'), (1, '66'), (0, '100'), (3, '0.1')]:
            with self.subTest(loss=loss):
                self.assertRaises(ValueError, f.ping_ok, f'3 packets transmitted, {received} received, {loss}% packet loss')

    def test_bad_counts_fail(self):
        self.assertRaises(ValueError, f.ping_ok, '0 packets transmitted, 0 received, 0% packet loss')

    def test_missing_ping_summary_fails(self):
        self.assertRaises(ValueError, f.ping_ok, 'ping: command not found')

    def test_duplicate_ping_summary_fails(self):
        self.assertRaises(ValueError, f.ping_ok, ('3 packets transmitted, 3 received, 0% packet loss\n')*2)

    def test_reconciliation_removes_stale_state(self):
        text = self.text.replace('interface lo', 'interface lo\n ip address 192.0.2.1/32').replace(' bgp router-id', ' neighbor 192.0.2.2 remote-as 65099\n bgp router-id').replace(' address-family ipv4 unicast', ' address-family ipv4 unicast\n  network 192.0.2.1/32')
        out = f.reconcile(text, self.want)
        self.assertIn('no ip address 192.0.2.1/32', out)
        self.assertIn('no neighbor 192.0.2.2', out)
        self.assertIn('no network 192.0.2.1/32', out)
        self.assertNotIn('no router bgp', out)

    def test_already_converged_has_no_removals(self):
        self.assertEqual(f.reconcile(self.text, self.want), '')

    def test_strict_template_rejects_undefined_intent(self):
        self.assertRaises(Exception, self.env.get_template('fabric.conf.j2').render)


if __name__ == '__main__':
    unittest.main(verbosity=2)
