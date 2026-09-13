#!/usr/bin/env python3
"""Explicit lab lifecycle commands; all operations use argument vectors."""
import argparse
import hashlib
import ipaddress
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import yaml
from jinja2 import Environment, FileSystemLoader, StrictUndefined

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'filter_plugins'))
from fabric import intent, model, ping_ok, require, kernel_ok, config_ok
from contracts import topology_ok, containerlab_version
from state import secure_dir, recorded_run
STATE = ROOT / '.state'
DC = yaml.safe_load((ROOT / 'model-upstream.yml').read_text())['dc']


def run(args, privileged=False, check=True):
    prefix = ['sudo'] if privileged and os.geteuid() != 0 else []
    r = subprocess.run(prefix + list(map(str, args)), text=True, capture_output=True)
    if check and r.returncode:
        raise RuntimeError(f'{args[0]} failed ({r.returncode}): {r.stderr or r.stdout}')
    return r


def prepare(public_key):
    model(DC)
    key = Path(public_key).expanduser().read_text().strip()
    require(key.startswith(('ssh-ed25519 ', 'ssh-rsa ', 'ecdsa-sha2-')), 'supply a public key, never a private key')
    require('\n' not in key, 'one public client key required')
    run(['ssh-keygen', '-l', '-f', Path(public_key).expanduser()])
    secure_dir(STATE)
    auth = STATE / 'authorized_keys'; auth.write_text(key + '\n'); auth.chmod(0o600)
    lines = []
    for name, node in sorted(DC['nodes'].items()):
        folder = secure_dir(secure_dir(STATE / 'hostkeys') / name)
        private = folder / 'ssh_host_ed25519_key'
        if not private.exists():
            run(['ssh-keygen', '-q', '-t', 'ed25519', '-N', '', '-f', private])
        private.chmod(0o600)
        public = run(['ssh-keygen', '-y', '-f', private]).stdout.strip()
        Path(str(private) + '.pub').write_text(public + '\n')
        lines.append(node['mgmt_ip'] + ' ' + public)
    (STATE / 'known_hosts').write_text('\n'.join(lines) + '\n')
    env = Environment(loader=FileSystemLoader(ROOT / 'templates'), undefined=StrictUndefined)
    topology = env.get_template('datacenter.clab.yml.j2').render(dc=DC, pack_root=str(ROOT))
    (STATE / 'datacenter.clab.yml').write_text(topology + '\n')
    print('Prepared topology and private host identities; public trust: .state/known_hosts')


def preflight():
    model(DC)
    require(' ' not in str(ROOT), 'extract under a path without spaces')
    for p in ['authorized_keys', 'known_hosts', 'datacenter.clab.yml']:
        require((STATE / p).is_file() and (STATE / p).stat().st_size > 0, 'missing ' + p)
    require(STATE.stat().st_mode & 0o777 == 0o700, 'state directory must be mode 0700')
    require((STATE/'authorized_keys').stat().st_mode & 0o777 == 0o600, 'authorized_keys must be mode 0600')
    key = (STATE/'authorized_keys').read_text().strip()
    require('\n' not in key, 'exactly one client authorization key required')
    run(['ssh-keygen', '-l', '-f', STATE/'authorized_keys'])
    topology_ok((STATE/'datacenter.clab.yml').read_text(), DC, ROOT, check_binds=True)
    trust = (STATE / 'known_hosts').read_text()
    for name, node in DC['nodes'].items():
        p = STATE / 'hostkeys' / name / 'ssh_host_ed25519_key'
        require(p.is_file() and p.stat().st_mode & 0o777 == 0o600, 'host private key absent/wrong mode: ' + name)
        public = run(['ssh-keygen', '-y', '-f', p]).stdout.strip()
        require(Path(str(p)+'.pub').read_text().strip() == public, 'public/private mismatch')
        require(node['mgmt_ip'] + ' ' + public in trust.splitlines(), 'trust mismatch')
    version = run(['containerlab', 'version']).stdout
    containerlab_version(version)
    image = json.loads(run(['docker', 'image', 'inspect', 'ex457-frr:v6'], privileged=True).stdout)
    require(bool(image), 'adapter image missing')
    print('Preflight passed: model, bind files, trust, host-key modes, image, Containerlab version')


def wait_ready():
    for name in DC['nodes']:
        for attempt in range(60):
            r = run(['docker', 'exec', 'clab-ex457-v6-' + name, 'vtysh', '-c', 'show version'], privileged=True, check=False)
            if r.returncode == 0:
                break
            time.sleep(1)
        else:
            raise RuntimeError('FRR readiness timeout: ' + name)


