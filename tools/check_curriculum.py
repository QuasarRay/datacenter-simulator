"""Validate committed course/project/exercise data with the shared obligation engine."""
from pathlib import Path
import sys
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT))
from verification.io import safe_path, strict_json
from verification.materials import Obligations, validate_blueprint, validate_materials


def validate(base):
    try:
        base=Path(base)
        bp=strict_json((base/'blueprint.json').read_text())
        validate_blueprint(bp,strict_json((ROOT/'verification/source-obligations.json').read_text()))
        curriculum=strict_json((base/'curriculum.json').read_text())
        bank=strict_json((ROOT/'education/assessment/bank.json').read_text())
        rendered={}
        for unit in bp['units']:
            paths=[f"courses/{unit['course']}.md", f"examples/{unit['course']}.py", f"projects/{unit['project']}.md",
                   *(f"exercises/{name}.md" for name in unit['exercises'])]
            for path in paths:rendered[path]=safe_path(base,path).read_text()
        validate_materials(bp,curriculum,bank,rendered,Obligations())
        return []
    except (ValueError, TypeError, KeyError, OSError) as error:
        return [str(error)]


if __name__=='__main__':
    errors=validate(ROOT/'education/ncp-metablueprint')
    if errors:raise SystemExit('\n'.join(errors))
    print('PASS: declared learning graph, exact source facets and every station; live teaching remains separate')
