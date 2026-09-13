"""Private, atomic local state and per-run evidence; no implicit successful run."""
import contextlib
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import tempfile
import uuid


def secure_dir(path):
    path = Path(path)
    if path.is_symlink():
        raise ValueError('state directory must not be a symlink')
    path.mkdir(parents=True, mode=0o700, exist_ok=True)
    path.chmod(0o700)
    return path


def atomic_json(path, data):
    path = Path(path)
    secure_dir(path.parent)
    fd, temporary = tempfile.mkstemp(prefix='.write-', dir=path.parent)
    try:
        with os.fdopen(fd, 'w') as stream:
            json.dump(data, stream, indent=2, sort_keys=True)
            stream.write('\n')
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)


def source_digest(root):
    root = Path(root)
    digest = hashlib.sha256()
    ignored = {'.state', '.git', '.venv', 'collections', '__pycache__', '.pytest_cache', '.hypothesis', 'context', 'evidence', 'pytest-of-root'}
    for path in sorted(root.rglob('*')):
        rel = path.relative_to(root)
        if path.is_file() and not set(rel.parts).intersection(ignored):
            digest.update(rel.as_posix().encode() + b'\0' + hashlib.sha256(path.read_bytes()).digest())
    return digest.hexdigest()


@contextlib.contextmanager
def recorded_run(root, kind, state=None):
    root = Path(root)
    state = secure_dir(state or root / '.state')
    folder = secure_dir(state / 'runtime')
    now = lambda: datetime.now(timezone.utc).isoformat()
    record = {'schema': 1, 'run_id': uuid.uuid4().hex, 'kind': kind, 'started_at': now(),
              'finished_at': None, 'status': 'RUNNING', 'source_sha256': source_digest(root),
              'model_sha256': hashlib.sha256((root/'model-upstream.yml').read_bytes()).hexdigest(), 'results': []}
    path = folder / (kind + '-' + record['run_id'] + '.json')
    latest = state / (kind + '.json')
    atomic_json(path, record)
    atomic_json(latest, record)
    try:
        yield record
    except BaseException as error:
        record['status'] = 'FAIL'
        record['error_type'] = type(error).__name__
        raise
    else:
        record['status'] = 'PASS'
    finally:
        record['finished_at'] = now()
        atomic_json(path, record)
        atomic_json(latest, record)
