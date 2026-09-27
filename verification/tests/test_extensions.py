import copy
from dataclasses import replace
import json
from pathlib import Path
import unittest

from education.authoring.dcit import ROWS, UNITS
from verification.dsl import ContractError
from verification.extensions import compile_dcit
from verification.materials import Obligations

ROOT = Path(__file__).resolve().parents[2]


class ExtensionBoundary(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.bp = json.loads((ROOT / 'education/ncp-metablueprint/blueprint.json').read_text())
        cls.lock = json.loads((ROOT / 'verification/dcit-source-obligations.json').read_text())

    def compile(self, rows=ROWS, bp=None, lock=None):
        return compile_dcit(bp or self.bp, lock or self.lock, rows, UNITS, Obligations())

    def test_every_leaf_is_required_even_when_another_leaf_has_similar_subject_matter(self):
        for i in range(len(ROWS)):
            with self.subTest(locator=ROWS[i].locator), self.assertRaises(ContractError):
                self.compile(ROWS[:i] + ROWS[i + 1:])
        self.assertEqual(self.compile()['source_leaves'], 52)

    def test_duplicate_unknown_and_unmapped_declarations_are_rejected(self):
        for changed in (ROWS[1], replace(ROWS[0], locator='1.99'), replace(ROWS[0], core='U99'),
                        replace(ROWS[0], unit=True), replace(ROWS[0], unit=13), replace(ROWS[0], boundary='')):
            with self.subTest(changed=changed), self.assertRaises(ContractError): self.compile([changed, *ROWS[1:]])

    def test_each_adaptation_requires_all_three_nvidia_domains(self):
        for domain in ('AIO', 'AIN', 'AII'):
            broken = copy.deepcopy(self.bp)
            removed = {o['id'] for o in broken['objectives'] if o['exam'] == domain}
            for unit in broken['units']:
                unit['objectives'] = [identity for identity in unit['objectives'] if identity not in removed]
            with self.subTest(domain=domain), self.assertRaises(ContractError): self.compile(bp=broken)

    def test_source_schema_and_empty_concepts_cannot_silently_pass(self):
        for update in ('version', 'duplicate', 'empty'):
            broken = copy.deepcopy(self.lock)
            if update == 'version': broken['source']['version'] = 'unknown'
            if update == 'duplicate': broken['objectives'][0] = broken['objectives'][1]
            if update == 'empty': broken['objectives'][0]['concepts'] = []
            with self.subTest(update=update), self.assertRaises(ContractError): self.compile(lock=broken)

    def test_a_complete_registry_is_never_a_live_grade(self):
        result = self.compile()
        self.assertIs(result['live_mastery_enabled'], False)
        self.assertTrue(all(row['status'] == 'specified-not-qualified' for row in result['objectives']))


if __name__ == '__main__': unittest.main()
