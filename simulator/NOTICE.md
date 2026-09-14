# Source and license notices

This Rust simulator is an extension of QuasarRay/datacenter-simulator. Its new simulator code and documentation use the repository's [Reciprocal Public License 1.5](../license.md). The simulator adds local lifecycle/topology/collective/verbs models and native integration modules.

The [NVIDIA Air SDK](https://github.com/QuasarRay/nv-air-sdk/tree/0a9ce86a6195e26c5522640d895a1adba9449cbd) is the specification source; its code is not executed or embedded. The small JSON-envelope test is based on its MIT-licensed documented export example. SDK copyright: NVIDIA CORPORATION & AFFILIATES.

Upstream implementations remain separate pinned submodules with their original notices and licenses:

- NCCL: NVIDIA CORPORATION & AFFILIATES, Apache-2.0; see vendor/nccl/LICENSE.txt and ThirdPartyNotices.txt.
- rust-ibverbs: original contributors, MIT OR Apache-2.0; see vendor/rust-ibverbs/LICENSE-MIT and LICENSE-APACHE.
- patchbay: original contributors, MIT OR Apache-2.0; see vendor/patchbay/LICENSE-MIT and LICENSE-APACHE.
- petgraph: original contributors, MIT OR Apache-2.0; see vendor/petgraph/LICENSE-MIT and LICENSE-APACHE.

The ring model is an independent Rust semantic implementation of the NCCL reduce-scatter/all-gather schedule, with original source attribution in docs/design.md. No upstream license files are modified.
