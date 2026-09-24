import copy
from datetime import datetime, timezone
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

from integrations.incus.fabric import Fabric, native_fabric

ROOT=Path(__file__).resolve().parents[1]

class NativeFabric(unittest.TestCase):
    def test_snapshot_compiles_seals_and_rejects_overcommit(self):
        binary=ROOT/'simulator/target/debug/datacenter-simulator'
        if not binary.exists():self.skipTest('compile the simulator with incus,netbox first')
        snapshot=json.loads((ROOT/'integrations/netbox/snapshot.example.json').read_text())
        snapshot['fetched_at']=datetime.now(timezone.utc).isoformat()
        for port in snapshot['interfaces']:
            if not port['mgmt_only']:port['type']['value']='1000base-t'
        config=json.loads((ROOT/'integrations/netbox/native.example.json').read_text())
        config['netbox'].update(api_url=snapshot['api_url'],allow_http=True,
            roles={'gpu-server':'host','ib-switch':'switch'},planned_links_up=True)
        config['image_fingerprint']='0'*64 # planning fixture; never sent to a daemon
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp)/'snapshot.json';path.write_text(json.dumps(snapshot))
            @native_fabric
            def lesson(memory):return config | {'max_memory_mib':memory}
            plan=lesson(3072,binary=binary,destination=Path(tmp)/'plan',snapshot=path)
            self.assertEqual(plan.verify()['memory_mib'],3072)
            generated=json.loads((plan.directory/'provisioning-vars.json').read_text())
            self.assertEqual(len(generated['ncp_addresses']),4)
            self.assertEqual(generated['ncp_forwarding'],{'nb1':False,'nb2':False,'nb3':True})
            failed=Path(tmp)/'overcommitted'
            with self.assertRaises(subprocess.CalledProcessError):
                lesson(3071,binary=binary,destination=failed,snapshot=path)
            self.assertFalse(failed.exists())
            (plan.directory/'provisioning-vars.json').write_text('{}')
            with self.assertRaisesRegex(ValueError,'changed'):plan.verify()

if __name__=='__main__':unittest.main()
