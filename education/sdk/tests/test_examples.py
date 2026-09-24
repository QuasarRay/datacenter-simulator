from pathlib import Path
import unittest
from mcpyrate.compiler import run
from ncp_lab import Envelope, commissioning, headroom_bytes, pkey_allows

ROOT=Path(__file__).resolve().parents[2]

class TeachingContracts(unittest.TestCase):
    def test_all_twenty_authored_examples_execute(self):
        paths=sorted((ROOT/'ncp-metablueprint/examples').glob('C*.py'))
        self.assertEqual(len(paths),20)
        for path in paths:
            with self.subTest(course=path.stem):run(path.read_text())

    def test_macro_evaluates_once_and_survives_optimization(self):
        source='''from ncp_lab.macros import macros, require
count=0
def observe():
    global count
    count+=1
    return True
require[observe()]
'''
        self.assertEqual(run(source,optimize=2).count,1)
        with self.assertRaises(ValueError):
            run('from ncp_lab.macros import macros, require\nrequire[False]\n',optimize=2)

    def test_boundary_units_and_membership(self):
        with self.assertRaises(ValueError):commissioning(Envelope(1,2000,1000,3000))
        with self.assertRaises(ValueError):headroom_bytes(100,0,100)
        with self.assertRaises(ValueError):pkey_allows(65536,1)
        self.assertFalse(pkey_allows(1,1))
        self.assertTrue(pkey_allows(0x8001,1))
        self.assertFalse(pkey_allows(0x8001,0x8002))
