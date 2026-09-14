fn main() -> anyhow::Result<()> {
    let result = datacenter_simulator::rdma::loopback(
        b"datacenter-rdma",
        std::time::Duration::from_secs(5),
    )?;
    println!("RDMA payload verified: {} bytes", result.len());
    Ok(())
}
