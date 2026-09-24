"""Trainer/operator API for stage startup and separately labeled planning examples."""
import json
from pathlib import Path
import re
from mcpyrate.compiler import run
from integrations.incus.lab import Lab

class Session:
    def __init__(self, repo, state):
        self.repo=Path(repo).resolve(strict=True)
        self.state=Path(state).resolve(strict=True)

    def start(self,stage):
        if not re.fullmatch(r'(?:C|P)(?:0[1-9]|1[0-9]|20)|E(?:0[1-9]|1[0-9]|20)[ab]',stage):
            raise ValueError('unknown curriculum stage')
        report=Lab(self.state).reconcile_hosts()
        identity={'stage':stage,'deepops':report,'mastery':'not-assessed'}
        directory=self.state/'sessions';directory.mkdir(exist_ok=True)
        (directory/(stage+'.json')).write_text(json.dumps(identity,indent=2)+'\n')
        return identity

    def example(self,course):
        if not re.fullmatch(r'C(?:0[1-9]|1[0-9]|20)',course):
            raise ValueError('unknown course example')
        path=self.repo/'education/ncp-metablueprint/examples'/(course+'.py')
        module=run(path.read_text())
        return {'course':course,'tier':'planning-model','mastery':'not-assessed','module':module}
