"""Strict JSON and bounded, deterministic publication of compiler-owned files."""
from contextlib import contextmanager
import fcntl
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import tempfile
from verification.dsl import require


def sha(data): return hashlib.sha256(data).hexdigest()


def strict_json(text):
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, f"duplicate JSON key: {key}")
            result[key] = value
        return result
    def invalid(value): raise ValueError(f"nonfinite JSON value: {value}")
    return json.loads(text, object_pairs_hook=pairs, parse_constant=invalid)


def safe_path(root, relative):
    require(type(relative) is str and "\\" not in relative and "\x00" not in relative, "invalid output path")
    path = PurePosixPath(relative)
    require(not path.is_absolute() and path.parts and all(p not in {".", ".."} for p in path.parts)
            and path.as_posix() == relative, f"noncanonical output path: {relative}")
    root = Path(root).absolute()
    require(not root.is_symlink(), "output root must not be a symlink")
    result = root.joinpath(*path.parts)
    for item in [result, *result.parents]:
        require(not item.is_symlink(), f"symlink forbidden: {item}")
        if item == root: break
    return result


def atomic_write(path, data):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, temporary = tempfile.mkstemp(prefix=".meta-", dir=path.parent)
    try:
        with os.fdopen(fd, "wb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
        directory = os.open(path.parent, os.O_RDONLY | os.O_DIRECTORY)
        try: os.fsync(directory)
        finally: os.close(directory)
    finally:
        if os.path.exists(temporary): os.unlink(temporary)


@contextmanager
def locked(root):
    path = safe_path(root, "artifacts/metaverify.lock")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a") as stream:
        fcntl.flock(stream, fcntl.LOCK_EX | fcntl.LOCK_NB)
        yield


def bundle_manifest(files, inputs):
    return {"schema": 1, "inputs": dict(sorted(inputs.items())),
            "outputs": {name: sha(content.encode()) for name, content in sorted(files.items())}}


def drift(root, files, manifest_path, inputs):
    expected = dict(files)
    expected[manifest_path] = json.dumps(bundle_manifest(files, inputs), indent=2) + "\n"
    errors = []
    for name, content in expected.items():
        path = safe_path(root, name)
        if not path.is_file() or path.read_bytes() != content.encode(): errors.append(name)
    old_path = safe_path(root, manifest_path)
    if old_path.exists():
        old = strict_json(old_path.read_text())
        for name in old["outputs"]:
            safe_path(root, name)
            if name not in files: errors.append("retired:" + name)
    return errors


def publish(root, files, manifest_path, inputs):
    """All content must be validated before this call. The manifest commits last.

    Each replacement is atomic. A crash between replacements is detected by the
    old manifest/check command, not described as a cross-file atomic transaction.
    Unlisted files are never deleted; retired generated paths need explicit review.
    """
    paths = {name: safe_path(root, name) for name in [*files, manifest_path]}
    require(manifest_path not in files and bool(files), "invalid ownership manifest")
    previous = paths[manifest_path]
    if previous.exists():
        old = strict_json(previous.read_text())
        require(set(old["outputs"]) <= set(files), "retired generated files require an explicit reviewed migration")
    for name, content in files.items():
        require(type(content) is str, f"not text: {name}")
    for name, content in files.items():
        path = paths[name]
        if not path.exists() or path.read_bytes() != content.encode(): atomic_write(path, content.encode())
    atomic_write(previous, (json.dumps(bundle_manifest(files, inputs), indent=2) + "\n").encode())
