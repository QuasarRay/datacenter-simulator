"""Compact self-audit: transcripts stay on disk; failure remains a hard gate."""
from pathlib import Path
import shutil
import sys
import tempfile
from verification.dsl import require
from verification.io import strict_json
from verification.prove import execute


def audit(root, obligations):
    output = root / "artifacts/metaverify"
    output.mkdir(parents=True, exist_ok=True)
    folder = Path(tempfile.mkdtemp(prefix="audit-", dir=output))
    cargo = shutil.which("cargo")
    require(cargo is not None, "Rust is required for the compiled material check")
    lock = strict_json((root / "verification/toolchain.json").read_text())
    manifest = str(root / "verification/generated/kernel/Cargo.toml")
    for name, command in [
        ("boundaries", [sys.executable, "-m", "unittest", "discover", "-s", str(root / "verification/tests"), "-t", str(root)]),
        ("optimized-boundaries", [sys.executable, "-O", "-m", "unittest", "discover", "-s", str(root / "verification/tests"), "-t", str(root)]),
        ("rust-witnesses", [cargo, "+" + lock["rust"], "test", "--locked", "--manifest-path", manifest, "--lib"]),
        ("material-obligations", [cargo, "+" + lock["rust"], "run", "--quiet", "--locked", "--manifest-path", manifest, "--bin", "check_materials"]),
    ]:
        run = execute(command, folder, name)
        require(run["exit"] == 0, f"{name}: inspect {folder.relative_to(root)}")
        if name == "material-obligations":
            result = strict_json((folder / run["stdout"]).read_text())
            require(result == {"obligations": obligations, "failed": 0}, "compiled material inventory mismatch")
    return {"audit": str(folder.relative_to(root))}
