"""Enforce the exact exported run; never re-execute tests to decide its status."""
import argparse
import hashlib
import json
from pathlib import Path
import xml.etree.ElementTree as ET
import zipfile
from run_ci import REQUIRED


def check(directory, run_id, pack):
    directory=Path(directory);pack=Path(pack)
    report=json.loads((directory/'results.json').read_text())
    assert report['schema']==1 and report['run_id']==run_id,'wrong CI run identity'
    assert report['status']=='PASS','CI report contains failed or missing required gates'
    expected=hashlib.sha256((pack/'evidence/manifest.sha256').read_bytes()).hexdigest()
    assert report['source_manifest_sha256']==expected,'CI report belongs to different source content'
    rows=report['checks'];names=[r['name'] for r in rows]
    assert len(names)==len(set(names)) and set(REQUIRED)|{'package'} <= set(names),'missing or duplicate checks'
    assert report['required']==REQUIRED,'required gate definition changed'
    assert all(r['status']=='PASS' and r['returncode']==0 for r in rows),'a reported check did not pass'
    tests=ET.parse(directory/'pytest.xml').findall('.//testcase')
    assert tests and not any(t.find(tag) is not None for t in tests for tag in ['failure','error','skipped']),'empty or incomplete pytest run'
    history=json.loads((pack/'evidence/audit-history.json').read_text())
    for row in history['findings']:
        for target in row['tests']:
            file,symbol=target.split('::',1)
            if file=='tests/pyats_live.py':continue
            module=file.removesuffix('.py').replace('/','.')
            assert any((t.get('classname')==module and t.get('name')==symbol) or t.get('classname','').startswith(module+'.'+symbol) for t in tests),'historical regression was not executed: '+target
    archive=directory/'ex457_containerlab_cheatsheet_v6.zip'
    assert archive.is_file(),'delivery ZIP missing'
    with zipfile.ZipFile(archive) as zipped:
        assert zipped.testzip() is None,'damaged delivery ZIP'
        assert not any('/.state/' in n for n in zipped.namelist()),'private runtime state in ZIP'
        prefix='ex457-v6-project/cheatsheets/ex457/v6/'
        assert zipped.read(prefix+'evidence/manifest.sha256')==(pack/'evidence/manifest.sha256').read_bytes(),'ZIP manifest differs from tested source'
        for line in (pack/'evidence/manifest.sha256').read_text().splitlines():
            digest,name=line.split('  ',1)
            assert hashlib.sha256(zipped.read(prefix+name)).hexdigest()==digest,'ZIP source mismatch: '+name
    return report


if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--directory',default='artifacts');parser.add_argument('--run-id',required=True);parser.add_argument('--pack',default='cheatsheets/ex457/v6');args=parser.parse_args()
    check(args.directory,args.run_id,args.pack)
    print('Exact exported run passed all required gates and audit-mapped pytest cases; delivery ZIP verified.')
