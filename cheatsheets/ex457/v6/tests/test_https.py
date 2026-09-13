"""Real loopback HTTPS I/O; the object service is a disposable test implementation."""
from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
from pathlib import Path
import ssl
import subprocess
import threading
import urllib.error
import pytest
import backup_store as bs
from test_state import capture


def test_https_conditional_storage_tls_and_redirects(tmp_path):
    cert=tmp_path/'cert.pem';key=tmp_path/'key.pem'
    subprocess.run(['openssl','req','-x509','-newkey','rsa:2048','-nodes','-keyout',str(key),'-out',str(cert),'-days','1','-subj','/CN=localhost','-addext','subjectAltName=DNS:localhost'],check=True,capture_output=True)
    objects={};headers=[]
    class Handler(BaseHTTPRequestHandler):
        def log_message(self,*args):pass
        def do_PUT(self):
            headers.append(dict(self.headers))
            if self.headers.get('Authorization')!='Bearer fixture':self.send_error(403);return
            if self.headers.get('If-None-Match')!='*':self.send_error(428);return
            if self.path in objects:self.send_error(412);return
            objects[self.path]=self.rfile.read(int(self.headers['Content-Length']))
            self.send_response(201);self.end_headers()
        def do_GET(self):
            if self.path.startswith('/redirect/'):
                self.send_response(302);self.send_header('Location','https://untrusted.invalid/');self.end_headers();return
            if self.headers.get('Authorization')!='Bearer fixture':self.send_error(403);return
            if self.path not in objects:self.send_error(404);return
            self.send_response(200);self.end_headers();self.wfile.write(objects[self.path])
    server=ThreadingHTTPServer(('127.0.0.1',0),Handler)
    ctx=ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER);ctx.load_cert_chain(cert,key);server.socket=ctx.wrap_socket(server.socket,server_side=True)
    thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start()
    url=f'https://localhost:{server.server_port}'
    try:
        folder=capture(tmp_path/'source/backups','https')
        bs.export(folder,url,'fixture',str(cert))
        assert len(objects)==5 and all(h['If-None-Match']=='*' for h in headers)
        with pytest.raises(urllib.error.HTTPError) as error:bs.export(folder,url,'fixture',str(cert))
        assert error.value.code==412
        with pytest.raises(urllib.error.URLError):bs.request(url,'https','manifest.json','fixture')
        with pytest.raises(ValueError,match='redirect'):bs.request(url+'/redirect','https','manifest.json','fixture',cafile=str(cert))
        bs.download(tmp_path/'download/backups','https',url,'fixture',str(cert))
        with pytest.raises(urllib.error.HTTPError) as error:bs.request(url,'https','manifest.json','wrong-token',cafile=str(cert))
        assert error.value.code==403
    finally:server.shutdown();server.server_close();thread.join(timeout=5)
