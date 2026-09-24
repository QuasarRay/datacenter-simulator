"""Prevent reintroducing retired executable deployment backends."""
from pathlib import Path
import ast
import re

ROOT=Path(__file__).resolve().parents[1]
forbidden={'docker','dockerd','containerlab'}
errors=[]
for relative in ['playbooks','integrations/incus','ci','.github/workflows']:
    for p in (ROOT/relative).rglob('*'):
        if not p.is_file() or p.suffix not in {'.py','.yml','.yaml','.sh'}:
            continue
        text=p.read_text()
        if p.suffix=='.py':
            for node in ast.walk(ast.parse(text)):
                if isinstance(node,ast.List) and node.elts and isinstance(node.elts[0],ast.Constant) and node.elts[0].value in forbidden:
                    errors.append(str(p.relative_to(ROOT)))
        elif re.search(r'(?m)^\s*(?:-\s*run:\s*)?(?:sudo\s+)?(?:docker|dockerd|containerlab)\s',text):
            errors.append(str(p.relative_to(ROOT)))
for p in ROOT.rglob('Dockerfile'):
    if 'upstream' not in p.parts and 'vendor' not in p.parts:
        errors.append(str(p.relative_to(ROOT)))
if errors:
    raise SystemExit('Retired runtime found: '+', '.join(errors))
print('PASS: active project deployment has no retired runtime executables')
