# Trainer evidence gate

`ncp-grade` consumes a Rustlings `ncp-grade-v1` request on stdin and one absolute
trainer evidence directory as its argument. It reads `STATION/ATTEMPT.json` and
raw content-addressed observations under `objects/SHA256`. Missing live evidence
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
