# 18 — Repository migration and review

## Review v5

The pack is added under `cheatsheets/ex457/v5`; existing root deployment examples remain historical baseline. Its model bytes match the pinned upstream blob. The new pack supplies its own `ex457-v5` topology and management network; do not deploy it concurrently with the old fabric on the overlapping management subnet.

Review [all 48 audit dispositions](evidence/audit-closure.md), [source claims](evidence/content-to-sources.md), [blueprint coverage](00_blueprint_map.md) and [validation](evidence/validation.json). Run the standalone validation from a clean extraction. On a suitable lab host, run chapter 20 before changing any disposition to `CLOSED_RUNTIME`.

Generated private keys, client keys, configs, backups and site secrets are ignored and excluded from the release ZIP. The manifest detects accidental changes. It is an unsigned integrity record, not an authenticity or correctness signature.

[Pinned simulator model](https://github.com/QuasarRay/datacenter-simulator/blob/2003f8bdd88423c77c528785f7ffa70f398a23e0/vars/datacenter.yml).

