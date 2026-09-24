"""Mutation tests ensure the ledger cannot quietly shed source obligations."""
import copy
import importlib.util
import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('check_metablueprint', ROOT/'tools/check_metablueprint.py')
checker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checker)

class Coverage(unittest.TestCase):
    def setUp(self):
        self.data = json.loads((ROOT/'education/ncp-metablueprint/blueprint.json').read_text())

    def test_registry(self):
        self.assertEqual(checker.validate(self.data), [])

    def test_removing_any_objective_is_rejected(self):
        for index in range(len(self.data['objectives'])):
            mutated = copy.deepcopy(self.data)
            mutated['objectives'].pop(index)
            self.assertTrue(checker.validate(mutated), index)

    def test_each_unit_needs_all_three_domains(self):
        exams = {o['id']: o['exam'] for o in self.data['objectives']}
        for index in range(len(self.data['units'])):
            for missing in ['AIO', 'AIN', 'AII']:
                mutated = copy.deepcopy(self.data)
                u = mutated['units'][index]
                u['objectives'] = [i for i in u['objectives'] if exams[i] != missing]
                self.assertTrue(checker.validate(mutated), (index, missing))

if __name__ == '__main__':
    unittest.main()
