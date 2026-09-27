"""Compile a NetBox-named path investigation into bounded native observations.

Trainer API only. The caller already owns the lab and controls trusted source.
Reports are formative, not station receipts or assertions of GPU execution.
"""
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from functools import wraps
import hashlib
import ipaddress
import json
import os
from pathlib import Path
import re
import secrets
import subprocess

from .transport import Incus
from verification.dsl import require
from verification.evidence import Checklist
from verification.io import strict_json

CHECKS = ('owned-endpoints', 'deepops-idempotent', 'baseline-subject', 'baseline-control',
          'fault-isolates-subject', 'control-survives-fault', 'link-restored',
          'recovered-subject', 'recovered-control')


@dataclass(frozen=True)
class Endpoint:
    device: str
    interface: str


@dataclass(frozen=True)
class Flow:
    source: Endpoint
    target: Endpoint


@dataclass(frozen=True)
class Port:
    node: str
    interface: str
    address: str


def resolve(manifest, addresses, endpoint):
    require(isinstance(endpoint, Endpoint), 'use a typed NetBox endpoint')
    matches = [(node, spec) for node, spec in manifest['content']['nodes'].items()
               if spec['labels'].get('netbox.device_name') == endpoint.device]
    require(len(matches) == 1, 'NetBox device name must select exactly one scoped object')
    node, spec = matches[0]
    require(spec['role'] == 'host' and spec['labels']['simulator.medium'] == 'ethernet',
            'path probes require Ethernet host endpoints')
    ports = [port for port, data in spec['labels']['netbox.interfaces'].items() if data['name'] == endpoint.interface]
    require(len(ports) == 1 and ports[0] in spec['interfaces'], 'unknown or ambiguous NetBox interface')
    port = ports[0]
    require(re.fullmatch(r'[a-zA-Z0-9_-]{1,15}', port) is not None, 'invalid generated Linux interface')
    address = str(ipaddress.IPv4Address(addresses[node + ':' + port]))
    return Port(node, port, address)


