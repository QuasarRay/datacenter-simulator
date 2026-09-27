"""Bounded native UDP probe, executed verbatim inside an owned Linux guest.

This payload checks a CPU packet path. It does not execute a GPU or a pod.
There is no daemon, shell expansion, downloaded program or background process.
"""
import errno
import hashlib
import ipaddress
import json
from pathlib import Path
import re
import socket
import sys
import time


def probe(mode, config):
    if mode not in {'inspect', 'server', 'client'}:
        raise ValueError('unknown probe mode')
    if set(config) != {'interface', 'address', 'peer', 'nonce', 'port'}:
        raise ValueError('probe schema')
    if not re.fullmatch(r'[a-zA-Z0-9_-]{1,15}', config['interface']):
        raise ValueError('invalid interface')
    for field in ('address', 'peer'): ipaddress.IPv4Address(config[field])
    if not re.fullmatch('[a-f0-9]{64}', config['nonce']): raise ValueError('invalid nonce')
    if type(config['port']) is not int or not 20000 <= config['port'] < 30000:
        raise ValueError('probe port budget')
    result = {'schema': 'ncp-path-probe-v1', 'role': mode, 'nonce': config['nonce'],
              'address': config['address'], 'interface': config['interface']}
    if mode == 'inspect':
        base = Path('/sys/class/net') / config['interface'] / 'statistics'
        return result | {'tx': int((base / 'tx_packets').read_text()),
                         'rx': int((base / 'rx_packets').read_text())}
    payload = bytes.fromhex(config['nonce'])
    response = hashlib.sha256(payload).digest()
    deadline = time.monotonic() + (4 if mode == 'server' else 3)
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as channel:
        channel.setsockopt(socket.SOL_SOCKET, socket.SO_BINDTODEVICE, config['interface'].encode() + b'\0')
        channel.bind((config['address'], config['port'] if mode == 'server' else 0))
        channel.settimeout(0.2)
        if mode == 'server':
            result.update(listening=True, received=False)
            while time.monotonic() < deadline:
                try: data, peer = channel.recvfrom(1024)
                except socket.timeout: continue
                if peer[0] == config['peer'] and data == payload:
                    channel.sendto(response, peer)
                    result['received'] = True
                    break
        else:
            result.update(delivered=False)
            while time.monotonic() < deadline:
                try:
                    channel.sendto(payload, (config['peer'], config['port']))
                    data, peer = channel.recvfrom(1024)
                    if peer == (config['peer'], config['port']) and data == response:
                        result['delivered'] = True
                        break
                except socket.timeout: continue
                except OSError as error:
                    if error.errno not in {errno.EHOSTUNREACH, errno.ENETUNREACH, errno.ECONNREFUSED, errno.ETIMEDOUT}:
                        raise
                    time.sleep(0.05)
    return result


if __name__ == '__main__':
    if len(sys.argv) != 3: raise SystemExit('requires mode and bounded probe configuration')
    print(json.dumps(probe(sys.argv[1], json.loads(sys.argv[2])), sort_keys=True))
