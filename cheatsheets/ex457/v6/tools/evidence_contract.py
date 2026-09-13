"""Release content integrity; hashes are integrity checks, not signatures."""
import hashlib
from pathlib import Path
import re

EXCLUDE = {'.state', '.git', '.venv', 'collections', '__pycache__', '.pytest_cache', '.hypothesis', 'context', 'pytest-of-root'}


def release_files(root):
    root = Path(root)
    return sorted(p.relative_to(root).as_posix() for p in root.rglob('*')
                  if p.is_file() and not EXCLUDE.intersection(p.relative_to(root).parts)
                  and p.relative_to(root).as_posix() != 'evidence/manifest.sha256')


def manifest_text(root):
    root = Path(root)
    return ''.join(hashlib.sha256((root/p).read_bytes()).hexdigest()+'  '+p+'\n' for p in release_files(root))


def manifest_ok(root, text):
    root = Path(root)
    entries = {}
    for line in text.splitlines():
        match = re.fullmatch(r'([0-9a-f]{64})  ([^\r\n]+)', line)
        if not match:
            raise ValueError('malformed manifest')
        digest, name = match.groups()
        p = Path(name)
        if p.is_absolute() or '..' in p.parts or name in entries:
            raise ValueError('unsafe or duplicate manifest path')
        entries[name] = digest
    if not entries or set(entries) != set(release_files(root)):
        raise ValueError('manifest must cover the exact nonempty release file set')
    for name, digest in entries.items():
        if (root/name).is_symlink() or hashlib.sha256((root/name).read_bytes()).hexdigest() != digest:
            raise ValueError('manifest content mismatch: '+name)
    return True


def claims_ok(claims, sources):
    ids = {f'C{i:03}' for i in range(1, 43)}
    if len(claims) != len(ids) or {x.get('id') for x in claims} != ids:
        raise ValueError('claim IDs missing, duplicated or unexpected')
    for row in claims:
        if not row.get('claim') or not row.get('section') or not row.get('sources') or not set(row['sources']) <= set(sources):
            raise ValueError('claim has no text, section or resolving sources')
    return True


def history_ok(history):
    expected = {f'V3-{i:03}' for i in range(1, 33)} | {f'V4-{i:03}' for i in range(1, 49)} | {f'V5-F{i:03}' for i in range(1, 24)}
    rows = history['findings']
    if len(rows) != len(expected) or {r['id'] for r in rows} != expected:
        raise ValueError('available historical audit coverage is incomplete')
    for row in rows:
        if not row.get('tests') or not row.get('acceptance') or not row.get('source_sha256'):
            raise ValueError('finding has no test mapping, acceptance or provenance')
        if row['status'] not in {'IMPLEMENTED_RUNTIME_GATED', 'REGRESSION_COVERED', 'DOCUMENTED_BOUNDARY'}:
            raise ValueError('unsubstantiated historical closure status')
    return True
