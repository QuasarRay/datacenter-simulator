"""Challenge the compiler and its evidence boundary, not just successful examples."""
from dataclasses import replace
import json
from pathlib import Path
import tempfile
import unittest

from verification.compiler import expand
from verification.dsl import ContractError, Expr, emit, literal, node, variable
from verification.io import drift, publish, safe_path, strict_json
from verification.prove import kani_result, verus_result
from verification.specification import registry


class CompilerBoundary(unittest.TestCase):
    def test_python_short_circuit_cannot_erase_a_symbolic_branch(self):
        with self.assertRaises(ContractError):
            variable("value", "bool") and variable("other", "bool")

    def test_types_and_ranges_reject_python_bool_as_integer(self):
        for value, kind in [(True, "u8"), (-1, "u64"), (256, "u8"), (2**64, "u64"), (1, "bool")]:
            with self.subTest(value=value, kind=kind), self.assertRaises(ContractError): literal(value, kind)
        with self.assertRaises(ContractError): variable("value", "u8").eq(variable("other", "u64"))

    def test_raw_code_and_bypass_operations_are_not_a_language_feature(self):
        for name in ["result", "name(); unsafe", "x\n", "crate", "_private"]:
            with self.subTest(name=name), self.assertRaises(ContractError): variable(name, "bool")
        for op in ["assume", "admit", "external_body", "call", "eval", "raw"]:
            with self.subTest(op=op), self.assertRaises(ContractError): node(op, literal(True, "bool"), literal(True, "bool"))

    def test_forged_nodes_unbound_inputs_and_empty_specs_are_rejected(self):
        c = registry.contracts["subset"]
        variants = [replace(c, body=Expr("eq", "u8", (literal(1, "u8"), literal(1, "u8")))),
                    replace(c, body=variable("unknown", "bool")), replace(c, properties=()),
                    replace(c, properties=(literal(True, "bool"),)), replace(c, witnesses=())]
        for mutated in variants:
            with self.assertRaises(ContractError): mutated.validate()

    def test_vacuous_result_property_is_rejected(self):
        c = registry.contracts["subset"]
        result = Expr("var", "bool", ("result",))
        # Even mentioning every input cannot rescue a result-independent tautology.
        tautology = result.eq(result) & c.body.eq(c.body)
        with self.assertRaises(ContractError): replace(c, properties=(tautology,)).validate()

    def test_all_contracts_have_nonvacuous_witnesses(self):
        for c in registry.contracts.values():
            with self.subTest(contract=c.name): c.validate()

    def test_single_expression_is_used_by_both_execution_backends(self):
        first, second = expand(registry), expand(registry)
        self.assertEqual(first, second)
        for c in registry.contracts.values():
            body = "{ " + emit(c.body) + " }"
            self.assertIn(body, first["kernel/src/lib.rs"])
            self.assertIn(body, first["verus.rs"])
        for forbidden in ["kani::assume", "external_body", "admit(", "requires ", "unsafe {"]:
            self.assertNotIn(forbidden, first["kernel/src/lib.rs"] + first["verus.rs"])


class PublicationBoundary(unittest.TestCase):
    def test_duplicate_keys_and_nonfinite_json_cannot_hide_an_obligation(self):
        for data in ['{"id":1,"id":2}', '{"x":NaN}', '{"x":Infinity}']:
            with self.assertRaises(ValueError): strict_json(data)

    def test_publication_preflights_every_path_before_any_write(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            for bad in ["../escape", "/escape", "a/../escape", "a//escape", "a\\escape"]:
                with self.subTest(path=bad), self.assertRaises(ContractError):
                    publish(root, {"good": "one", bad: "two"}, "manifest.json", {})
                self.assertFalse((root / "good").exists())
            (root / "linked").symlink_to(root, target_is_directory=True)
            with self.assertRaises(ContractError): safe_path(root, "linked/x")

    def test_drift_is_read_only_and_a_partial_publish_cannot_pass(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            files = {"a": "A", "b": "B"}
            publish(root, files, "manifest.json", {"input": "one"})
            before = {p.name: (p.read_bytes(), p.stat().st_mtime_ns) for p in root.iterdir()}
            self.assertEqual(drift(root, files, "manifest.json", {"input": "one"}), [])
            self.assertEqual(before, {p.name: (p.read_bytes(), p.stat().st_mtime_ns) for p in root.iterdir()})
            (root / "a").write_text("partial change")
            self.assertIn("a", drift(root, files, "manifest.json", {"input": "one"}))
            self.assertIn("manifest.json", drift(root, files, "manifest.json", {"input": "two"}))
            (root / "human.md").write_text("authored")
            with self.assertRaises(ContractError): publish(root, {"a": "A"}, "manifest.json", {})
            self.assertEqual((root / "human.md").read_text(), "authored")


class ProofReportBoundary(unittest.TestCase):
    def test_kani_green_exit_with_missing_duplicated_or_failed_harness_is_rejected(self):
        good = "Checking harness proofs::law_one...\nVERIFICATION:- SUCCESSFUL\n"
        kani_result(good, 0, ["one"], False)
        for text, code, names in [("", 0, ["one"]), (good * 2, 0, ["one"]),
                                  (good, 0, ["one", "two"]), (good, 1, ["one"]),
                                  (good.replace("SUCCESSFUL", "FAILED"), 0, ["one"])]:
            with self.assertRaises(ContractError): kani_result(text, code, names, False)
        with self.assertRaises(ContractError): kani_result(good, 0, ["one"], True)

    def test_verus_no_verify_partial_crate_and_wrong_inventory_cannot_pass(self):
        report = {"func-details": {"verus::one": {}}, "verification-results": {
            "is-verifying-entire-crate": True, "encountered-vir-error": False,
            "verified": 1, "errors": 0, "success": True}}
        verus_result(json.dumps(report), 0, ["one"], False)
        for key, value in [("verified", 0), ("errors", 1), ("is-verifying-entire-crate", False),
                           ("encountered-vir-error", True), ("success", False)]:
            changed = json.loads(json.dumps(report)); changed["verification-results"][key] = value
            with self.subTest(key=key), self.assertRaises(ContractError): verus_result(json.dumps(changed), 0, ["one"], False)
        with self.assertRaises(ContractError): verus_result(json.dumps(report), 0, ["two"], False)
        with self.assertRaises(ContractError): verus_result(json.dumps(report), 0, ["one"], True)


if __name__ == "__main__": unittest.main()
