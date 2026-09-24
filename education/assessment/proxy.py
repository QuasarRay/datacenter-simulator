"""Unprivileged Rustlings grader transport; it has no Incus or evidence access."""
import json
import socket
import sys

def request(path,payload):
    with socket.socket(socket.AF_UNIX,socket.SOCK_STREAM) as client:
        client.settimeout(420)
        client.connect(path)
        client.sendall(json.dumps(payload).encode()+b'\n')
        result=b''
        while b'\n' not in result and len(result)<=65536:
            chunk=client.recv(4096)
            if not chunk:break
            result+=chunk
        if len(result)>65536:raise ValueError('grader response too large')
        return json.loads(result)

if __name__=='__main__':
    if len(sys.argv)!=2:raise SystemExit('provide the assigned assessment socket path')
    data=sys.stdin.buffer.read(32769)
    if len(data)>32768:raise ValueError('request too large')
    print(json.dumps(request(sys.argv[1],json.loads(data))))
