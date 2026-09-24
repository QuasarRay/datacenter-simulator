#!/usr/bin/env python3
"""One compact entry point for contract generation, checking and real dual proofs."""
import argparse
import json
import os
from pathlib import Path
import sys
import subprocess
from contextlib import nullcontext

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from verification.compiler import expand
from verification.dsl import ContractError
from verification.io import drift, locked, publish, sha


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=["render", "check", "prove", "verify", "impact", "explain"])
    parser.add_argument("name", nargs="?")
    parser.add_argument("--fresh", action="store_true", help="ignore performance receipts and run both verifiers")
    parser.add_argument("--verus", default=os.environ.get("VERUS", "verus"))
    args = parser.parse_args()
    try:
        import shutil
        from verification.specification import registry
        from verification.dsl import emit, require
        from verification.pipeline import compile_repository, identities, MANIFEST, PROOF_INPUTS
        require(args.name is None or args.operation == "explain", "only explain accepts a name")
        result = {"status": "pass", "operation": args.operation, "contracts": len(registry.contracts)}
        if args.operation == "explain":
            require(args.name in registry.contracts, "unknown contract name")
            c = registry.contracts[args.name]
            result.update(name=c.name, scope=c.scope, parameters=dict(c.parameters),
                          implementation=emit(c.body), postconditions=[emit(p) for p in c.properties])
        else:
            with (nullcontext() if args.operation in {"check", "impact"} else locked(ROOT)):
                generated, identity, obligations = compile_repository(ROOT)
                result.update(artifacts=len(generated), material_obligations=obligations)
                if args.operation == "render": publish(ROOT, generated, MANIFEST, identity)
                elif args.operation == "impact":
                    from verification.io import strict_json
                    path = ROOT / MANIFEST
                    before = strict_json(path.read_text())["inputs"] if path.exists() else {}
                    changes = sorted(name for name in set(before) | set(identity) if before.get(name) != identity.get(name))
                    result.update(changed_inputs=changes, proof_inputs_changed=bool(set(changes) & set(PROOF_INPUTS)),
                                  generated_drift=drift(ROOT, generated, MANIFEST, identity)[:8])
                else:
                    changed = drift(ROOT, generated, MANIFEST, identity)
                    require(not changed, "generated drift; run meta.py render: " + ", ".join(changed[:8]))
                if args.operation == "verify":
                    from verification.audit import audit
                    result.update(audit(ROOT, obligations))
                if args.operation in {"prove", "verify"}:
                    from verification.prove import verify
                    result.update(verify(ROOT, registry, identities(ROOT, PROOF_INPUTS),
                                         verus=shutil.which(args.verus) or args.verus, fresh=args.fresh))
                require(all(sha((ROOT / path).read_bytes()) == digest for path, digest in identity.items()),
                        "source changed during the gate")
        print(json.dumps(result, separators=(",", ":")))
        return 0
    except (ValueError, OSError, KeyError, TypeError, ImportError, SyntaxError, subprocess.SubprocessError) as error:
        print(json.dumps({"status": "fail", "operation": args.operation, "error": str(error)[:2000]}, separators=(",", ":")))
        return 1


if __name__ == "__main__": raise SystemExit(main())
