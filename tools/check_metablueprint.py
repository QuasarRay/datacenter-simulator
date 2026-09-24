#!/usr/bin/env python3
"""Check the reviewed source-obligation lock through the shared strict validator."""
from pathlib import Path
import sys
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT))
from verification.io import strict_json
from verification.materials import validate_blueprint


def validate(data):
    try:
        validate_blueprint(data, strict_json((ROOT/'verification/source-obligations.json').read_text()))
        return []
    except (ValueError, TypeError, KeyError, OSError) as error:
        return [str(error)]


if __name__=='__main__':
    data=strict_json((ROOT/'education/ncp-metablueprint/blueprint.json').read_text())
    errors=validate(data)
    if errors:raise SystemExit('\n'.join(errors))
    print('PASS: 113 locked source objectives, 203 source facets and 20 tri-domain units')
