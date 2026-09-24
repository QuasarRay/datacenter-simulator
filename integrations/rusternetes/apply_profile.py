"""Apply the reviewed native control-plane adaptation to one exact source pin.

This is an explicit derived build, never an assertion that upstream is unchanged.
It removes the API server's runtime clients and fabricated log fallback, and
wires its declared client-CA option into the real TLS verifier.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

HERE = Path(__file__).resolve().parent
PIN = '64b56a4ac3b33b1a5cbd33313ed033f15605d294'

def replace_once(text, old, new):
    if text.count(old) != 1:
        raise ValueError('upstream source shape changed; review the profile')
    return text.replace(old, new, 1)

def apply(source):
    source = Path(source).resolve()
    def git(*args):
        return subprocess.check_output(['git', '-C', str(source), *args], text=True).strip()
    if git('rev-parse', 'HEAD') != PIN or git('status', '--porcelain', '--untracked-files=no'):
        raise ValueError('requires the exact clean source revision')
    changes = {}
    path = 'crates/api-server/src/handlers/pod_subresources.rs'
    old = (source/path).read_text()
    boundary = '/// POST /api/v1/namespaces/{namespace}/pods/{name}/binding\n'
    if old.count(boundary) != 1:
        raise ValueError('pod subresource boundary changed')
    changes[path] = (HERE/'pod_runtime.rs').read_text() + old[old.index(boundary):]
    for path in ['crates/api-server/src/lib.rs', 'crates/api-server/src/main.rs']:
        text = (source/path).read_text()
        prefix = 'pub ' if path.endswith('lib.rs') else ''
        for module in ['spdy', 'spdy_handlers', 'streaming']:
            text = replace_once(text, f'#[allow(dead_code)]\n{prefix}mod {module};\n', '')
        if path.endswith('main.rs'):
            text = replace_once(text,
                'let rustls_config = RustlsConfig::from_config(tls_config.into_server_config()?);',
                'let server_config = if let Some(ref ca) = args.client_ca_file {\n'
                '            tls_config.into_mtls_server_config(ca)?\n'
                '        } else { tls_config.into_server_config()? };\n'
                '        let rustls_config = RustlsConfig::from_config(server_config);')
        changes[path] = text
    path = 'crates/api-server/Cargo.toml'
    changes[path] = replace_once((source/path).read_text(), 'bollard.workspace = true\n', '')
    path = 'Cargo.lock'
    text = (source/path).read_text()
    start = text.index('name = "rusternetes-api-server"\n')
    end = text.index('[[package]]', start)
    changes[path] = text[:start] + replace_once(text[start:end], ' "bollard",\n', '') + text[end:]
    # Calculate all changes before writing: any shape error leaves the checkout intact.
    for path, text in changes.items():
        (source/path).write_text(text)
    record = {'profile': 'ncp-native-control-plane-v1', 'upstream': PIN,
              'modified_files': {p: hashlib.sha256(t.encode()).hexdigest() for p,t in changes.items()},
              'pod_runtime': 'unavailable-501', 'client_tls': 'ca-verified; bearer RBAC still required'}
    (source/'ncp-profile.json').write_text(json.dumps(record, indent=2)+'\n')
    return record

if __name__ == '__main__':
    if len(sys.argv) != 2: raise SystemExit('one clean pinned Rusternetes checkout is required')
    print(json.dumps(apply(sys.argv[1]), indent=2))
