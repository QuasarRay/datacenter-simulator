"""Seed only the freshly created disposable qualification database via its real ORM.

Invoked by manage.py shell after migrations. Never point this at an existing site.
The production topology compiler uses only the subsequently issued read-only token.
"""
from datetime import timedelta
import json
import os
from pathlib import Path

from django.conf import settings
from django.utils import timezone
from dcim.models import Cable, Device, DeviceRole, DeviceType, Interface, Manufacturer, Site
from users.models import Token, User

if settings.DATABASES['default']['NAME'] != 'ncp_netbox_ci' or Site.objects.exists():
    raise RuntimeError('requires the fresh dedicated qualification database')

site=Site.objects.create(name='NCP native qualification',slug='ncp-lab',status='active')
manufacturer=Manufacturer.objects.create(name='NCP emulated',slug='ncp-emulated')
kind=DeviceType.objects.create(manufacturer=manufacturer,model='Native system container',slug='native-container')
roles={name:DeviceRole.objects.create(name=name,slug=name) for name in ['control','gpu-server','services','leaf']}
devices={name:Device.objects.create(name=name,site=site,device_type=kind,role=roles[role],status='active')
         for name,role in [('control','control'),('compute-a','gpu-server'),('compute-b','gpu-server'),('services','services'),('leaf','leaf')]}
for index,name in enumerate(['control','compute-a','compute-b','services'],1):
    a=Interface.objects.create(device=devices[name],name='data0',type='1000base-t',speed=1000000)
    b=Interface.objects.create(device=devices['leaf'],name=f'swp{index}',type='1000base-t',speed=1000000)
    cable=Cable(a_terminations=[a],b_terminations=[b],status='connected')
    cable.full_clean();cable.save()
user=User.objects.create_superuser(username='ncp-ci-reader',password=None)
reader=Token.objects.create(user=user,write_enabled=False,description='Disposable native qualification',
                            expires=timezone.now()+timedelta(hours=1),allowed_ips=['127.0.0.1/32'])
path=Path(os.environ['NCP_NETBOX_TOKEN_FILE'])
with path.open('x') as stream:
    stream.write(f'nbt_{reader.key}.{reader.token}')
path.chmod(0o600)
print(json.dumps({'scope':'fresh-native-netbox-ci','devices':len(devices),'cables':4,'token_read_only':True}))
