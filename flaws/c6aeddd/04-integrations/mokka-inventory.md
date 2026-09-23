# Mokka inventory and allocation evidence

## INT-01 — The GPU identity/model predicate accepts invalid inventory

**Medium · Confirmed, reproduced.** [simulator/src/mokka.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/mokka.rs#L517) requires only the expected number of nonempty lines and that each starts with `GPU-`. It does not parse UUID/name columns, require valid/unique UUIDs, or compare model names with the selected profile.

The real `mokka-apply` CLI, with the repository's existing fake-tool harness changed to return the following two lines for a two-T4 plan, completed successfully:

```text
GPU-DUPLICATE, Not-a-T4
GPU-DUPLICATE, Not-a-T4
```

Observed: exit 0, `result.json.ok = true`, `batch-state.json.status = complete`, both invalid lines retained in `observations`. No Kubernetes or GPU execution occurred. This is a validator counterexample, not a claim that the pinned upstream emits this inventory in normal operation.

## Reproduce using the existing isolated harness

Build `--features mokka`, initialize the pinned Mokka submodule, then run this from the repository root. The harness supplies its own fake `kubectl` and `helm`; the example digest is never pulled.

```python
import pathlib, sys
repo = pathlib.Path.cwd()
sys.argv = ['probe', str(repo)]
text = (repo/'simulator/tests/mokka_cli.py').read_text()
text = text.rsplit("for case in ['mig-1g.5gb'", 1)[0]
text = text.replace('GPU-1, T4', 'GPU-DUPLICATE, Not-a-T4')
text = text.replace('GPU-2, T4', 'GPU-DUPLICATE, Not-a-T4')
text = text.replace('        else:\n            pid=int',
    "        elif case=='inventory':\n"
    "            result=json.loads((root/'state/result.json').read_text())\n"
    "            assert proc.returncode==0 and result['ok'] is True\n"
    "            values['accepted_inventory']=result['observations'][0]['gpu_inventory']\n"
    "        else:\n            pid=int")
exec(compile(text + "mokka('inventory')\n", 'isolated-probe', 'exec'))
```

**Remedy/acceptance:** parse the requested CSV fields, require unique valid identities within the intended scope, and compare the profile's declared model with an explicit allowed naming rule. Reject duplicate, malformed, missing and wrong-model rows before committing the batch; exercise rollback through the CLI. Reused mock identities across separate physical-model hosts need an explicit policy as well.

## INT-02 — The successful probe stops inside the mock agent

**High · Scope gap.** [simulator/src/mokka.rs](https://github.com/QuasarRay/datacenter-simulator/blob/c6aedddfbb9ac569b5e0eb68acc616452bf66136/simulator/src/mokka.rs#L506) runs `nvidia-smi` inside the `node-agent` container. Generated values disable NRI and allocation watching. There is no downstream test that installs/configures a device plugin, observes the intended capacity/allocatable resources, requests a GPU in a workload Pod, verifies allocation or checks release/recovery. An opted-in CPU node and Ready DaemonSet are not those allocation assertions.

The pinned chart configures NVML consumers inside the agent using its mock configuration/shims. Success there establishes that narrow contract, not that ordinary scheduled workloads see an appropriate GPU. This is consistent with the documented refusal to claim CUDA; it also leaves the broader Kubernetes GPU-resource goal unverified.

**Remedy/acceptance:** either narrow the result scope to agent-local inventory, or add an explicit allocation-contract mode on disposable nodes with configured device plugin, positive/negative Pod requests, capacity reconciliation, allocation isolation and cleanup. Keep CUDA/NCCL claims separate.
