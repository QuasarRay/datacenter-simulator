"""Run both real verifiers; fail closed on missing, stale, partial or unexpected output.

Cached receipts are a local performance aid, not remote attestations. CI uses
fresh runs. Tool executables, dependency trees and source bytes bind every key.
"""
import json
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import tempfile
from verification.compiler import expand, instances
from verification.dsl import require
from verification.io import atomic_write, sha, strict_json


def execute(command, directory, label, timeout=300):
    env = dict(os.environ)
    for key in list(env):
        if key in {"RUSTFLAGS", "RUSTDOCFLAGS", "RUSTC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER",
                   "CARGO_ENCODED_RUSTFLAGS", "RUSTUP_TOOLCHAIN", "KANIFLAGS", "VERUS_ARGS"}:
            env.pop(key)
    out, err = directory / (label + ".stdout"), directory / (label + ".stderr")
    process = subprocess.Popen(command, cwd=directory, env=env, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, start_new_session=True)
    failure = None
    try:
        stdout, stderr = process.communicate(timeout=timeout)
    except BaseException as error:
        os.killpg(process.pid, signal.SIGKILL)
        stdout, stderr = process.communicate(timeout=10)
        failure = error
    require(len(stdout) + len(stderr) <= 16 * 1024 * 1024, f"excessive tool output: {label}")
    # The tool never owns the accepted transcript's file descriptor. Publish an
    # atomic snapshot only after EOF; inherited descriptors cannot truncate it.
    atomic_write(out, stdout)
    atomic_write(err, stderr)
    if failure is not None:
        raise failure
    return {"command": command, "exit": process.returncode, "stdout": out.name, "stderr": err.name}


def verus_result(text, code, names, mutant):
    report = strict_json(text)
    result = report["verification-results"]
    expected_count = len(names)
    require(result["is-verifying-entire-crate"] is True and result["encountered-vir-error"] is False,
            "Verus did not verify the entire valid crate")
    require(result["encountered-error"] is mutant, "Verus error flag contradicts the expected proof outcome")
    actual = {name.split("::")[-1] for name in report["func-details"] if not name.startswith("vstd::")}
    require(actual == set(names), "Verus function inventory mismatch")
    require(result["verified"] == (0 if mutant else expected_count)
            and result["errors"] == (expected_count if mutant else 0)
            and result["success"] is (not mutant)
            and ((code != 0) if mutant else code == 0), "Verus proof or negative control did not meet its contract")


def kani_result(text, code, names, mutant):
    pattern = r"Checking harness ([a-zA-Z0-9_:]+)\.\.\.(.*?)(?=Checking harness |\Z)"
    records = re.findall(pattern, text, flags=re.S)
    expected = {"proofs::law_" + name for name in names}
    require(len(records) == len(expected) and {n for n, _ in records} == expected, "Kani harness inventory mismatch")
    for name, output in records:
        statuses = re.findall(r"VERIFICATION:- (SUCCESSFUL|FAILED)", output)
        require(statuses == ["FAILED" if mutant else "SUCCESSFUL"], f"Kani incomplete or unexpected status: {name}")
    require((code != 0) if mutant else code == 0, "Kani exit status contradicts report")


def tree_identity(directory):
    """Hash the installed verifier distribution, including solver and libraries."""
    directory = Path(directory).resolve()
    require(directory.is_dir(), f"missing verifier distribution: {directory}")
    digest = __import__("hashlib").sha256()
    # No generated proof receipts or build caches are located in these trees.
    for path in sorted(directory.rglob("*")):
        if path.is_file():
            digest.update(path.relative_to(directory).as_posix().encode() + b"\0")
            with path.open("rb") as stream:
                for chunk in iter(lambda: stream.read(1024 * 1024), b""): digest.update(chunk)
    return digest.hexdigest()


