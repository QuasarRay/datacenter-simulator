"""Immutable private capture sets with a durable checksum index and TLS export."""
import argparse
from contextlib import contextmanager
import signal
import threading
import time
import hashlib
import json
import os
from pathlib import Path
import re
import ssl
import sys
import urllib.parse
import urllib.request
import yaml
from state import atomic_json, secure_dir, source_digest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT/'filter_plugins'))
from fabric import config_ok, intent, require


INDEX_LIMIT = 1024 * 1024
CONFIG_LIMIT = 4 * 1024 * 1024
CAPTURE_LIMIT = 64 * 1024 * 1024
TRANSFER_SECONDS = 120


@contextmanager
def deadline(seconds=TRANSFER_SECONDS):
    # These tools run on Linux. A process alarm also bounds DNS, TLS and a
    # peer that trickles bytes faster than the per-socket inactivity timeout.
    require(threading.current_thread() is threading.main_thread(),
            'backup transfers must run on the main thread for deadline enforcement')
    previous_handler = signal.getsignal(signal.SIGALRM)
    previous_timer = signal.getitimer(signal.ITIMER_REAL)
    started = time.monotonic()
    def expired(signum, frame):
        raise TimeoutError('backup transfer exceeded total deadline')
    signal.signal(signal.SIGALRM, expired)
    remaining = min(seconds, previous_timer[0]) if previous_timer[0] else seconds
    signal.setitimer(signal.ITIMER_REAL, remaining)
    try:
        yield
    finally:
        signal.setitimer(signal.ITIMER_REAL, 0)
        signal.signal(signal.SIGALRM, previous_handler)
        if previous_timer[0]:
            signal.setitimer(signal.ITIMER_REAL,
                            max(.000001, previous_timer[0] - (time.monotonic() - started)),
                            previous_timer[1])


def bounded_read(stream, limit):
    data = bytearray()
    while True:
        block = stream.read(min(65536, limit - len(data) + 1))
        if not block:
            return bytes(data)
        data.extend(block)
        require(len(data) <= limit, 'backup body exceeds byte limit')


def local_bytes(path, limit=CONFIG_LIMIT):
    with path.open('rb') as stream:
        return bounded_read(stream, limit)


def reserve(base, run_id):
    require(re.fullmatch(r'[A-Za-z0-9_-]+', run_id), 'unsafe backup run ID')
    base = Path(base)
    secure_dir(base.parent)
    secure_dir(base)
    directory = base/run_id
    directory.mkdir(mode=0o700, exist_ok=False)
    return directory


def seal(directory, model_file=ROOT/'model-upstream.yml'):
    directory = secure_dir(directory)
    model_file = Path(model_file)
    dc = yaml.safe_load(model_file.read_text())['dc']
    expected = {name+'.cfg' for name in dc['nodes']}
    require({p.name for p in directory.iterdir()} == expected, 'capture must contain exactly one file per modeled node')
    entries = {}
    for name in sorted(dc['nodes']):
        path = directory/(name+'.cfg')
        require(path.is_file() and not path.is_symlink(), 'backup must be a regular file')
        path.chmod(0o600)
        data = local_bytes(path)
        config_ok(data.decode(), intent(dc, name))
        entries[name] = {'filename': path.name, 'sha256': hashlib.sha256(data).hexdigest(), 'bytes': len(data)}
    index = {'schema': 1, 'run_id': directory.name, 'model_sha256': hashlib.sha256(model_file.read_bytes()).hexdigest(),
             'source_sha256': source_digest(ROOT), 'nodes': entries}
    atomic_json(directory/'manifest.json', index)
    return index


def verify(directory, model_file=ROOT/'model-upstream.yml'):
    directory = Path(directory)
    require(directory.is_dir() and not directory.is_symlink(), 'invalid backup directory')
    require((directory/'manifest.json').is_file() and not (directory/'manifest.json').is_symlink(), 'invalid backup index')
    require(directory.stat().st_mode & 0o777 == 0o700, 'backup directory must be private')
    index = json.loads(local_bytes(directory/'manifest.json', INDEX_LIMIT))
    model_file = Path(model_file)
    dc = yaml.safe_load(model_file.read_text())['dc']
    require(index.get('schema') == 1 and index.get('run_id') == directory.name, 'backup identity mismatch')
    require(index.get('model_sha256') == hashlib.sha256(model_file.read_bytes()).hexdigest(), 'backup model revision mismatch')
    require(set(index.get('nodes', {})) == set(dc['nodes']), 'incomplete backup index')
    require({p.name for p in directory.iterdir()} == {'manifest.json'} | {n+'.cfg' for n in dc['nodes']}, 'unexpected/missing backup files')
    for name, entry in index['nodes'].items():
        require(entry['filename'] == name+'.cfg', 'unsafe backup filename')
        path = directory/entry['filename']
        require(path.is_file() and not path.is_symlink(), 'backup must be a regular file')
        data = local_bytes(path)
        require(len(data) == entry['bytes'] and hashlib.sha256(data).hexdigest() == entry['sha256'], 'backup checksum mismatch')
        config_ok(data.decode(), intent(dc, name))
    return index


