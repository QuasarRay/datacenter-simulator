import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
def module(name):
    spec=importlib.util.spec_from_file_location(name, ROOT/'integrations/incus'/f'{name}.py')
    result=importlib.util.module_from_spec(spec);spec.loader.exec_module(result);return result

class NativeBoundaries(unittest.TestCase):
    def test_simulated_workers_are_tainted_and_not_control_planes(self):
        services=module('services')
        nodes=services.worker_objects(1000,'unit')
        self.assertEqual(len({n['metadata']['name'] for n in nodes}),1000)
        self.assertTrue(all(n['spec']['taints'][0]['effect']=='NoSchedule' for n in nodes))
        for count in [0,1001,True,-1]:
            with self.assertRaises(ValueError):services.worker_objects(count,'unit')

    def test_transport_refuses_unowned_or_inactive_instances(self):
        transport=module('transport')
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)/'journal.json'
            p.write_text(json.dumps({'phase':'active','config':{},'plan':{'owner':'expected','nodes':[{'name':'node'}]}}))
            client=transport.Incus(p)
            with self.assertRaises(ValueError):client.checked('outside')
            client.request=lambda *a,**k:{'metadata':{'status_code':103,'config':{'user.ncp.owner':'foreign'}}}
            with self.assertRaises(ValueError):client.checked('node')
            p.write_text(p.read_text().replace('active','creating-instances'))
            with self.assertRaises(ValueError):transport.Incus(p)

    def test_multiline_service_injection_is_rejected(self):
        services=module('services')
        with self.assertRaises(ValueError):
            services.service('bad','/bin/true','x\nExecStart=/bin/sh','ncp','/tmp','missing')

if __name__=='__main__':unittest.main()
