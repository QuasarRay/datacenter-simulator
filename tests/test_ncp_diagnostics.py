import copy
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from integrations.incus.diagnostics import CHECKS, Endpoint, Flow, PathStudy, Port, resolve
from integrations.incus.path_probe import probe
from verification.dsl import ContractError
from verification.evidence import Checklist


class NativeDiagnosticBoundary(unittest.TestCase):
    def test_netbox_names_resolve_to_unique_ethernet_host_ports(self):
        spec = {'role': 'host', 'interfaces': {'p1': {}}, 'labels': {
            'netbox.device_name': 'gpu-a', 'simulator.medium': 'ethernet',
            'netbox.interfaces': {'p1': {'id': 1, 'name': 'data0'}}}}
        manifest = {'content': {'nodes': {'nb1': spec}}}
        self.assertEqual(resolve(manifest, {'nb1:p1': '10.0.0.1'}, Endpoint('gpu-a', 'data0')), Port('nb1', 'p1', '10.0.0.1'))
        for change in ('duplicate', 'switch', 'infiniband', 'wrong-port'):
            bad = copy.deepcopy(manifest)
            if change == 'duplicate': bad['content']['nodes']['nb2'] = spec
            if change == 'switch': bad['content']['nodes']['nb1']['role'] = 'switch'
            if change == 'infiniband': bad['content']['nodes']['nb1']['labels']['simulator.medium'] = 'infiniband'
            if change == 'wrong-port': bad['content']['nodes']['nb1']['interfaces'] = {}
            with self.subTest(change=change), self.assertRaises(ContractError):
                resolve(bad, {'nb1:p1': '10.0.0.1'}, Endpoint('gpu-a', 'data0'))

    def test_probe_rejects_unsafe_input_before_opening_a_socket(self):
        config = dict(interface='p1', address='10.0.0.1', peer='10.0.0.2', nonce='a'*64, port=21000)
        for field, value in [('interface', '../x'), ('address', 'localhost'), ('peer', '::1'), ('nonce', 'short'), ('port', True)]:
            with self.subTest(field=field), self.assertRaises(ValueError): probe('client', config | {field: value})

    def exercise(self, directory, defect=None):
        directory = Path(directory); (directory/'state').mkdir(exist_ok=True)
        state = {'up': True, 'tx': 0, 'rx': 0, 'links': []}
        report = {'returncode': 0, 'deepops_revision': 'test-fixture-not-live', 'stdout': '\n'.join(
            f'nb{i} : ok=4 changed=0 unreachable=0 failed=0' for i in range(1,5))}
        fabric = SimpleNamespace(directory=directory, binary=Path('/fixture/simulator'), reconcile=lambda: report)
        flow = Flow(Endpoint('a', 'data0'), Endpoint('b', 'data0'))
        study = PathStudy(fabric, flow, flow)
        ports = tuple(Port('nb'+str(i), 'p'+str(i), '10.0.0.'+str(i)) for i in range(1,5))
        study.plan = lambda: (ports, 0, 'fixture-plan')
        def link(argv, **_):
            value = argv[-1]; state['links'].append(value)
            state['up'] = value == 'up'
            if defect == 'down-failure' and value == 'down': raise RuntimeError('ambiguous down failure')
            if defect == 'restore-failure' and value == 'up': raise RuntimeError('restore failure')
        def execute(node, argv, **_):
            mode, config = argv[-2], json.loads(argv[-1])
            result = {'schema': 'ncp-path-probe-v1', 'role': mode, 'nonce': config['nonce'],
                      'address': config['address'], 'interface': config['interface']}
            subject = node in ('nb1','nb2')
            delivered = state['up'] or not subject
            if defect == 'leaky-path' and subject: delivered = True
            if defect == 'control-loss' and not state['up']: delivered = False
            if defect == 'bad-baseline': delivered = False
            if defect == 'stale': result['nonce'] = 'b'*64
            if defect == 'tool-failure': return 127, b'', b'missing interpreter'
            if mode == 'inspect':
                if defect != 'no-counter-change': state['tx'] += 2; state['rx'] += 2
                result.update(tx=state['tx'], rx=state['rx'])
            elif mode == 'server': result.update(listening=True, received=delivered)
            else: result.update(delivered=delivered)
            return 0, json.dumps(result).encode(), b''
        api = SimpleNamespace(nodes={p.node for p in ports}, checked=lambda node: None, execute=execute)
        return study, state, api, link

    def test_complete_native_report_requires_every_causal_check_but_never_awards_mastery(self):
        with tempfile.TemporaryDirectory() as directory:
            study, state, api, link = self.exercise(directory)
            with patch('integrations.incus.diagnostics.Incus', return_value=api), patch('integrations.incus.diagnostics.subprocess.run', side_effect=link):
                result = study.run(Path(directory)/'result.json')
            self.assertEqual(state['links'], ['down','up'])
            self.assertIs(result['complete'], True)
            self.assertIs(result['live_mastery_enabled'], False)
            Checklist.validate(result['checklist'], CHECKS, result['checklist']['identity'])
            self.assertFalse((Path(directory)/'state/diagnostic.lock').exists())

    def test_each_failure_stays_incomplete_and_faults_restore_even_after_ambiguous_down(self):
        for defect in ('bad-baseline','tool-failure','stale','no-counter-change','leaky-path','control-loss','down-failure','restore-failure'):
            with self.subTest(defect=defect), tempfile.TemporaryDirectory() as directory:
                study, state, api, link = self.exercise(directory, defect)
                with patch('integrations.incus.diagnostics.Incus', return_value=api), patch('integrations.incus.diagnostics.subprocess.run', side_effect=link):
                    with self.assertRaises((ContractError, RuntimeError)): study.run(Path(directory)/'result.json')
                result = json.loads((Path(directory)/'result.json').read_text())
                self.assertIs(result['complete'], False)
                if 'down' in state['links']: self.assertEqual(state['links'][-1], 'up')
                else: self.assertEqual(state['links'], [])
                self.assertFalse((Path(directory)/'state/diagnostic.lock').exists())

    def test_earlier_report_and_in_progress_study_cannot_be_overwritten(self):
        with tempfile.TemporaryDirectory() as directory:
            study, state, api, link = self.exercise(directory)
            output=Path(directory)/'result.json'; output.write_text('earlier receipt')
            with self.assertRaises(ContractError): study.run(output)
            self.assertEqual(output.read_text(), 'earlier receipt')
            output.unlink(); (Path(directory)/'state/diagnostic.lock').touch()
            with patch('integrations.incus.diagnostics.Incus', return_value=api), self.assertRaises(FileExistsError): study.run(output)
            self.assertFalse(output.exists())
            self.assertEqual(state['links'], [])


if __name__ == '__main__': unittest.main()