def request(base, run_id, filename, token, method='GET', body=None, cafile=None, limit=None):
    url = urllib.parse.urlsplit(base)
    require(url.scheme == 'https' and bool(url.netloc) and not url.query and not url.fragment and not url.username, 'backup base must be HTTPS without credentials/query')
    headers = {'Authorization': 'Bearer '+token, 'Content-Type': 'application/octet-stream'}
    if method == 'PUT':
        headers['If-None-Match'] = '*'
    target = base.rstrip('/')+'/'+urllib.parse.quote(run_id, safe='')+'/'+urllib.parse.quote(filename, safe='')
    req = urllib.request.Request(target, data=body, headers=headers, method=method)
    # Redirects could disclose the bearer token to another authority.
    class NoRedirect(urllib.request.HTTPRedirectHandler):
        def redirect_request(self, *args, **kwargs):
            raise ValueError('backup redirects are not allowed')
    opener = urllib.request.build_opener(NoRedirect(), urllib.request.HTTPSHandler(context=ssl.create_default_context(cafile=cafile)))
    limit = (INDEX_LIMIT if filename == 'manifest.json' else CONFIG_LIMIT) if limit is None else limit
    if method == 'PUT':
        limit = 4096
    with deadline(), opener.open(req, timeout=30) as response:
        require(response.status in ([200, 201, 204] if method == 'PUT' else [200]), 'backup HTTP status')
        length = response.headers.get('Content-Length')
        if length is not None:
            require(length.isdecimal() and int(length) <= limit, 'backup Content-Length exceeds byte limit')
        return bounded_read(response, limit)


def export(directory, base, token, cafile=None):
    with deadline():
        return _export(directory, base, token, cafile)


def _export(directory, base, token, cafile=None):
    directory = Path(directory)
    index = verify(directory)
    require(bool(token), 'backup token required')
    # Publish the complete index last; no index means an incomplete capture.
    filenames = [name+'.cfg' for name in sorted(index['nodes'])] + ['manifest.json']
    total = 0
    for filename in filenames:
        data = local_bytes(directory/filename, INDEX_LIMIT if filename == 'manifest.json' else CONFIG_LIMIT)
        total += len(data)
        require(total <= CAPTURE_LIMIT, 'backup capture exceeds byte limit')
        request(base, directory.name, filename, token, 'PUT', data, cafile)
        observed = request(base, directory.name, filename, token, cafile=cafile)
        require(observed == data, 'remote backup bytes differ')
    return index


def download(base_dir, run_id, base_url, token, cafile=None):
    with deadline():
        return _download(base_dir, run_id, base_url, token, cafile)


def _download(base_dir, run_id, base_url, token, cafile=None):
    directory = reserve(base_dir, run_id)
    index_bytes = request(base_url, run_id, 'manifest.json', token, cafile=cafile)
    index = json.loads(index_bytes)
    dc = yaml.safe_load((ROOT/'model-upstream.yml').read_text())['dc']
    require(set(index.get('nodes', {})) == set(dc['nodes']), 'remote index node set mismatch')
    require(index.get('schema') == 1 and index.get('run_id') == run_id, 'remote capture identity mismatch')
    require(index.get('model_sha256') == hashlib.sha256((ROOT/'model-upstream.yml').read_bytes()).hexdigest(), 'remote model mismatch')
    total = len(index_bytes)
    for name in sorted(dc['nodes']):
        filename = name+'.cfg'
        require(index['nodes'][name]['filename'] == filename, 'unsafe remote filename')
        entry = index['nodes'][name]
        expected = entry.get('bytes')
        require(type(expected) is int and 0 <= expected <= CONFIG_LIMIT, 'invalid remote size')
        require(total + expected <= CAPTURE_LIMIT, 'backup capture exceeds byte limit')
        data = request(base_url, run_id, filename, token, cafile=cafile, limit=expected)
        require(len(data) == expected and hashlib.sha256(data).hexdigest() == entry['sha256'], 'remote backup checksum mismatch')
        config_ok(data.decode(), intent(dc, name))
        total += len(data)
        with (directory/filename).open('xb') as stream:
            os.fchmod(stream.fileno(), 0o600)
            stream.write(data)
    atomic_json(directory/'manifest.json', index)
    return verify(directory)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('action', choices=['reserve', 'seal', 'verify', 'export', 'download'])
    parser.add_argument('--directory')
    parser.add_argument('--run-id')
    parser.add_argument('--url')
    parser.add_argument('--cafile')
    args = parser.parse_args()
    base = ROOT/'.state/backups'
    token = os.environ.get('EX457_BACKUP_TOKEN', '')
    if args.action == 'reserve':
        reserve(base, args.run_id)
    elif args.action == 'download':
        require(bool(token), 'backup token required')
        download(base, args.run_id, args.url, token, args.cafile)
    elif args.action == 'export':
        export(args.directory, args.url, token, args.cafile)
    else:
        globals()[args.action](args.directory)
    print('Backup '+args.action+' completed')


if __name__ == '__main__':
    main()