def dataplane():
    with recorded_run(ROOT, 'dataplane', STATE) as record:
        _dataplane(record['results'])


def _dataplane(transcript):
    for name in DC['nodes']:
        desired = intent(DC, name)
        container = 'clab-ex457-v6-' + name
        for remote in desired['remote_nodes']:
            prefix = remote['loopback']
            route = run(['docker', 'exec', container, 'ip', '-j', '-4', 'route', 'show', 'exact', prefix], privileged=True)
            rows = json.loads(route.stdout)
            kernel_ok(rows, prefix, desired)
            ping = run(['docker', 'exec', '-e', 'LC_ALL=C', container, 'ping', '-n', '-c', '3', '-W', '2', '-I', desired['router_id'], str(ipaddress.IPv4Interface(prefix).ip)], privileged=True)
            ping_ok(ping.stdout)
            transcript.append({'node': name, 'prefix': prefix, 'kernel': rows, 'ping': ping.stdout})
    print(f'{len(transcript)} sourced-ping/kernel-route pairs passed')


def saved(node=None):
    names = [node] if node else sorted(DC['nodes'])
    require(set(names) <= set(DC['nodes']), 'unknown saved-state node')
    with recorded_run(ROOT, 'saved', STATE) as record:
        for name in names:
            text = run(['docker', 'exec', 'clab-ex457-v6-' + name, 'cat', '/etc/frr/frr.conf'], privileged=True).stdout
            config_ok(text, intent(DC, name))
            record['results'].append({'node': name, 'file': '/etc/frr/frr.conf', 'sha256': hashlib.sha256(text.encode()).hexdigest()})


def main():
    p = argparse.ArgumentParser()
    sub = p.add_subparsers(dest='action', required=True)
    s = sub.add_parser('prepare'); s.add_argument('--public-key', required=True)
    s = sub.add_parser('build-image'); s.add_argument('--base', required=True)
    s = sub.add_parser('saved'); s.add_argument('--node')
    for a in ['preflight', 'deploy', 'restart', 'dataplane', 'destroy']:
        sub.add_parser(a)
    args = p.parse_args()
    if args.action == 'prepare':
        prepare(args.public_key)
    elif args.action == 'build-image':
        import re
        require(re.fullmatch(r'[^\s@]+@sha256:[0-9a-f]{64}', args.base), 'supply an inspected containerlab-flavor FRR base digest')
        secure_dir(STATE)
        build = run(['docker', 'build', '--build-arg', 'FRR_BASE=' + args.base, '-t', 'ex457-frr:v6', ROOT / 'lab/frr'], privileged=True)
        (STATE / 'image-build.log').write_text(build.stdout + build.stderr)
        (STATE / 'image-inspect.json').write_text(run(['docker', 'image', 'inspect', 'ex457-frr:v6'], privileged=True).stdout)
        (STATE / 'base-image.txt').write_text(args.base + '\n')
    elif args.action == 'preflight':
        preflight()
    elif args.action == 'dataplane':
        dataplane()
    elif args.action == 'saved':
        saved(args.node)
    elif args.action in ('deploy', 'restart'):
        preflight()
        r = run(['containerlab', args.action, '--topo', STATE / 'datacenter.clab.yml'], privileged=True)
        (STATE / (args.action + '.log')).write_text(r.stdout + r.stderr)
        wait_ready()
        if args.action == 'restart':
            # Lifecycle restores links; explicitly replay saved integrated config.
            reloads = []
            for name in DC['nodes']:
                loaded = run(['docker', 'exec', 'clab-ex457-v6-' + name,
                              '/usr/local/sbin/ex457-load'], privileged=True)
                reloads.append({'node': name, 'stdout': loaded.stdout, 'stderr': loaded.stderr})
            (STATE / 'restart-reload.json').write_text(json.dumps(reloads, indent=2) + '\n')
        print('Lifecycle complete; run verify_fabric.yml and labctl dataplane next')
    elif args.action == 'destroy':
        run(['containerlab', 'destroy', '--topo', STATE / 'datacenter.clab.yml', '--cleanup'], privileged=True)
        run([sys.executable, ROOT.parents[2] / 'ci/cleanup_network.py', '--name', 'ex457-v6-mgmt', '--subnet', DC['mgmt']['subnet']], privileged=True)
        print('Lab destroyed; private host identities and backups retained in .state')


if __name__ == '__main__':
    main()
