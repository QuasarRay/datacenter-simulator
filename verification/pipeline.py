"""Validate -> expand in memory -> refine -> publish. No output on invalid input."""
import json
import os
from pathlib import Path
from types import ModuleType
from verification.compiler import expand
from verification.dsl import require
from verification.io import locked, publish, sha, strict_json
from verification.materials import Obligations, validate_authoring, validate_blueprint, validate_materials
from verification.specification import registry

MANIFEST = "verification/generated/manifest.json"
PROOF_INPUTS = ("tools/meta.py", "verification/__init__.py", "verification/dsl.py", "verification/specification.py",
                "verification/compiler.py", "verification/io.py", "verification/prove.py", "verification/toolchain.json")
MATERIAL_INPUTS = ("verification/pipeline.py", "verification/materials.py", "verification/source-obligations.json",
                   "education/ncp-metablueprint/blueprint.json", "education/authoring/units.py",
                   "tools/build_curriculum.py", "tools/build_assessment_bank.py")


def identities(root, paths): return {name: sha((root / name).read_bytes()) for name in paths}


def source_paths(root):
    # New reusable helpers and self-tests join the closure automatically.
    return tuple(sorted(set(PROOF_INPUTS + MATERIAL_INPUTS) |
                        {p.relative_to(root).as_posix() for p in (root / "verification").rglob("*.py")
                         if not {"generated", "target", "__pycache__"} & set(p.relative_to(root / "verification").parts)}))


def load_module(root, path, name):
    # These are reviewed authoring/generator modules, never learner submissions.
    # Compile current bytes: timestamp/size-based pyc reuse could otherwise render
    # old content while the manifest records the hash of a new same-sized source.
    module = ModuleType(name)
    module.__file__ = str(root / path)
    exec(compile((root / path).read_bytes(), str(root / path), "exec"), module.__dict__)
    return module


def compile_repository(root):
    root = Path(root)
    paths = source_paths(root)
    inputs = identities(root, paths)
    bp = strict_json((root / "education/ncp-metablueprint/blueprint.json").read_text())
    source_lock = strict_json((root / "verification/source-obligations.json").read_text())
    validate_blueprint(bp, source_lock)
    authored = load_module(root, "education/authoring/units.py", "ncp_authored").UNITS
    validate_authoring(authored, bp)
    material = load_module(root, "tools/build_curriculum.py", "ncp_material_renderer").render(bp, authored)
    bank = load_module(root, "tools/build_assessment_bank.py", "ncp_bank_renderer").build(bp)
    obligations = Obligations()
    validate_materials(bp, strict_json(material["curriculum.json"]), bank, material, obligations)
    files = {"education/ncp-metablueprint/" + path: value for path, value in material.items()}
    files["education/assessment/bank.json"] = json.dumps(bank, indent=2) + "\n"
    files.update({"verification/generated/" + path: value for path, value in expand(registry).items()})
    files["verification/generated/kernel/src/bin/check_materials.rs"] = obligations.rust()
    allowed = set(files) | {MANIFEST, "education/ncp-metablueprint/courses/C00.md",
                           "education/ncp-metablueprint/courses/C01-NETBOX.md"}
    for scope in ["verification/generated", *("education/ncp-metablueprint/" + name for name in ["courses", "projects", "exercises", "examples"])]:
        for parent, directories, names in os.walk(root / scope):
            directories[:] = [name for name in directories if name not in {"target", "__pycache__"}]
            for name in names:
                relative = (Path(parent) / name).relative_to(root).as_posix()
                require(name == "AGENTS.md" or relative in allowed, f"unregistered artifact: {relative}")
    require(paths == source_paths(root) and inputs == identities(root, paths), "source changed while compiling")
    return files, inputs, len(obligations.facts)


def render_repository(root):
    with locked(root):
        files, inputs, obligations = compile_repository(root)
        publish(root, files, MANIFEST, inputs)
    return {"artifacts": len(files), "obligations": obligations}
