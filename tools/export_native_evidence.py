"""Export this disposable qualification's reports without opening private lab state.

Only fixed CI report locations are archived. Credentials and native service
configuration under /tmp are deliberately absent from that allowlist.
"""
from pathlib import Path
import tarfile

ROOT=Path(__file__).resolve().parents[1]

def export(root=ROOT, extra=Path('/tmp')):
    root=Path(root).resolve();extra=Path(extra).resolve()
    output=root/'artifacts/native-evidence.tar.gz'
    output.parent.mkdir(exist_ok=True)
    sources=[(root/'artifacts'/name,name) for name in ('incus-qualification','netbox-qualification')]
    sources += [(extra/name,'host/'+name) for name in ('ncp-image.json','ncp-incus-version.txt','ncp-netbox.log')]
    entries=[]
    for path, archive in sources:
        if not path.exists():continue
        if path.is_symlink():raise ValueError('qualification artifact must not be a symlink')
        if path.is_dir():
            for child in sorted(path.rglob('*')):
                if child.is_symlink():raise ValueError('qualification artifact must not contain symlinks')
                if child.is_file():entries.append((child,archive+'/'+str(child.relative_to(path))))
        else:entries.append((path,archive))
    with tarfile.open(output,'x:gz') as archive:
        for path,name in entries:
            info=archive.gettarinfo(str(path),arcname=name)
            info.uid=info.gid=0;info.uname=info.gname='';info.mode=0o644
            with path.open('rb') as stream:archive.addfile(info,stream)
    output.chmod(0o644)
    return output

if __name__=='__main__':print(export())
