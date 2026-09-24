"""Persistent typed-by-operation client for the existing Rust simulator JSON API."""
import json
import os
from pathlib import Path
import selectors
import subprocess
import tempfile
import time

class Twin:
    def __init__(self, executable, timeout=15):
        executable = Path(executable).resolve(strict=True)
        if not 0 < timeout <= 300:
            raise ValueError('timeout must be in (0,300] seconds')
        self.timeout = timeout
        self.errors = tempfile.TemporaryFile()
        self.process = subprocess.Popen([str(executable),'json'],stdin=subprocess.PIPE,
                                        stdout=subprocess.PIPE,stderr=self.errors)
        os.set_blocking(self.process.stdout.fileno(),False)
        self.pending = b''
        self.selector = selectors.DefaultSelector()
        self.selector.register(self.process.stdout,selectors.EVENT_READ)

    def call(self, operation, **fields):
        message = json.dumps({'operation':operation,**fields}).encode()+b'\n'
        if len(message) > 65536:
            raise ValueError('teaching request exceeds 64 KiB')
        self.process.stdin.write(message)
        self.process.stdin.flush()
        deadline = time.monotonic()+self.timeout
        while b'\n' not in self.pending:
            left=deadline-time.monotonic()
            if left <= 0 or not self.selector.select(left):
                self.close()
                raise TimeoutError('simulator reply deadline expired')
            chunk=os.read(self.process.stdout.fileno(),65536)
            if not chunk:
                raise RuntimeError('simulator exited without a reply')
            self.pending += chunk
            if len(self.pending)>4*1024*1024:
                self.close()
                raise ValueError('simulator response exceeds 4 MiB')
        line,self.pending=self.pending.split(b'\n',1)
        result=json.loads(line)
        if result['ok'] is not True:
            raise RuntimeError(result['error'])
        return result['result']

    def create(self,name):
        return self.call('create',name=name)['id']

    def node(self,simulation,name,role='host',memory_mib=1024):
        return self.call('create_node',simulation=simulation,node={
            'name':name,'image':'cachyos','role':role,
            'resources':{'cpu':1,'memory':memory_mib,'storage':8}})

    def cable(self,simulation,a,b,a_port='eth1',b_port='eth1',bps=100_000_000_000,ns=1000):
        endpoints=[self.call('create_interface',simulation=simulation,node=node,name=port)
                   for node,port in ((a,a_port),(b,b_port))]
        return self.call('create_link',simulation=simulation,interfaces=endpoints,
                         condition={'bandwidth_bps':bps,'latency_ns':ns,'up':True})

    def close(self):
        if self.process.poll() is None:
            self.process.terminate()
            try: self.process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                self.process.kill(); self.process.wait()
        self.selector.close()
        self.process.stdin.close()
        self.process.stdout.close()
        self.errors.close()

    def __enter__(self): return self
    def __exit__(self,*_): self.close()
