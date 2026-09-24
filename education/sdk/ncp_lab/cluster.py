"""kr8s against a supplied Rusternetes kubeconfig, with no implicit local context."""
from pathlib import Path
import kr8s
from kr8s.objects import Node

class Cluster:
    def __init__(self,kubeconfig):
        self.api=kr8s.api(kubeconfig=str(Path(kubeconfig).resolve(strict=True)))

    def synthetic_workers(self,specifications):
        result=[]
        for spec in specifications:
            if spec['metadata'].get('labels',{}).get('ncp.simulated')!='true':
                raise ValueError('synthetic worker label is mandatory')
            if not any(t.get('key')=='ncp.simulated' and t.get('effect')=='NoSchedule'
                       for t in spec['spec'].get('taints',[])):
                raise ValueError('synthetic worker isolation taint is mandatory')
            node=Node(spec,api=self.api)
            if node.exists():
                node.refresh()
                # Never silently adopt a real node with a matching name.
                if node.labels.get('ncp.simulated')!='true':
                    raise ValueError('refusing to adopt a real worker')
            else: node.create()
            result.append(node.to_dict())
        return result

    def inventory(self):
        return [node.to_dict() for node in self.api.get('nodes')]
