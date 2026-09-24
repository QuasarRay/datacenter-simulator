#!/usr/bin/env python3
"""One compact entry point for contract generation, checking and real dual proofs."""
import argparse
import json
import os
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from verification.compiler import expand
from verification.dsl import ContractError
from verification.io import drift, locked, publish, sha


def inputs():
    paths = ["tools/meta.py", "verification/__init__.py", "verification/dsl.py", "verification/specification.py",
             "verification/compiler.py", "verification/io.py", "verification/prove.py", "verification/toolchain.json"]
    return {name: sha((ROOT / name).read_bytes()) for name in paths}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=["render", "check", "prove", "explain"])
    parser.add_argument("name", nargs="?")
    parser.add_argument("--fresh", action="store_true", help="ignore performance receipts and run both verifiers")
    parser.add_argument("--verus", default=os.environ.get("VERUS", "verus"))
    args = parser.parse_args()
    try:
        import shutil
        from verification.specification import registry
        from verification.dsl import emit, require
        require(args.name is None or args.operation == "explain", "only explain accepts a name")
        generated = {"verification/generated/" + name: content for name, content in expand(registry).items()}
        manifest = "verification/generated/manifest.json"
        identity = inputs()
        result = {"status": "pass", "operation": args.operation, "contracts": len(registry.contracts)}
        if args.operation == "explain":
            require(args.name in registry.contracts, "unknown contract name")
            c = registry.contracts[args.name]
            result.update(name=c.name, scope=c.scope, parameters=dict(c.parameters),
                          implementation=emit(c.body), postconditions=[emit(p) for p in c.properties])
        elif args.operation == "check":
            changed = drift(ROOT, generated, manifest, identity)
            require(not changed, "generated drift: " + ", ".join(changed[:8]))
        else:
            with locked(ROOT):
                if args.operation == "render": publish(ROOT, generated, manifest, identity)
                elif args.operation == "prove":
                    changed = drift(ROOT, generated, manifest, identity)
                    require(not changed, "run meta.py render before proving changed inputs")
                    from verification.prove import verify
                    result.update(verify(ROOT, registry, identity, verus=shutil.which(args.verus) or args.verus, fresh=args.fresh))
        print(json.dumps(result, separators=(",", ":")))
        return 0
    except (ValueError, OSError, KeyError, TypeError, ImportError, SyntaxError) as error:
        print(json.dumps({"status": "fail", "operation": args.operation, "error": str(error)[:2000]}, separators=(",", ":")))
        return 1


if __name__ == "__main__": raise SystemExit(main())
