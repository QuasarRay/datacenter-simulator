# NVIDIA DGX SuperPOD B300 / Quantum-X800 reference architecture

Authoritative external reference for the simulator's NVIDIA GPU-datacenter architecture work.

## Official source

- **Title:** NVIDIA SuperPOD with DGX B300 Systems, NVIDIA Quantum-X800 InfiniBand switching and AC Power Reference Architecture
- **Document:** RA-11339-001 V01
- **Publication date:** 2025-07-23
- **Official PDF:** https://docs.nvidia.com/dgx-superpod/reference-architecture/scalable-infrastructure-b300-xdr/latest/_downloads/f2d6423ee942e3f144511f5df58e3080/RA11339001-DSPB300-XDR-ReferenceArch.pdf
- **Official HTML:** https://docs.nvidia.com/dgx-superpod/reference-architecture/scalable-infrastructure-b300-xdr/latest/

## Why this reference matters

Use this document as the high-level architectural spine for NVIDIA-oriented
simulation, exercises, and NCP-Metablueprint material. Its useful architectural
coverage includes:

- scalable-unit system design;
- DGX B300 compute nodes;
- Quantum-X800 InfiniBand compute fabric;
- Ethernet and InfiniBand storage fabrics;
- in-band and out-of-band management networks;
- storage architecture and connectivity;
- management-server placement and connectivity;
- NVIDIA Mission Control;
- NVIDIA Base Command Manager;
- NVIDIA Run:ai;
- NVIDIA NGC and NVIDIA AI Enterprise.

For concept-first study, prioritize chapters 1-7. Chapter 8 is mostly a summary,
chapter 9 is primarily a representative bill of materials, and chapter 10 contains
notices and legal terms.

## Public repository policy

The PDF is **not committed to this public repository**. NVIDIA's notice states
that reproduction requires advance written approval from NVIDIA. Keep the
authoritative NVIDIA URL above as the public source of record.

The workflow `.github/workflows/nvidia-reference.yml` downloads the official
PDF only into ephemeral CI storage and validates that the source still resolves
to a plausible PDF. It does not publish, cache, attach, or commit the document.

## Owner-only GitHub copy

For personal retrieval through GitHub without exposing the PDF through this
public repository, `.github/workflows/private-nvidia-reference.yml` publishes
the PDF as an **unlinked private OCI artifact** in GitHub Container Registry:

`ghcr.io/quasarray/private-nvidia-superpod-b300-xdr:ra-11339-001-v01`

The workflow deliberately authenticates with the repository secret
`PRIVATE_DOCS_GHCR_TOKEN` rather than `GITHUB_TOKEN`. The secret must contain
a classic personal access token owned by `QuasarRay` with at least
`read:packages` and `write:packages`. Because the package is pushed with a
personal token and no repository-source link, a newly created package remains
private and does not inherit this public repository's read permissions.

After the package exists, the owner can retrieve the original PDF with ORAS:

```sh
printf '%s' "$GHCR_READ_TOKEN" | oras login ghcr.io -u QuasarRay --password-stdin
oras pull ghcr.io/quasarray/private-nvidia-superpod-b300-xdr:ra-11339-001-v01
```

Keep package visibility set to **Private**, leave repository permission
inheritance disabled, and do not grant package read access to other users or
repositories. The publishing workflow finishes by verifying that an anonymous
manifest fetch is denied.
