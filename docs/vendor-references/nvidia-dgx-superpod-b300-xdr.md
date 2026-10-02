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

## Repository policy

The PDF is **not vendored into this public repository**. NVIDIA's notice states
that reproduction requires advance written approval from NVIDIA. Keep the
authoritative URL above as the source of record.

The GitHub Actions workflow
`.github/workflows/nvidia-reference.yml` downloads the official PDF only into
ephemeral CI storage and verifies that the source still resolves to a plausible
PDF. The workflow does not publish, cache, attach, or commit the NVIDIA document.
