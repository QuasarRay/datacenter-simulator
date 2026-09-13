"""Immutable private capture sets with a durable checksum index and TLS export."""
import argparse
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
        data = path.read_bytes()
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
    index = json.loads((directory/'manifest.json').read_text())
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
        data = path.read_bytes()
        require(len(data) == entry['bytes'] and hashlib.sha256(data).hexdigest() == entry['sha256'], 'backup checksum mismatch')
        config_ok(data.decode(), intent(dc, name))
    return index


def request(base, run_id, filename, token, method='GET', body=None, cafile=None):
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
    with opener.open(req, timeout=30) as response:
        require(response.status in ([200, 201, 204] if method == 'PUT' else [200]), 'backup HTTP status')
        return response.read()


def export(directory, base, token, cafile=None):
    directory = Path(directory)
    index = verify(directory)
    require(bool(token), 'backup token required')
    # Publish the complete index last; no index means an incomplete capture.
    filenames = [name+'.cfg' for name in sorted(index['nodes'])] + ['manifest.json']
    for filename in filenames:
        data = (directory/filename).read_bytes()
        request(base, directory.name, filename, token, 'PUT', data, cafile)
        observed = request(base, directory.name, filename, token, cafile=cafile)
        require(observed == data, 'remote backup bytes differ')
    return index


def download(base_dir, run_id, base_url, token, cafile=None):
    directory = reserve(base_dir, run_id)
    index_bytes = request(base_url, run_id, 'manifest.json', token, cafile=cafile)
    index = json.loads(index_bytes)
    dc = yaml.safe_load((ROOT/'model-upstream.yml').read_text())['dc']
    require(set(index.get('nodes', {})) == set(dc['nodes']), 'remote index node set mismatch')
    require(index.get('run_id') == run_id, 'remote capture identity mismatch')
    for name in sorted(dc['nodes']):
        filename = name+'.cfg'
        require(index['nodes'][name]['filename'] == filename, 'unsafe remote filename')
        data = request(base_url, run_id, filename, token, cafile=cafile)
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
