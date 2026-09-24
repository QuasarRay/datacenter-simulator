"""Trainer-owned collection bridge. No client field selects a command or path.

The site collector is trusted, pinned code. Its product semantics require live
qualification; protocol integrity alone cannot establish hardware truth.
"""
import hashlib
import json
import os
from pathlib import Path
import signal
import socket
import struct
import subprocess
import sys
import tempfile

LIMIT=4*1024*1024
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT))
from integrations.incus.lab import Lab, DEEPOPS

def validate_request(q):
    if set(q)!={'protocol','name','exercise','attempt'} or q['protocol']!='ncp-grade-v1':
        raise ValueError('invalid grading request')
    i=q['exercise'];attempt=q['attempt']
    if type(i) is not int or not 0<=i<40 or type(attempt) is not int or not 0<attempt<2**64:
        raise ValueError('invalid station or attempt')
    if q['name']!=f'e{i//2+1:02}{"a" if i%2==0 else "b"}':
        raise ValueError('station identity mismatch')
    return q

def bounded_run(argv, data, timeout):
    with tempfile.TemporaryFile() as out,tempfile.TemporaryFile() as err:
        process=subprocess.Popen(argv,stdin=subprocess.PIPE,stdout=out,stderr=err,start_new_session=True)
        try:
            process.communicate(json.dumps(data).encode(),timeout=timeout)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid,signal.SIGKILL);process.wait()
            raise TimeoutError('collector exceeded deadline')
        if out.tell()>LIMIT or err.tell()>LIMIT:
            raise ValueError('collector output exceeds limit')
        out.seek(0);err.seek(0)
        if process.returncode:raise RuntimeError('trusted collector failed: '+err.read(2048).decode(errors='replace'))
        return json.loads(out.read())

class Bridge:
    def __init__(self,profile):
        self.profile=json.loads(Path(profile).read_text())
        self.root=Path(self.profile['evidence']).resolve(strict=True)
        if self.root.stat().st_uid!=os.geteuid() or self.root.stat().st_mode&0o077:
            raise ValueError('evidence directory must be owned by trainer with mode 0700')
        for key in ('python','collector','grader','state'):
            if not Path(self.profile[key]).is_absolute():raise ValueError('profile paths must be absolute')

    def artifact(self,q,scope,case,raw):
        if not isinstance(raw,str) or not raw.strip():raise ValueError('empty raw observation')
        envelope={'protocol':'ncp-artifact-v1','request':q,'scope':scope,'case':case,'raw':raw}
        payload=json.dumps(envelope,sort_keys=True).encode()
        if len(payload)>LIMIT:raise ValueError('artifact exceeds limit')
        digest=hashlib.sha256(payload).hexdigest()
        directory=self.root/'objects';directory.mkdir(mode=0o700,exist_ok=True)
        path=directory/digest
        try:
            fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
        except FileExistsError:
            if path.read_bytes()!=payload:raise ValueError('artifact store corruption')
        else:
            with os.fdopen(fd,'wb') as out:out.write(payload);out.flush();os.fsync(out.fileno())
        return digest

    def assess(self,q):
        validate_request(q)
        profile=self.profile
        if profile.get('qualification')!='live-validated':
            raise RuntimeError('site collector is not live-qualified')
        collector=Path(profile['collector'])
        if hashlib.sha256(collector.read_bytes()).hexdigest()!=profile['collector_sha256']:
            raise ValueError('collector source identity changed')
        # Always run real DeepOps. This report alone grants no product facet.
        deepops=Lab(profile['state']).reconcile_hosts(timeout=120)
        payload=bounded_run([profile['python'],str(collector)],
                            {'request':q,'state':profile['state']},180)
        if set(payload)!={'observations','interventions'}:raise ValueError('invalid collector output')
        observations=[]
        for row in payload['observations']:
            if set(row)!={'facet','tier','case','passed','raw'}:raise ValueError('invalid observation')
            row=dict(row);raw=row.pop('raw')
            row['artifact']=self.artifact(q,row['facet'],row['case'],raw);observations.append(row)
        interventions=[]
        for row in payload['interventions']:
            if set(row)!={'domain','failed_when_removed','recovered_when_restored','raw'}:raise ValueError('invalid intervention')
            row=dict(row);raw=row.pop('raw')
            row['artifact']=self.artifact(q,f'intervention/{row["domain"]}',0,raw);interventions.append(row)
        bundle={'request':q,'observations':observations,'interventions':interventions,
                'deepops_revision':DEEPOPS,
                'deepops_artifact':self.artifact(q,'deepops',0,json.dumps(deepops))}
        destination=self.root/q['name'];destination.mkdir(mode=0o700,exist_ok=True)
        path=destination/f'{q["attempt"]}.json'
        # Duplicate attempts do not overwrite earlier evidence.
        with path.open('x') as out:json.dump(bundle,out);out.flush();os.fsync(out.fileno())
        return bounded_run([profile['grader'],str(self.root)],q,30)

def blocked(q):
    return q|{'tier':4,'ops':3,'net':3,'infra':3,'coupled':False,'provenance':False,
              'observed_facets':0,'feedback':'Live assessment unavailable or unqualified; inspect the trainer report. No mastery awarded.'}

def serve(profile):
    bridge=Bridge(profile);socket_path=Path(bridge.profile['socket'])
    if not socket_path.is_absolute():raise ValueError('socket path must be absolute')
    allowed=bridge.profile['learner_host_uid']
    if type(allowed) is not int or allowed==os.geteuid():
        raise ValueError('learner and trainer must have distinct host identities')
    with socket.socket(socket.AF_UNIX,socket.SOCK_STREAM) as server:
        # Never unlink or adopt a preexisting socket. Operator reconciles a crash.
        server.bind(str(socket_path))
        os.chown(socket_path,-1,bridge.profile['learner_host_gid'])
        os.chmod(socket_path,0o660)
        server.listen(2)
        try:
            while True:
                client,_=server.accept()
                with client:
                    client.settimeout(5)
                    _,uid,_=struct.unpack('3i',client.getsockopt(socket.SOL_SOCKET,socket.SO_PEERCRED,12))
                    if uid!=allowed:continue
                    try:
                        data=b''
                        while b'\n' not in data and len(data)<=32768:
                            chunk=client.recv(4096)
                            if not chunk:break
                            data+=chunk
                        if len(data)>32768:raise ValueError('request too large')
                        q=validate_request(json.loads(data))
                    except (ValueError,TypeError,KeyError,OSError):continue
                    try:receipt=bridge.assess(q)
                    except Exception as error:
                        # Details stay on trainer stderr; never return credentials/raw device output.
                        print(f'{q["name"]}: {type(error).__name__}: {error}',file=sys.stderr,flush=True)
                        receipt=blocked(q)
                    try:client.sendall(json.dumps(receipt).encode()+b'\n')
                    except OSError:pass  # A disconnected learner cannot stop the trainer service.
        finally:socket_path.unlink(missing_ok=True)

if __name__=='__main__':
    if len(sys.argv)!=2:raise SystemExit('provide a trainer-owned profile path')
    serve(sys.argv[1])
