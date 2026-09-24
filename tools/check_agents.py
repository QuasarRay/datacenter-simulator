"""Keep the user's full project directives in every repository-owned directory.

Gitlinks are separate repositories. Build outputs, .git and untracked caches are
not source directories. --sync generates missing copies without erasing local
additional guidance; the default operation is read-only and suitable for CI.
"""
from pathlib import Path, PurePosixPath
import subprocess
import sys

ROOT=Path(__file__).resolve().parents[1]
required=(ROOT/'AGENTS.md').read_text()
directories={PurePosixPath('.')}
for record in subprocess.check_output(['git','-C',str(ROOT),'ls-files','--stage','-z']).decode().split('\0'):
    if not record:continue
    metadata,path=record.split('\t',1)
    if metadata.startswith('160000 '):continue
    directories.update(PurePosixPath(path).parents)
sync=sys.argv[1:]==['--sync']
if sys.argv[1:] and not sync:raise SystemExit('optional --sync; no other arguments')
missing=[]
for directory in sorted(directories):
    path=ROOT/str(directory)/'AGENTS.md'
    if path.exists() and required in path.read_text():continue
    if sync:
        before=path.read_text() if path.exists() else ''
        path.write_text(required+('\n---\n\n'+before if before else ''))
    else:missing.append(str(path.relative_to(ROOT)))
if missing:raise SystemExit('Full project directives missing from: '+', '.join(missing))
print(f'PASS: full directives in {len(directories)} repository-owned directories')
