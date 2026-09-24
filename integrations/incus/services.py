"""Generate native service declarations; fail before emitting unpinned binaries."""
import hashlib
from pathlib import Path

def service(name, executable, arguments, user, directory, local_binary):
    fields = (name, executable, arguments, user, directory)
    if any('\n' in s or '\r' in s for s in fields):
        raise ValueError('multiline systemd fields are forbidden')
    return {'name':'ncp-'+name,'executable':executable,'arguments':arguments,
            'user':user,'directory':directory,
            'sha256':hashlib.sha256(Path(local_binary).read_bytes()).hexdigest()}

def control_plane(assets):
    """Single local etcd for an initial lab; HA needs a separate qualified profile."""
    assets = Path(assets)
    def make(name, binary, args):
        return service(name,'/opt/ncp/bin/'+binary,args,'ncp','/var/lib/ncp',assets/binary)
    etcd='--storage-backend etcd --etcd-servers http://127.0.0.1:2379'
    return [
      make('etcd','etcd','--data-dir /var/lib/ncp/etcd --listen-client-urls http://127.0.0.1:2379 --advertise-client-urls http://127.0.0.1:2379'),
      make('api','rusternetes-api-server',etcd+' --bind-address 0.0.0.0:6443 --tls --tls-cert-file /etc/ncp/pki/server.crt --tls-key-file /etc/ncp/pki/server.key --client-ca-file /etc/ncp/pki/ca.crt --jwt-secret ${NCP_JWT_SECRET}'),
      make('scheduler','rusternetes-scheduler',etcd),
      make('controllers','rusternetes-controller-manager',etcd),
    ]

def workers(assets):
    return [service('kwok','/opt/ncp/bin/kwok',
        '--kubeconfig /etc/ncp/kwok.kubeconfig --manage-nodes-with-label-selector ncp.simulated=true --config /etc/ncp/stages.yaml',
        'ncp','/var/lib/ncp',Path(assets)/'kwok')]

def worker_objects(count, cluster, gpu_per_node=8):
    """Logical workers, not real control-plane nodes or executing GPU pods."""
    if type(count) is not int or not 1 <= count <= 1000 or type(gpu_per_node) is not int or not 0 <= gpu_per_node <= 16:
        raise ValueError('worker budget exceeded')
    if not cluster or not cluster.replace('-','').isalnum() or len(cluster)>30:
        raise ValueError('invalid cluster identifier')
    return [{'apiVersion':'v1','kind':'Node','metadata':{'name':f'{cluster}-w{i:04}',
             'labels':{'ncp.simulated':'true','ncp.cluster':cluster},
             'annotations':{'kwok.x-k8s.io/node':'fake'}},
             'spec':{'taints':[{'key':'ncp.simulated','value':'true','effect':'NoSchedule'}]},
             'status':{'capacity':{'cpu':'16','memory':'64Gi','pods':'110','nvidia.com/gpu':str(gpu_per_node)},
                       'allocatable':{'cpu':'16','memory':'64Gi','pods':'110','nvidia.com/gpu':str(gpu_per_node)}}}
            for i in range(count)]
