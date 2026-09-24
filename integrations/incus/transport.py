"""Trainer-side Incus API transport; never distribute the daemon socket to students."""
import http.client
import json
import socket
import time
from pathlib import Path
from urllib.parse import quote, urlencode

LIMIT = 16 * 1024 * 1024

class UnixHTTP(http.client.HTTPConnection):
    def __init__(self, path):
        super().__init__('localhost', timeout=35)
        self.path = path

    def connect(self):
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.settimeout(self.timeout)
        self.sock.connect(self.path)

class Incus:
    def __init__(self, journal):
        self.journal = json.loads(Path(journal).read_text())
        self.config = self.journal['config']
        self.owner = self.journal['plan']['owner']
        self.nodes = {n['name'] for n in self.journal['plan']['nodes']}
        if self.journal['phase'] != 'active':
            raise ValueError('provisioning requires an active lifecycle journal')

    def request(self, method, path, body=None, headers=None, raw=False):
        if not path.startswith('/1.0/'):
            raise ValueError('unexpected Incus API path')
        connection = UnixHTTP(self.config['socket'])
        path += ('&' if '?' in path else '?') + urlencode({'project': self.config['project']})
        headers = dict(headers or {})
        if isinstance(body, dict):
            body = json.dumps(body).encode()
            headers['Content-Type'] = 'application/json'
        try:
            connection.request(method, path, body, headers)
            response = connection.getresponse()
            data = response.read(LIMIT + 1)
            if len(data) > LIMIT:
                raise RuntimeError('Incus response exceeds limit')
            if response.status >= 400:
                raise RuntimeError(f'Incus HTTP {response.status}: {data[:2048]!r}')
            if raw:
                return data
            result = json.loads(data)
            if result.get('type') == 'error':
                raise RuntimeError(result.get('error'))
            return result
        finally:
            connection.close()

    def checked(self, node):
        if node not in self.nodes:
            raise ValueError('instance is outside the owned inventory')
        path = '/1.0/instances/' + quote(node, safe='')
        current = self.request('GET', path)['metadata']
        if current['config'].get('user.ncp.owner') != self.owner:
            raise ValueError('instance ownership changed')
        if current['status_code'] != 103:
            raise RuntimeError('instance is not running')
        return path

    def execute(self, node, argv, timeout=300):
        import uuid
        path = self.checked(node)
        started = self.request('POST', path + '/exec', {
            'command': list(argv), 'environment': {'PATH':'/usr/local/sbin:/usr/local/bin:/usr/bin:/bin'},
            'wait-for-websocket': False, 'interactive': False, 'record-output': True,
        })
        operation = started['operation'].split('?')[0]
        uuid.UUID(operation.removeprefix('/1.0/operations/'))
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            event = self.request('GET', operation + '/wait?timeout=10')['metadata']
            if event['status_code'] >= 400:
                raise RuntimeError(event.get('err', 'exec operation failed'))
            if event['status_code'] != 200:
                continue
            result = event['metadata']
            streams = []
            for fd in ['1', '2']:
                location = result.get('output', {}).get(fd)
                if location is None:
                    streams.append(b'')
                    continue
                location = location.split('?')[0]
                if not location.startswith(path + '/logs/') or '..' in location:
                    raise ValueError('unexpected output log location')
                streams.append(self.request('GET', location, raw=True))
            return int(result['return']), *streams
        raise TimeoutError('guest execution deadline exceeded; inspect the recorded operation before retry')

    def put(self, node, path, data, mode='0644'):
        target = self.checked(node) + '/files?' + urlencode({'path': str(path)})
        return self.request('POST', target, data, {'X-Incus-type':'file','X-Incus-mode':mode,'X-Incus-uid':'0','X-Incus-gid':'0'})

    def get(self, node, path):
        target = self.checked(node) + '/files?' + urlencode({'path': str(path)})
        return self.request('GET', target, raw=True)
