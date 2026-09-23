// SPDX-License-Identifier: RPL-1.5
//! File limits apply before deserialization, including non-regular inputs.
use std::{
    io::{self, Read},
    path::Path,
};
pub const CONFIG_LIMIT: usize = 1024 * 1024;
pub const SNAPSHOT_LIMIT: usize = 64 * 1024 * 1024;
pub fn read(path: impl AsRef<Path>, limit: usize) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("input exceeds {limit} bytes"),
        ));
    }
    Ok(bytes)
}
