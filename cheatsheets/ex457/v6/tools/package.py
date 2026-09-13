"""Create a source ZIP containing the v6 pack and the repository CI entrypoints."""
import argparse
import hashlib
from pathlib import Path
import zipfile
from evidence_contract import manifest_ok, EXCLUDE
ROOT=Path(__file__).resolve().parents[1]
REPO=ROOT.parents[2]


def package(output):
    manifest_ok(ROOT,(ROOT/'evidence/manifest.sha256').read_text())
    output=Path(output).resolve();output.parent.mkdir(parents=True,exist_ok=True)
    allowed={'cheatsheets','.github','ci','inventory','playbooks','templates','tests','vars'}
    files=[]
    for path in sorted(REPO.rglob('*')):
        rel=path.relative_to(REPO)
        if rel.parts[:2]==('ci','sdk'):continue
        if not path.is_file() or path.is_symlink() or set(rel.parts).intersection(EXCLUDE|{'artifacts','build'}):continue
        if rel.parts[0] not in allowed and not (len(rel.parts)==1 and (path.suffix in {'.md','.yml','.yaml','.cfg','.ini','.py','.toml'} or path.name=='.gitignore')):continue
        if path.suffix in {'.zip','.pyc'} or path.name in {'site.yml','secrets.yml'}:continue
        files.append((path,rel))
    with zipfile.ZipFile(output,'w',zipfile.ZIP_DEFLATED,compresslevel=9) as archive:
        for path,rel in files:archive.write(path,'ex457-v6-project/'+rel.as_posix())
    with zipfile.ZipFile(output) as archive:
        assert archive.testzip() is None
        assert 'ex457-v6-project/ci/dagger.json' in archive.namelist()
        assert not any('/.state/' in n for n in archive.namelist())
    print(output.name,hashlib.sha256(output.read_bytes()).hexdigest(),len(files),'source files')


if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--output',required=True);args=p.parse_args();package(args.output)
