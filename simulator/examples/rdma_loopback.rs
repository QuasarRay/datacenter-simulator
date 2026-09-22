// SPDX-License-Identifier: RPL-1.5
// Rust simulator extension of QuasarRay/datacenter-simulator.
// Unless explicitly acquired and licensed from Licensor under another license,
// the contents of this file are subject to the Reciprocal Public License
// ("RPL") Version 1.5, or subsequent versions as allowed by the RPL, and You
// may not copy or use this file in either source code or executable form,
// except in compliance with the terms and conditions of the RPL.
// All software distributed under the RPL is provided strictly on an "AS IS"
// basis, WITHOUT WARRANTY OF ANY KIND, EITHER EXPRESS OR IMPLIED, AND LICENSOR
// HEREBY DISCLAIMS ALL SUCH WARRANTIES, INCLUDING WITHOUT LIMITATION, ANY
// WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, QUIET
// ENJOYMENT, OR NON-INFRINGEMENT. See ../license.md for the RPL's specific
// language governing rights and limitations.

fn main() -> anyhow::Result<()> {
    let selection = datacenter_simulator::rdma::RdmaDevice {
        name: std::env::var("SIMULATOR_RDMA_DEVICE")?,
        port: std::env::var("SIMULATOR_RDMA_PORT")?.parse()?,
        gid_index: std::env::var("SIMULATOR_RDMA_GID_INDEX")?.parse()?,
    };
    let result = datacenter_simulator::rdma::loopback_on(
        &selection,
        b"datacenter-rdma",
        std::time::Duration::from_secs(5),
    )?;
    println!(
        "{}",
        serde_json::json!({"ok":true,"scope":"local-rc-loopback",
        "selection":selection,"bytes":result.len(),"modeled_fabric_tested":false,"nccl_tested":false})
    );
    Ok(())
}
