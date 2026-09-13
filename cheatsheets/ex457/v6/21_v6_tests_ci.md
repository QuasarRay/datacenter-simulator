# 21 — v6 regression tests and CI

## Scope of correctness

No finite pyATS or Hypothesis suite guarantees correctness of every project state, vendor release, kernel, Controller installation or exam task. v6 defines observable contracts, rejects the audited counterexamples, and makes missing live evidence visible. The [historical mapping](evidence/audit-history.md) accounts for all 103 available findings. The public blueprint remains 23 mapped objectives in eight families; mapping is content coverage, not an exam pass guarantee.

## Official-document provenance gate

`evidence/official-doc-audit.json` independently classifies all 42 evidence claims. A claim is not forced into a Red Hat or Ansible citation when it is actually a repository policy, runtime observation, FRR/Containerlab behavior or another third-party concern. For claims that are genuinely about Ansible or Red Hat products, `evidence/official-source-quotes.json` records the retrieved official URL, product/version scope, locator and a short direct quotation. `tools/documentation_contract.py` freezes the reviewed claim-to-source mapping and requires Red Hat authority for AAP/RHEL product claims.

The audit deliberately separates **documented proposition** from **runtime oracle**. `evidence/runtime-doc-contracts.json` connects selected official Ansible propositions to named live pyATS observations. For example, Ansible documents the `config` subset for `frr_facts`, but its own page says the module was tested against FRR 6.0; the live FRR 10.7 test must therefore prove `ansible_net_config` is actually returned in this lab rather than treating the documentation as a compatibility guarantee.

Hypothesis mutation tests fail if an official quote, locator, product version, approved domain, reviewed source mapping or required Red Hat authority is removed or substituted. These tests do not claim that Python can infer textual entailment; the semantic review is explicit in the checked audit, while the tests prevent a later edit from silently weakening its provenance.

## Property and stateful tests

From `cheatsheets/ex457/v6`, install `requirements.txt`, `tests/requirements.txt` and the collections in `requirements.yml`, then run:

```bash
python -m pytest -q --junitxml=.state/pytest.xml
python tools/validate.py --full --manifest
```

Hypothesis uses 150 examples per ordinary property where the strategy has sufficient distinct values, a deterministic CI profile, no timing deadline, and shrinking/reproduction blobs. The backup state machine uses 35 examples with up to 12 operations per example. These are settings, not a claimed count of distinct cases actually executed. Set `HYPOTHESIS_PROFILE=dev` for a shorter interactive run.

Independent oracles derive endpoint adjacency and kernel address sets directly from the model. Mutation tests reject wrong address families, peer groups, out-of-scope routing statements, mixed valid/invalid ECMP hops, wrong egress, management gateway conflicts, duplicate ASNs, invalid router IDs, altered topology binds, swapped inventory addresses, malformed keys, permissive versions, incomplete evidence registries and weakened official-document provenance. Stateful tests create, verify, corrupt and attempt to reuse immutable backup sets. The state recorder must replace a prior success with a fresh failed run after an injected error.

The AWX test downloads a pinned upstream method, checks its SHA-256, and executes the method with real Jinja sandboxing and synthetic database fields. This checks `tower.filename` for AWX 24.6.1; it does not simulate a whole Controller deployment and does not establish the AAP 2.6 supported CaC collection. The real Ansible bootstrap test runs network_cli variable evaluation and local trust tasks, ending before device I/O.

[Hypothesis stateful testing](https://hypothesis.readthedocs.io/en/latest/stateful.html).

## Live pyATS gate

`tests/pyats_live.py` is destructive within the dedicated `ex457-v6` lab. Run it only after preparing the lab with an ephemeral client key, building the pinned adapter and deploying it. Missing tools or nodes cause failure. It checks unlocked SSH accounts, effective public-key policy, real Ansible/libssh positive and wrong-host-key connections, configuration convergence, the documented FRR `config` facts path on live FRR 10.7, independent Linux addresses, BGP/Zebra/kernel routes, all 12 sourced ping pairs, injected stale peer/address, complete ping loss, immutable private backups, restoration, actual integrated saved files, and lifecycle reload without reapplying configuration.

Four live observations are explicitly tied to official Ansible documentation contracts: network_cli host-key checking, `frr_facts` config gathering, `cli_config` configuration push and `cli_backup` backup behavior. The official docs define the mechanism; pyATS proves the selected playbook and pinned lab actually produce the expected observable state.

The script explicitly turns the AEtest aggregate result into a process exit status. A deliberately failing subprocess regression checks this behavior. The pre-existing root pyATS script receives the same exit fix.

[Cisco AEtest execution](https://devnet-pubhub-site.s3.amazonaws.com/media/pyats/docs/aetest/run.html).

## Dagger and GitHub Actions

Run from the **repository root**, or the ZIP's `ex457-v6-project` root:

```bash
ex457_ci_run_id="local-$(date +%s)"
dagger -m ci call artifacts --source=. --run-id="$ex457_ci_run_id" export --path=artifacts
# Check the exact exported run without invoking the lab again.
python3 ci/check_results.py --run-id="$ex457_ci_run_id"
```

Dagger engine 0.21.8 installs the pinned Ansible and test dependencies, deploys/tests/destroys the original simulator, then builds the FRR 10.7.1 adapter and runs the v6 lab. Both labs use overlapping management space, so they run sequentially. Nested Docker requires a Docker-capable Linux host and Dagger's privileged execution option. The default CI runs on disposable GitHub-hosted Ubuntu 24.04 runners, with a read-only repository token and action commit pins. It does not use `pull_request_target` or expose a real Controller to untrusted pull requests.

`artifacts()` preserves diagnostics. The next Actions step checks those exact exported bytes with `ci/check_results.py`, including the run identity, source manifest, mandatory gates, every audit-mapped pytest case and the ZIP. It does not start a second lab run. Calling only `artifacts()` is not a pass gate. The standalone Dagger `ci()` function can run the suite without exporting artifacts. Every required test must pass; missing required checks fail the result. Actions uploads logs even on failure. A source ZIP is generated only after all required lab gates pass. This is the delivery portion of the pipeline; merging and deployment to an external environment are separate operator actions.

The artifact allowlist excludes client keys, host private keys, backup files and tokens. Do not upload `.state` wholesale. System package repositories and Python transitive dependencies remain external build inputs; named top-level versions and OCI base digests do not make this a fully hermetic supply chain.

[Dagger action at the pinned commit](https://github.com/dagger/dagger-for-github/tree/27b130bf0f79a7f6fbbbe0fbca6760dc9bb40a77), [GitHub secure use](https://docs.github.com/en/actions/reference/security/secure-use).

## External acceptance

An entitled EE registry, real AAP Controller, CachyOS host and rebootable RHEL VM are not provisioned by the disposable FRR pipeline. Their status is explicitly `NOT_RUN` until their chapter 20 acceptance is executed on the specified target. Local properties and Linux containers cannot prove RHEL NetworkManager persistence, AAP server schemas/RBAC or a registry pull from an execution node.

## Nested Docker resource delegation

The disposable Dagger runner moves its root cgroup processes into a child before enabling cgroup v2 subtree controllers, following [Moby's DinD initialization](https://github.com/moby/moby/blob/master/hack/dind). CPU and memory controllers must be available; missing delegation fails the gate. The original simulator's resource-limit tests stay enabled. This initialization is guarded for the Dagger runner and must not be invoked on a production host.
