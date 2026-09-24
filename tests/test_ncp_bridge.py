import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('ncp_bridge',ROOT/'education/assessment/bridge.py')
bridge=importlib.util.module_from_spec(spec);spec.loader.exec_module(bridge)

class EvidenceBridgeTests(unittest.TestCase):
    def test_request_schema_has_no_paths_or_commands(self):
        q={'protocol':'ncp-grade-v1','name':'e01a','exercise':0,'attempt':1}
        self.assertEqual(bridge.validate_request(q),q)
        for patch in [{'name':'../x'},{'exercise':40},{'attempt':0},{'attempt':2**64},
                      {'command':'anything'},{'attempt':True},{'exercise':False}]:
            with self.assertRaises(ValueError):bridge.validate_request(q|patch)

    def test_unqualified_collector_cannot_execute_or_award(self):
        with tempfile.TemporaryDirectory() as tmp:
            profile=Path(tmp)/'profile.json'
            profile.write_text(json.dumps({'qualification':'unqualified','evidence':tmp,
                'python':'/no/python','collector':'/no/collector','grader':'/no/grader','state':'/no/state'}))
            instance=bridge.Bridge(profile)
            q={'protocol':'ncp-grade-v1','name':'e01a','exercise':0,'attempt':1}
            with self.assertRaisesRegex(RuntimeError,'not live-qualified'):instance.assess(q)
            digest=instance.artifact(q,'test-facet',0,'real collector bytes')
            artifact=json.loads((Path(tmp)/'objects'/digest).read_text())
            self.assertEqual(artifact['request'],q)
            self.assertEqual(artifact['scope'],'test-facet')
            self.assertNotEqual(digest,instance.artifact(q|{'attempt':2},'test-facet',0,'real collector bytes'))

    def test_shared_evidence_directory_is_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            os.chmod(tmp,0o755);profile=Path(tmp)/'profile.json'
            profile.write_text(json.dumps({'evidence':tmp}))
            with self.assertRaisesRegex(ValueError,'0700'):bridge.Bridge(profile)
