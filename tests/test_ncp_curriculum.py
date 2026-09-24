import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[1]
def module(name):
    spec=importlib.util.spec_from_file_location(name,ROOT/'tools'/f'{name}.py')
    m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m

class CurriculumTests(unittest.TestCase):
    def test_complete_chain(self):
        self.assertEqual(module('check_curriculum').validate(ROOT/'education/ncp-metablueprint'),[])

    def test_bank_is_exact_compilation_of_source_obligations(self):
        bp=json.loads((ROOT/'education/ncp-metablueprint/blueprint.json').read_text())
        self.assertEqual(module('build_assessment_bank').build(bp),json.loads((ROOT/'education/assessment/bank.json').read_text()))

    def test_student_export_omits_trainer_material_and_cannot_overwrite(self):
        with tempfile.TemporaryDirectory() as temp:
            target=Path(temp)/'learner';export=module('export_ncp_workshop').export
            export(target)
            self.assertEqual(len(list((target/'controllers').glob('*.py'))),40)
            self.assertEqual(len(list((target/'exercises').glob('*.rs'))),40)
            self.assertFalse(any('bank' in p.name or 'grader' in p.name or 'evidence' in p.name for p in target.rglob('*')))
            with self.assertRaises(FileExistsError):export(target)
