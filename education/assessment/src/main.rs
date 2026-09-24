use ncp_grade::{Bundle, Request, Station, evaluate, validate_bank};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
};

fn read_limited(path: &std::path::Path) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(4 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err(io::Error::other("evidence too large"));
    }
    Ok(bytes)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 2 {
        return Err("usage: ncp-grade ABSOLUTE_TRAINER_EVIDENCE_DIRECTORY".into());
    }
    let root = PathBuf::from(&args[1]);
    if !root.is_absolute() {
        return Err("trainer evidence directory must be absolute".into());
    }
    let bank: Vec<Station> = serde_json::from_str(include_str!("../bank.json"))?;
    validate_bank(&bank)?;
    let mut input = String::new();
    io::stdin().take(32769).read_to_string(&mut input)?;
    if input.len() > 32768 {
        return Err("request too large".into());
    }
    let q: Request = serde_json::from_str(&input)?;
    let s = bank
        .iter()
        .find(|s| s.name == q.name && s.id == q.exercise)
        .ok_or("unknown station")?;
    let path = root.join(&s.name).join(format!("{}.json", q.attempt));
    let bundle: Option<Bundle> = match read_limited(&path) {
        Ok(bytes) => Some(serde_json::from_slice(&bytes)?),
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => return Err(e.into()),
    };
    let receipt = evaluate(s, &q, bundle.as_ref(), |hash| {
        if hash.len() != 64
            || !hash
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return false;
        }
        read_limited(&root.join("objects").join(hash))
            .is_ok_and(|bytes| !bytes.is_empty() && format!("{:x}", Sha256::digest(bytes)) == hash)
    });
    println!("{}", serde_json::to_string(&receipt)?);
    Ok(())
}
