import copy
from dataclasses import replace
import json
from pathlib import Path
import shutil
import tempfile
import unittest

from verification.dsl import ContractError
from verification.evidence import Checklist
from verification.materials import LearningItem, Obligations, learning_graph, validate_blueprint, validate_materials
from verification.pipeline import compile_repository, load_module, render_repository, source_paths

ROOT = Path(__file__).resolve().parents[2]


class LearningGraphBoundary(unittest.TestCase):
    def setUp(self):
        self.items = [LearningItem("C1", "course", 1, teaches=("one", "two")),
                      LearningItem("P1", "project", 1, ("C1",), uses=("one", "two"), practices=("one", "two")),
                      LearningItem("E1", "exercise", 1, ("P1",), uses=("one", "two"))]

    def test_transitive_teaching_and_practice_are_required(self):
        learning_graph(self.items, Obligations())
        cases = [replace(self.items[1], uses=("untaught",)), replace(self.items[1], prerequisites=()),
                 replace(self.items[2], prerequisites=("C1",)), replace(self.items[2], uses=("untaught",)),
                 replace(self.items[1], prerequisites=("E1",)), replace(self.items[2], level=3)]
        for changed in cases:
            items = [changed if item.name == changed.name else item for item in self.items]
            with self.subTest(changed=changed), self.assertRaises(ContractError): learning_graph(items, Obligations())

    def test_unrelated_earlier_course_cannot_lend_its_skills(self):
        items = [LearningItem("C1", "course", 1, teaches=("one",)), LearningItem("C2", "course", 2, teaches=("two",)),
                 LearningItem("P1", "project", 1, ("C1",), uses=("two",), practices=("two",))]
        with self.assertRaises(ContractError): learning_graph(items, Obligations())

    def test_more_than_64_skills_are_chunked_without_silent_truncation(self):
        skills = tuple(f"skill{i:03}" for i in range(130))
        items = [LearningItem("C1", "course", 1, teaches=skills),
                 LearningItem("P1", "project", 1, ("C1",), uses=skills, practices=skills),
                 LearningItem("E1", "exercise", 1, ("P1",), uses=skills)]
        facts = Obligations(); learning_graph(items, facts)
        self.assertTrue(any(f.name == "skills/E1/2" for f in facts.facts))
        for missing in (0, 63, 64, 127, 129):
            broken = list(items); broken[1] = replace(items[1], practices=skills[:missing] + skills[missing + 1:])
            with self.subTest(missing=missing), self.assertRaises(ContractError): learning_graph(broken, Obligations())

    def test_duplicate_or_forward_graph_identity_cannot_disappear_in_a_map(self):
        for items in [self.items + [self.items[0]], [replace(self.items[0], prerequisites=("P1",)), *self.items[1:]],
                      [self.items[0], replace(self.items[1], prerequisites=("C1", "C1")), self.items[2]]]:
            with self.assertRaises(ContractError): learning_graph(items, Obligations())


class MaterialBoundary(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.bp = json.loads((ROOT / "education/ncp-metablueprint/blueprint.json").read_text())
        cls.lock = json.loads((ROOT / "verification/source-obligations.json").read_text())
        files, _, _ = compile_repository(ROOT)
        cls.material = {p.removeprefix("education/ncp-metablueprint/"): s for p, s in files.items() if p.startswith("education/ncp-metablueprint/")}
        cls.curriculum = json.loads(cls.material["curriculum.json"])
        cls.bank = json.loads(files["education/assessment/bank.json"])

    def test_every_source_facet_is_locked_against_silent_deletion(self):
        count = 0
        for i, objective in enumerate(self.bp["objectives"]):
            for j in range(len(objective["facets"])):
                broken = copy.deepcopy(self.bp); broken["objectives"][i]["facets"].pop(j)
                with self.subTest(facet=(i, j)), self.assertRaises(ContractError): validate_blueprint(broken, self.lock)
                count += 1
        self.assertGreaterEqual(count, 203)

    def test_authoring_always_compiles_current_bytes_despite_a_same_size_stale_pyc(self):
        import os
        import py_compile
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); path = root / "authored.py"
            path.write_text("value = 1\n")
            before = path.stat()
            py_compile.compile(str(path), doraise=True)
            path.write_text("value = 2\n")
            os.utime(path, ns=(before.st_atime_ns, before.st_mtime_ns))
            self.assertEqual(load_module(root, "authored.py", "current_bytes").value, 2)

    def test_every_station_and_learning_stage_requires_its_exact_facets(self):
        for index in range(40):
            bank = copy.deepcopy(self.bank); bank[index]["facets"].pop()
            with self.subTest(station=index), self.assertRaises(ContractError):
                validate_materials(self.bp, self.curriculum, bank, self.material, Obligations())
        for index in range(20):
            for kind in ("course", "project"):
                curriculum = copy.deepcopy(self.curriculum); curriculum[index][kind]["facets"].pop()
                with self.subTest(unit=index, kind=kind), self.assertRaises(ContractError):
                    validate_materials(self.bp, curriculum, self.bank, self.material, Obligations())

    def test_rejected_source_or_authoring_produces_no_generated_outputs(self):
        for failure in ("source", "authoring"):
            with tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                for path in source_paths(ROOT):
                    destination = root / path; destination.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copyfile(ROOT / path, destination)
                if failure == "source":
                    broken = copy.deepcopy(self.bp); broken["objectives"][0]["facets"] = []
                    (root / "education/ncp-metablueprint/blueprint.json").write_text(json.dumps(broken))
                else:
                    with (root / "education/authoring/units.py").open("a") as stream: stream.write("\nUNITS[20]['code'] = 'broken syntax ('\n")
                with self.assertRaises((ContractError, SyntaxError)): render_repository(root)
                self.assertFalse((root / "verification/generated").exists())
                self.assertFalse((root / "education/ncp-metablueprint/courses").exists())


class ChecklistBoundary(unittest.TestCase):
    def test_missing_unknown_duplicate_false_and_stale_cannot_finish(self):
        ledger = Checklist(["one", "two"], "attempt/17")
        ledger.record("one")
        with self.assertRaises(ContractError): ledger.finish()
        for name, passed in [("one", True), ("other", True), ("two", 1)]:
            with self.assertRaises(ContractError): ledger.record(name, passed)
        ledger.record("two", False)
        with self.assertRaises(ContractError): ledger.finish()
        # A failed outcome cannot be replaced by a later passing value.
        with self.assertRaises(ContractError): ledger.record("two", True)

    def test_completion_is_recomputed_not_trusted(self):
        ledger = Checklist(["one", "two"], "attempt/17")
        ledger.record("two"); ledger.record("one")
        good = ledger.finish()
        Checklist.validate(good, ["one", "two"], "attempt/17")
        cases = []
        for key, value in [("complete", False), ("identity", "attempt/16"), ("required", ["one"]),
                           ("records", [{"name": "one", "passed": True}]),
                           ("records", [{"name": "one", "passed": True}] * 2)]:
            changed = copy.deepcopy(good); changed[key] = value; cases.append(changed)
        for changed in cases:
            with self.assertRaises(ContractError): Checklist.validate(changed, ["one", "two"], "attempt/17")

    def test_zero_obligations_never_mean_success(self):
        with self.assertRaises(ContractError): Checklist([], "attempt/17")


if __name__ == "__main__": unittest.main()
