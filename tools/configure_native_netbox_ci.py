"""Configure only a fresh local native NetBox qualification installation."""
import os
from pathlib import Path
import secrets
import subprocess
import sys

source=Path(sys.argv[1]).resolve()
pin=subprocess.check_output(['git','-C',str(source),'rev-parse','HEAD'],text=True).strip()
if pin!='00791344e68213bde942218283dce03cc3941c30':raise SystemExit('wrong NetBox revision')
password=secrets.token_hex(32)
sql=f"CREATE USER ncp_netbox_ci WITH PASSWORD '{password}';\nCREATE DATABASE ncp_netbox_ci OWNER ncp_netbox_ci;\n"
subprocess.run(['sudo','-u','postgres','psql','-v','ON_ERROR_STOP=1'],input=sql,text=True,check=True,capture_output=True)
settings={'ALLOWED_HOSTS':['127.0.0.1','localhost'],'SECRET_KEY':secrets.token_hex(48),
 'API_TOKEN_PEPPERS':{1:secrets.token_hex(48)},
 'DATABASES':{'default':{'ENGINE':'django.db.backends.postgresql','NAME':'ncp_netbox_ci','USER':'ncp_netbox_ci',
                       'PASSWORD':password,'HOST':'127.0.0.1','PORT':5432}},
 'REDIS':{name:{'HOST':'127.0.0.1','PORT':6379,'PASSWORD':'','DATABASE':db,'SSL':False} for name,db in [('tasks',0),('caching',1)]}}
path=source/'netbox/netbox/configuration.py'
with path.open('x') as stream:
    for key,value in settings.items():stream.write(f'{key} = {value!r}\n')
path.chmod(0o600)