def verify(root, registry, inputs, *, verus, fresh=False):
    root = Path(root)
    toolchain = strict_json((root / "verification/toolchain.json").read_text())
    cargo = shutil.which("cargo")
    require(cargo is not None and Path(verus).is_file(), "install the pinned Kani and Verus tools first")
    names = {"positive": list(registry.contracts), "mutants": [name for name, _, _ in instances(registry, True)]}
    base = root / "artifacts/metaverify"
    base.mkdir(parents=True, exist_ok=True)
    directory = Path(tempfile.mkdtemp(prefix="run-", dir=base))
    # Any failure retains its full transcript but never writes an accepted receipt.
    versions = {}
    for name, command in [("kani", [cargo, "kani", "--version"]),
                          ("verus", [str(Path(verus).resolve()), "--version", "--output-json"]),
                          ("rust", [cargo, "+" + toolchain["rust"], "--version"])]:
        run = execute(command, directory, name + "-version")
        output = (directory / run["stdout"]).read_text()
        observed = (strict_json(output)["verus"]["version"] if name == "verus"
                    else output.split()[1] if len(output.split()) >= 2 else None)
        require(run["exit"] == 0 and toolchain[name] == observed, f"{name}: wrong or missing pinned version")
        versions[name] = output.strip()
    kani_home = Path.home() / ".kani" / ("kani-" + toolchain["kani"])
    tools = {"versions": versions, "verus_tree": tree_identity(Path(verus).parent),
             "kani_tree": tree_identity(kani_home), "cargo": sha(Path(cargo).resolve().read_bytes())}
    key = sha(json.dumps({"inputs": inputs, "tools": tools}, sort_keys=True).encode())
    index = base / (key + ".json")

    def commands(folder, mutant):
        prefix = "mutants" if mutant else "positive"
        suffix = "-mutants" if mutant else ""
        return {
            "verus" + suffix: [str(Path(verus).resolve()), "--no-cheating", "--output-json", str(folder / prefix / "verus.rs")],
            "kani" + suffix: [cargo, "kani", "--manifest-path", str(folder / prefix / "kernel/Cargo.toml"), "--output-format", "terse"],
        }

    def inspect_receipt(receipt):
        require(set(receipt) == {"schema", "key", "inputs", "tools", "names", "runs", "files", "directory"}, "unexpected proof receipt")
        require(receipt["schema"] == 1 and receipt["key"] == key and receipt["inputs"] == inputs
                and receipt["tools"] == tools and receipt["names"] == names, "stale proof receipt")
        folder = base / receipt["directory"]
        require(folder.parent == base and folder.is_dir() and not folder.is_symlink(), "invalid receipt directory")
        for path, digest in receipt["files"].items():
            from verification.io import safe_path
            target = safe_path(folder, path)
            require(target.is_file() and sha(target.read_bytes()) == digest, "cached proof bytes changed")
        expected_runs = {"verus", "kani", "verus-mutants", "kani-mutants"}
        require(set(receipt["runs"]) == expected_runs, "missing verifier or negative-control run")
        for label, run in receipt["runs"].items():
            require(run["command"] == commands(folder, "mutants" in label)[label], "proof command drift")
            require(run["stdout"] in receipt["files"] and run["stderr"] in receipt["files"], "unbound transcript")
            output = (folder / run["stdout"]).read_text()
            try:
                mutant = "mutants" in label
                (verus_result if label.startswith("verus") else kani_result)(output, run["exit"], names["mutants" if mutant else "positive"], mutant)
            except (ValueError, KeyError, TypeError) as error:
                raise ValueError(f"{label}: invalid transcript; inspect {folder.relative_to(root)}: {error}") from error
        # A receipt must bind every expected expansion, not merely some log files.
        for mutant in (False, True):
            prefix = "mutants" if mutant else "positive"
            for path, content in expand(registry, mutants=mutant).items():
                require(receipt["files"].get(prefix + "/" + path) == sha(content.encode()), "proof source missing or drifted")
        return folder

    if not fresh and index.exists():
        try:
            cached = strict_json(index.read_text())
            folder = inspect_receipt(cached)
            shutil.rmtree(directory)
            return {"contracts": len(names["positive"]), "mutants_rejected": 2 * len(names["mutants"]), "cached": True, "evidence": str(folder.relative_to(root))}
        except (ValueError, KeyError, TypeError, OSError):
            pass  # An unusable cache triggers actual verification; it never passes.
    runs = {}
    for mutant in (False, True):
        prefix = "mutants" if mutant else "positive"
        for name, content in expand(registry, mutants=mutant).items():
            path = directory / prefix / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content)
        for label, command in commands(directory, mutant).items():
            runs[label] = execute(command, directory, label)
            output = (directory / runs[label]["stdout"]).read_text()
            try:
                (verus_result if label.startswith("verus") else kani_result)(output, runs[label]["exit"], names["mutants" if mutant else "positive"], mutant)
            except (ValueError, KeyError, TypeError) as error:
                raise ValueError(f"{label}: {error}; inspect {directory.relative_to(root)}") from error
    files = {}
    for path in sorted(directory.rglob("*")):
        if path.is_file() and "target" not in path.relative_to(directory).parts:
            files[path.relative_to(directory).as_posix()] = sha(path.read_bytes())
    require(all(sha((root / name).read_bytes()) == digest for name, digest in inputs.items()), "source changed during verification")
    receipt = {"schema": 1, "key": key, "inputs": inputs, "tools": tools, "names": names,
               "runs": runs, "files": files, "directory": directory.name}
    inspect_receipt(receipt)
    atomic_write(directory / "receipt.json", (json.dumps(receipt, indent=2) + "\n").encode())
    atomic_write(index, (json.dumps(receipt, indent=2) + "\n").encode())
    return {"contracts": len(names["positive"]), "mutants_rejected": 2 * len(names["mutants"]), "cached": False, "evidence": str(directory.relative_to(root))}
