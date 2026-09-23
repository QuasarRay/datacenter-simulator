# VAL-03 — Fault evidence needs a cause, not just a failed command

**Medium · Root predicate reproduced; DeepOps causal limitation from source.**

[tests/datacenter_state.py](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/tests/datacenter_state.py#L284) accepts any nonzero return code from the partitioned ping as evidence that traffic cannot bypass the cut. It does not distinguish packet loss from a failed Docker exec, missing ping executable, signal termination or invalid invocation. Positive probes before/after reduce the risk but do not prove what failed during the cut.

A local AST-isolated invocation of the actual test method used successful stubs for the positive commands and returned `127, "ping: not found"` for the negative probe. The method returned normally. No Docker/network commands were executed. This is an oracle counterexample, not the explanation for VAL-01 (which failed its recovery ping).

Reproduce from the repository root:

```python
import ast, ipaddress, pathlib, types
path = pathlib.Path('tests/datacenter_state.py')
node = next(n for n in ast.parse(path.read_text()).body
            if isinstance(n, ast.ClassDef) and n.name == 'FabricTraffic')
class Case:
    def failed(self, text): raise AssertionError(text)
scope = {'aetest': types.SimpleNamespace(Testcase=Case, test=lambda f: f),
    'ipaddress': ipaddress, 'container_name': lambda lab, n: lab+'-'+n,
    'command': lambda argv: None,
    'subprocess': types.SimpleNamespace(run=lambda *a, **k:
        types.SimpleNamespace(returncode=127, stderr=b'ping: not found'))}
exec(compile(ast.Module(body=[node], type_ignores=[]), str(path), 'exec'), scope)
scope['FabricTraffic']().direct_delivery_partition_and_recovery({'links': [
    {'name': 'wire', 'endpoints': [
        {'node': 'a', 'interface': 'eth1', 'address': '10.0.0.0/31'},
        {'node': 'b', 'interface': 'eth1', 'address': '10.0.0.1/31'}]}]}, 'fixture')
print('Returned normally despite negative probe execution failure')
```

The more sophisticated [simulator/src/deepops_runtime.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/deepops_runtime.rs#L534) waits for a genuine correctness row before cutting GPU-node links, then accepts a non-successful `srun` exit. It restores links and runs recovery collectives, which is valuable. However, SSH/Slurm and NCCL share the modeled fabric in this setup. A control-plane loss or unrelated rank failure can satisfy the exit predicate; the test does not isolate the NCCL transport as the cause. No such hardware false pass was demonstrated locally.

**Remedy/acceptance:** record separate tool-execution and traffic outcomes, expected status/error classes, cut acknowledgements and relevant counters. Keep management/control connectivity independent when claiming a data-plane-specific NCCL fault, or narrow the claim to whole-node network isolation. Inject missing binaries, Docker failures, unrelated rank exits and partial cuts; none should certify a successful transport-failure test. Continue requiring before/after positive execution.
