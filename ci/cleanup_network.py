"""Release only an empty, model-matching Containerlab management network."""
import argparse
import json
import subprocess
import time


def validate(network, name, subnet):
    if network.get('Name') != name or 'containerlab' not in (network.get('Labels') or {}):
        raise ValueError('network name or Containerlab ownership label mismatch')
    subnets={x.get('Subnet') for x in network.get('IPAM',{}).get('Config',[])}
    if subnet not in subnets or network.get('Driver') != 'bridge':
        raise ValueError('network subnet or driver mismatch')
    if network.get('Containers'):
        raise ValueError('network still has attached containers; refusing removal')
    return network['Id']


def cleanup(name, subnet):
    # List by exact name: daemon failures must not masquerade as an absent network.
    def inspect():
        listing=subprocess.run(['docker','network','ls','--format','{{.Name}}'],check=True,text=True,capture_output=True)
        if name not in listing.stdout.splitlines():return None
        result=subprocess.run(['docker','network','inspect',name],check=True,text=True,capture_output=True)
        rows=json.loads(result.stdout)
        if len(rows)!=1:raise ValueError('ambiguous management network')
        return rows[0]
    network=inspect()
    if network is None:
        print('Management network already absent: '+name);return
    network_id=validate(network,name,subnet)
    subprocess.run(['docker','network','rm',network_id],check=True)
    for _ in range(20):
        if inspect() is None:
            print('Management network released: '+name);return
        time.sleep(0.25)
    raise RuntimeError('management network remained after removal')


if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--name',required=True);parser.add_argument('--subnet',required=True)
    args=parser.parse_args();cleanup(args.name,args.subnet)
