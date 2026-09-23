// SPDX-License-Identifier: RPL-1.5
//! Observable file identity, not remote attestation of a trusted host.
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FileIdentity {
    pub path: PathBuf,
    pub sha256: String,
}
impl FileIdentity {
    pub fn read(path: impl AsRef<Path>) -> Result<Self> {
        let path = fs::canonicalize(path)?;
        let mut file = fs::File::open(&path)?;
        if !file.metadata()?.is_file() {
            return Err(Error::Invalid("identity requires a regular file".into()));
        }
        let mut hash = Sha256::new();
        let mut buffer = [0u8; 65536];
        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hash.update(&buffer[..n]);
        }
        Ok(Self {
            path,
            sha256: format!("{:x}", hash.finalize()),
        })
    }
    pub fn verify(&self) -> Result<()> {
        if &Self::read(&self.path)? != self {
            return Err(Error::Conflict(format!(
                "file identity differs: {}",
                self.path.display()
            )));
        }
        Ok(())
    }
}
pub fn resolve(program: impl AsRef<Path>) -> Result<PathBuf> {
    use std::os::unix::fs::PermissionsExt;
    let program = program.as_ref();
    let paths: Vec<_> = if program.components().count() > 1 || program.is_absolute() {
        vec![program.to_owned()]
    } else {
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
            .map(|p| p.join(program))
            .collect()
    };
    paths
        .into_iter()
        .find(|p| fs::metadata(p).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0))
        .ok_or_else(|| Error::NotFound(format!("executable {}", program.display())))?
        .canonicalize()
        .map_err(Into::into)
}
pub fn tools(programs: &[&Path]) -> Result<BTreeMap<String, FileIdentity>> {
    programs
        .iter()
        .map(|p| Ok((p.display().to_string(), FileIdentity::read(resolve(p)?)?)))
        .collect()
}
pub fn reject_loader_injection() -> Result<()> {
    for name in ["LD_PRELOAD", "LD_AUDIT", "DYLD_INSERT_LIBRARIES"] {
        if std::env::var_os(name).is_some_and(|v| !v.is_empty()) {
            return Err(Error::Invalid(format!(
                "native evidence rejects {name}; start from a trusted clean launcher"
            )));
        }
    }
    Ok(())
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NcclLibraries {
    pub nccl: FileIdentity,
    pub cuda: FileIdentity,
}
impl NcclLibraries {
    pub fn verify(&self) -> Result<()> {
        self.nccl.verify()?;
        self.cuda.verify()?;
        Ok(())
    }
    /// Loaded mappings are observed by the supervisor, not asserted by JSON workers.
    pub fn verify_mapped(&self, pid: u32, include_cuda: bool) -> Result<()> {
        let maps = fs::read_to_string(format!("/proc/{pid}/maps"))?;
        for identity in std::iter::once(&self.nccl).chain(include_cuda.then_some(&self.cuda)) {
            let expected = identity
                .path
                .to_str()
                .ok_or_else(|| Error::Invalid("library path is not UTF-8".into()))?;
            if !maps.lines().any(|line| {
                line.split_whitespace()
                    .skip(5)
                    .collect::<Vec<_>>()
                    .join(" ")
                    == expected
            }) {
                return Err(Error::Conflict(format!(
                    "expected library is not mapped: {expected}"
                )));
            }
            identity.verify()?;
        }
        Ok(())
    }
}