class PathStudy:
    def __init__(self, fabric, subject, control):
        require(isinstance(subject, Flow) and isinstance(control, Flow), 'two typed flows required')
        self.fabric, self.subject, self.control = fabric, subject, control

    def plan(self):
        ready = self.fabric.verify()
        directory = self.fabric.directory
        manifest = strict_json((directory / 'manifest.json').read_text())
        variables = strict_json((directory / 'provisioning-vars.json').read_text())
        journal = strict_json((directory / 'state/incus.json').read_text())
        require(journal['phase'] == 'active', 'diagnostic requires an active owned lab')
        require(Path(journal['config']['manifest']) == directory / 'manifest.json', 'journal belongs to a different plan')
        ports = tuple(resolve(manifest, variables['ncp_addresses'], endpoint)
                      for flow in (self.subject, self.control) for endpoint in (flow.source, flow.target))
        require(len({port.node for port in ports}) == 4, 'subject and control must use four distinct host endpoints')
        require(all(cable['up'] is True for cable in journal['plan']['cables']), 'start with healthy cable intent')
        source = next(node for node in journal['plan']['nodes'] if node['name'] == ports[0].node)
        peer = source['spec']['devices'][ports[0].interface]['host_name']
        cables = [i for i, cable in enumerate(journal['plan']['cables']) if peer in cable['peers']]
        require(len(cables) == 1, 'source port must select exactly one owned cable')
        identity = hashlib.sha256(json.dumps({'plan': ready, 'owner': journal['plan']['owner'],
            'ports': [vars(port) for port in ports], 'cable': cables[0]}, sort_keys=True).encode()).hexdigest()
        return ports, cables[0], identity

    def run(self, output):
        ports, cable, identity = self.plan()  # reject ambiguous intent before effects
        output = Path(output).resolve()
        require(output.parent.is_dir() and not output.exists(), 'choose a new report path in an existing trainer directory')
        source = (Path(__file__).with_name('path_probe.py')).read_text()
        api = Incus(self.fabric.directory / 'state/incus.json')
        lock = self.fabric.directory / 'state/diagnostic.lock'
        lock_fd = os.open(lock, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        os.close(lock_fd)
        # Reserve the report before fault injection, and never overwrite an earlier attempt.
        try: descriptor = os.open(output, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        except Exception:
            lock.unlink()
            raise
        report = {'schema': 'ncp-path-study-v1', 'plan_identity': identity, 'tier': 'emulated',
                  'live_mastery_enabled': False, 'gpu_execution': False, 'observations': []}
        ledger = Checklist(CHECKS, 'path-study/' + identity + '/' + secrets.token_hex(16))
        report['probe_sha256'] = hashlib.sha256(source.encode()).hexdigest()
        restore_needed = False

        def execute(port, mode, peer, nonce, number):
            config = {'interface': port.interface, 'address': port.address, 'peer': peer.address,
                      'nonce': nonce, 'port': number}
            code, out, err = api.execute(port.node, ['python3', '-c', source, mode, json.dumps(config)], timeout=20)
            require(code == 0 and len(out) <= 4096, 'native probe failed; no network outcome inferred')
            observed = strict_json(out.decode())
            require(observed.get('schema') == 'ncp-path-probe-v1' and observed.get('role') == mode
                    and observed.get('nonce') == nonce and observed.get('address') == port.address
                    and observed.get('interface') == port.interface, 'stale or misdirected probe response')
            return observed

        def trial(name, pair, expected):
            sender, receiver = pair
            nonce, number = secrets.token_hex(32), 20000 + secrets.randbelow(10000)
            before = execute(sender, 'inspect', receiver, nonce, number)
            with ThreadPoolExecutor(max_workers=2) as pool:
                listening = pool.submit(execute, receiver, 'server', sender, nonce, number)
                sent = pool.submit(execute, sender, 'client', receiver, nonce, number)
                client, server = sent.result(), listening.result()
            after = execute(sender, 'inspect', receiver, nonce, number)
            require(server.get('listening') is True, 'receiver did not establish a real bound socket')
            delivered = client.get('delivered')
            require(type(delivered) is bool and type(server.get('received')) is bool, 'probe result must be Boolean')
            require(all(type(row.get(key)) is int and row[key] >= 0 for row in (before, after) for key in ('tx', 'rx')),
                    'invalid interface counters')
            passed = delivered is expected and server['received'] is expected
            if expected: passed = passed and after['tx'] > before['tx'] and after['rx'] > before['rx']
            report['observations'].append({'check': name, 'expected_delivery': expected, 'sender': vars(sender),
                'receiver': vars(receiver), 'client': client, 'server': server, 'before': before, 'after': after})
            ledger.record(name, passed)
            require(passed, 'path postcondition failed: ' + name)

        def link(state):
            subprocess.run([str(self.fabric.binary), 'incus-link', str(self.fabric.directory / 'state'), str(cable), state],
                           check=True, capture_output=True, timeout=30)

        try:
            for port in ports: api.checked(port.node)
            ledger.record('owned-endpoints')
            first, second = self.fabric.reconcile(), self.fabric.reconcile()
            expected_hosts = len(api.nodes)
            recaps = [line for line in second['stdout'].splitlines() if 'changed=' in line and 'unreachable=' in line]
            good = first['returncode'] == second['returncode'] == 0 and len(recaps) == expected_hosts
            good = good and all(re.search(r'\bchanged=0\b', line) for line in recaps)
            report['deepops'] = {'revision': second['deepops_revision'],
                'first_sha256': hashlib.sha256(json.dumps(first, sort_keys=True).encode()).hexdigest(),
                'second': second}
            ledger.record('deepops-idempotent', bool(good))
            require(good, 'real DeepOps reconciliation must settle before the experiment')
            trial('baseline-subject', ports[:2], True)
            trial('baseline-control', ports[2:], True)
            restore_needed = True  # even an ambiguous failed down operation requires restoration
            link('down')
            trial('fault-isolates-subject', ports[:2], False)
            trial('control-survives-fault', ports[2:], True)
            link('up')
            restore_needed = False
            ledger.record('link-restored')
            trial('recovered-subject', ports[:2], True)
            trial('recovered-control', ports[2:], True)
            ledger.finish()
        finally:
            try:
                if restore_needed:
                    try: link('up')
                    except Exception:
                        ledger.record('link-restored', False)
                        raise
                    else: ledger.record('link-restored')
            finally:
                try:
                    report.update(complete=ledger.complete, checklist=ledger.report())
                    with os.fdopen(descriptor, 'w') as stream:
                        json.dump(report, stream, indent=2); stream.write('\n'); stream.flush(); os.fsync(stream.fileno())
                finally: lock.unlink()
        return report


def path_study(function):
    """Expand two symbolic flows into the shared preflight/observe/cut/restore program."""
    @wraps(function)
    def compile_study(fabric, *args, **kwargs):
        subject, control = function(*args, **kwargs)
        result = PathStudy(fabric, subject, control)
        result.plan()
        return result
    return compile_study
