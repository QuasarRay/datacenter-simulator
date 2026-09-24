"""Live qualification on a disposable host; no simulated success receipts.

The record is a capability report, not a station grade or conformance certificate.
Certificates, signing material and bearer tokens stay out of uploaded artifacts.
"""
import base64
import hashlib
import hmac
import json
import os
from pathlib import Path
import secrets
import ssl
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request

def run(*args, **kwargs):
    return subprocess.run(list(map(str,args)), check=True, capture_output=True, timeout=60, **kwargs)

def token(secret, subject):
    def b64(value):
        return base64.urlsafe_b64encode(json.dumps(value,separators=(',',':')).encode()).rstrip(b'=')
    now=int(time.time())
    parts=[b64({'alg':'HS256','typ':'JWT'}), b64({'sub':subject,'namespace':'ncp','uid':subject,
        'iat':now,'exp':now+900,'iss':'rusternetes-api-server','aud':['rusternetes']})]
    signature=hmac.new(secret.encode(),b'.'.join(parts),hashlib.sha256).digest()
    return b'.'.join(parts+[base64.urlsafe_b64encode(signature).rstrip(b'=')]).decode()

def qualify(output, kwok_source):
    output=Path(output).resolve();assets=output/'assets';children=[];streams=[];tests=[]
    with tempfile.TemporaryDirectory(prefix='ncp-control-') as private:
        private=Path(private);os.chmod(private,0o700)
        def start(name, *args):
            stream=(output/(name+'.log')).open('w');streams.append(stream)
            child=subprocess.Popen(list(map(str,args)),stdout=stream,stderr=subprocess.STDOUT,cwd=private)
            children.append(child);return child
        def record(name):tests.append({'test':name,'passed':True})
        def eventually(predicate, seconds=45):
            deadline=time.monotonic()+seconds
            while time.monotonic()<deadline:
                try:
                    value=predicate()
                    if value:return value
                except (OSError,urllib.error.URLError):pass
                if any(p.poll() is not None for p in children):
                    raise RuntimeError('a native component exited; inspect its transcript')
                time.sleep(0.2)
            raise TimeoutError('native postcondition deadline exceeded')
        try:
            # Explicit loopback control transport; all tested processes are native.
            start('etcd','etcd','--data-dir',private/'etcd','--listen-client-urls','http://127.0.0.1:2379',
                  '--advertise-client-urls','http://127.0.0.1:2379')
            eventually(lambda: urllib.request.urlopen('http://127.0.0.1:2379/health',timeout=1).status==200)
            ca=private/'ca.crt';key=private/'ca.key'
            run('openssl','req','-x509','-newkey','rsa:2048','-nodes','-days','1','-subj','/CN=NCP CI CA','-keyout',key,'-out',ca)
            for name,subject,extension in [('server','/CN=localhost','subjectAltName=IP:127.0.0.1,DNS:localhost\nextendedKeyUsage=serverAuth\n'),
                                           ('client','/CN=ncp-ci','extendedKeyUsage=clientAuth\n')]:
                run('openssl','req','-newkey','rsa:2048','-nodes','-subj',subject,'-keyout',private/(name+'.key'),'-out',private/(name+'.csr'))
                ext=private/(name+'.ext');ext.write_text(extension)
                run('openssl','x509','-req','-in',private/(name+'.csr'),'-CA',ca,'-CAkey',key,'-CAcreateserial',
                    '-days','1','-extfile',ext,'-out',private/(name+'.crt'))
            secret=secrets.token_hex(32);admin=token(secret,'system:admin');worker=token(secret,'system:serviceaccount:ncp:kwok')
            context=ssl.create_default_context(cafile=str(ca));context.load_cert_chain(private/'client.crt',private/'client.key')
            def api(method,path,body=None,bearer=admin,ctx=context):
                headers={'Accept':'application/json','Content-Type':'application/json'}
                if bearer:headers['Authorization']='Bearer '+bearer
                request=urllib.request.Request('https://127.0.0.1:6443'+path,
                    data=None if body is None else json.dumps(body).encode(),method=method,headers=headers)
                try:
                    with urllib.request.urlopen(request,context=ctx,timeout=5) as response:
                        return response.status,json.loads(response.read())
                except urllib.error.HTTPError as error:
                    return error.code,error.read().decode()
            start('api',assets/'rusternetes-api-server','--bind-address','127.0.0.1:6443',
                  '--jwt-secret',secret,'--tls','--tls-cert-file',private/'server.crt',
                  '--tls-key-file',private/'server.key','--client-ca-file',ca)
            eventually(lambda: api('GET','/api/v1/nodes')[0]==200)
            if api('GET','/api/v1/nodes',bearer=None)[0] not in (401,403):raise RuntimeError('missing bearer accepted')
            if api('GET','/api/v1/nodes',bearer='invalid')[0] not in (401,403):raise RuntimeError('invalid bearer accepted')
            if api('GET','/api/v1/nodes',bearer=worker)[0]!=403:raise RuntimeError('unbound worker granted authority')
            no_certificate=ssl.create_default_context(cafile=str(ca))
            try:api('GET','/api/v1/nodes',ctx=no_certificate)
            except (ssl.SSLError,urllib.error.URLError,OSError):pass
            else:raise RuntimeError('TLS accepted a client with no certificate')
            record('tls-client-proof-and-bearer-rbac')
            # Create minimum resource permissions for the selected-node controller.
            role={'apiVersion':'rbac.authorization.k8s.io/v1','kind':'ClusterRole','metadata':{'name':'ncp-kwok'},'rules':[
                {'apiGroups':[''],'resources':['nodes','nodes/status','pods','pods/status','events'],
                 'verbs':['get','list','watch','patch','update','create']},
                {'apiGroups':['coordination.k8s.io'],'resources':['leases'],'verbs':['get','list','watch','create','update','patch']}]}
            binding={'apiVersion':'rbac.authorization.k8s.io/v1','kind':'ClusterRoleBinding','metadata':{'name':'ncp-kwok'},
                'subjects':[{'kind':'User','apiGroup':'rbac.authorization.k8s.io','name':'system:serviceaccount:ncp:kwok'}],
                'roleRef':{'kind':'ClusterRole','apiGroup':'rbac.authorization.k8s.io','name':'ncp-kwok'}}
            for resource,body in [('clusterroles',role),('clusterrolebindings',binding)]:
                if api('POST','/apis/rbac.authorization.k8s.io/v1/'+resource,body)[0]!=201:raise RuntimeError('RBAC creation failed')
            for name,selected in [('ncp-selected',True),('ncp-unselected',False)]:
                node={'apiVersion':'v1','kind':'Node','metadata':{'name':name,'labels':{'ncp.simulated':str(selected).lower()}},
                      'spec':{'taints':[{'key':'ncp.simulated','value':'true','effect':'NoSchedule'}]},
                      'status':{'capacity':{'cpu':'2','memory':'1Gi','pods':'10'},'allocatable':{'cpu':'2','memory':'1Gi','pods':'10'}}}
                if api('POST','/api/v1/nodes',node)[0]!=201:raise RuntimeError('node creation failed')
            pod={'apiVersion':'v1','kind':'Pod','metadata':{'name':'ncp-synthetic','namespace':'default'},
                 'spec':{'nodeName':'ncp-selected','containers':[{'name':'placeholder','image':'ncp.invalid/synthetic-only'}]}}
            if api('POST','/api/v1/namespaces/default/pods',pod)[0]!=201:raise RuntimeError('synthetic pod creation failed')
            for suffix in ['log','exec','attach','portforward']:
                code,_=api('GET','/api/v1/namespaces/default/pods/ncp-synthetic/'+suffix)
                if code!=501:raise RuntimeError(f'{suffix} did not report unavailable execution: {code}')
            record('runtime-subresources-refuse-fabricated-execution')
            kubeconfig={'apiVersion':'v1','kind':'Config','current-context':'ncp','contexts':[{'name':'ncp','context':{'cluster':'ncp','user':'kwok'}}],
                'clusters':[{'name':'ncp','cluster':{'server':'https://127.0.0.1:6443','certificate-authority':str(ca)}}],
                'users':[{'name':'kwok','user':{'token':worker,'client-certificate':str(private/'client.crt'),'client-key':str(private/'client.key')}}]}
            config=private/'kwok.kubeconfig';config.write_text(json.dumps(kubeconfig));config.chmod(0o600)
            stage=Path(kwok_source).resolve()/'kustomize/stage/node/fast/node-initialize.yaml'
            start('kwok',assets/'kwok','--kubeconfig',config,'--manage-nodes-with-label-selector','ncp.simulated=true','--config',stage)
            def ready(name):
                code,node=api('GET','/api/v1/nodes/'+name)
                return code==200 and any(c['type']=='Ready' and c['status']=='True' for c in node.get('status',{}).get('conditions',[]))
            eventually(lambda: ready('ncp-selected'))
            if ready('ncp-unselected'):raise RuntimeError('KWOK mutated an unselected node')
            record('kwok-selected-node-ready-unselected-node-unchanged')
        finally:
            for child in reversed(children):
                if child.poll() is None:
                    child.terminate()
                    try:child.wait(timeout=5)
                    except subprocess.TimeoutExpired:child.kill();child.wait(timeout=5)
            for stream in streams:stream.close()
            report={'tier':'api-emulated','tests':tests,'required_tests':4,'complete':len(tests)==4,
                    'hardware_execution':False,'kubernetes_conformance':False,
                    'assets':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in assets.iterdir() if p.is_file()}}
            (output/'qualification.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))

if __name__=='__main__':
    if len(sys.argv)!=3:raise SystemExit('requires output directory and pinned KWOK source')
    qualify(*sys.argv[1:])
