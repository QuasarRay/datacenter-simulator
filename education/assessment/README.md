# Trainer evidence gate

`ncp-grade` consumes a Rustlings `ncp-grade-v1` request on stdin and one absolute
trainer evidence directory as its argument. It reads `STATION/ATTEMPT.json` and
raw content-addressed observation envelopes under `objects/SHA256`. Each envelope
binds its nonempty raw output to the exact request, facet/intervention and case.
Missing live evidence
returns blocked statuses. A synthetic fixture never completes the blueprint.

The trainer collector must run for the requested attempt **before** calling this
gate. It must collect every facet in a repaired baseline and an independent
holdout. Three additional intervention traces must establish that removing the
AIO, AIN or AII decision separately breaks the coupled workload, and restoring
it recovers. A fresh pinned DeepOps execution report is also required.

The compiled bank gives each facet its exact required execution tier. The final
receipt has product-assessment tier 4; this is a label for the aggregate contract,
not a claim that Linux emulation becomes NVIDIA hardware. Facet tiers remain
separately checked. Open-source equivalence cannot satisfy a licensed product
facet. Docker-specific course observations can be supplied by an external course
assessor; no Docker runner is invoked by an exercise.

Never give the learner write access to this directory, the grader executable,
its configuration or collector credentials. Hashes establish content integrity,
not who produced an observation. The collector is a trusted boundary; this code
cannot determine the truth of a fabricated observation. A same-identity local
demonstration is practice only. The repository is public; black-box delivery
requires exporting only learner material and keeping deployment-specific faults
and observations on the trainer host.

Validation mutates **every** facet in both cases of all forty stations, rejects
missing/false/wrong-tier records, stale attempts and missing coupling/provenance.
These are executable regression checks. Formal proofs cover the imported shared
decision kernel; the parser, artifact store, bank authoring and observation
semantics have not been formally proved. See the companion Rustlings proof scope.

Live collector implementations for every NVIDIA product are a release gate.
Until they and the qualified host exist, this bank can specify and reject
incomplete submissions but cannot award full live mastery. Do not replace a
missing collector with a constant passing result.

## Isolated delivery

`bridge.py` is a trainer-owned Unix socket service. It accepts only the forty
known station IDs and validates the peer's host UID with `SO_PEERCRED`. Its
profile selects the fixed collector, interpreter, grader and lab; no client
field can choose a command or path. The collector source must match its pinned
SHA-256 and the profile must have completed live qualification. The example
profile intentionally stays `unqualified` and has no fictitious collector.

The bridge freshly runs the real DeepOps host role, then invokes the qualified
site collector with `{request, state}` on stdin. The collector returns exactly
`observations` and `interventions`, with the Bundle fields from `src/lib.rs`
except that each `artifact` is replaced by a nonempty `raw` string. The bridge
wraps, hashes and stores those observations before invoking the Rust grader.
Collection is not allowed to promote model/emulated output into product tier.
Hashes are not signatures; truthful product collection remains a trusted,
independently qualified boundary.

Run one service/profile per learner. Give its evidence directory mode 0700,
keep the collector/configuration outside learner access, and set the exact host
UID/GID corresponding to the Incus learner identity. The socket parent must be
traversable by that group. Mount only this restricted assessment socket into the
learner context, never the Incus daemon socket. Qualify the UID mapping and socket
permissions on the actual host before assessment. A disconnected or malformed
client does not stop the service. An unqualified/failed collector returns blocked.

Point Rustlings' `NCP_GRADER_CONFIG` at Python plus `proxy.py SOCKET`, with a
480-second timeout and the compiled station masks. The proxy can request an
assessment; it cannot write the trainer store. A learner can alter their local
UI or bypass the proxy, but that cannot advance the authoritative trainer ledger.
Use the release Rustlings binary for community workspaces; debug builds use the
upstream development manifest. With external assessment enabled, edits to
`controllers/e01a.py` trigger the corresponding Rustlings watch event.

Successful stations update `progress.json` through the shared proved `advance`
function. The trainer serializes writes, fsyncs a temporary ledger, renames it
atomically and syncs the directory. Skips are refused and valid retakes add no
credit. `next / 2` is the number of complete incident pairs; only `next == 40`
establishes all station contracts. After a crash, inspect a retained
`progress.lock` and temporary file before operator reconciliation; the service
never deletes an unknown lock automatically. Disk/OS behavior is tested, not
formally proved. Course/project prerequisites additionally need the instructor's
accepted learning portfolio; this ledger tracks independent stations only.
