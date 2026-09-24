use ncp_assessment_kernel::advance;
use ncp_grade::{Artifact, Bundle, Request, Station, accepted, evaluate, validate_bank};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{self, Read, Write},
    path::PathBuf,
};

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Ledger {
    next: u16,
    attempts: Vec<u64>,
}
struct Lock(PathBuf);
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn record_progress(
    root: &std::path::Path,
    q: &Request,
) -> Result<bool, Box<dyn std::error::Error>> {
    let lock_path = root.join("progress.lock");
    let mut lock = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path)?;
    let _guard = Lock(lock_path);
    writeln!(lock, "pid={} attempt={}", std::process::id(), q.attempt)?;
    lock.sync_all()?;
    let path = root.join("progress.json");
    let mut ledger: Ledger = match read_limited(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes)?,
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ledger::default(),
        Err(e) => return Err(e.into()),
    };
    if ledger.next > 40
        || ledger.attempts.len() != usize::from(ledger.next)
        || ledger.attempts.contains(&0)
    {
        return Err("invalid trainer progression ledger".into());
    }
    if q.exercise > ledger.next {
        return Ok(false);
    }
    if q.exercise < ledger.next {
        return Ok(true);
    } // a valid retake cannot add credit
    let next = advance(ledger.next, q.exercise, 40, true);
    if next == ledger.next {
        return Ok(false);
    }
    ledger.next = next;
    ledger.attempts.push(q.attempt);
    let temporary = root.join(format!("progress-{}.tmp", q.attempt));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    file.write_all(&serde_json::to_vec(&ledger)?)?;
    file.sync_all()?;
    fs::rename(temporary, path)?;
    fs::File::open(root)?.sync_all()?;
    Ok(true)
}

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
    let mut receipt = evaluate(s, &q, bundle.as_ref(), |hash, scope, case| {
        if hash.len() != 64
            || !hash
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return false;
        }
        read_limited(&root.join("objects").join(hash)).is_ok_and(|bytes| {
            format!("{:x}", Sha256::digest(&bytes)) == hash
                && serde_json::from_slice::<Artifact>(&bytes)
                    .is_ok_and(|a| a.matches(&q, scope, case))
        })
    });
    if accepted(&receipt, s) && !record_progress(&root, &q)? {
        receipt.ops = 1;
        receipt.feedback =
            "An earlier integrated station is incomplete; no out-of-order credit.".into();
    }
    println!("{}", serde_json::to_string(&receipt)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn durable_progress_rejects_skips_and_does_not_double_count_retakes() {
        let root = std::env::temp_dir().join(format!("ncp-ledger-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let mut q = Request {
            protocol: "ncp-grade-v1".into(),
            name: "e01a".into(),
            exercise: 1,
            attempt: 10,
        };
        assert!(!record_progress(&root, &q).unwrap());
        q.exercise = 0;
        assert!(record_progress(&root, &q).unwrap());
        q.attempt = 11;
        assert!(record_progress(&root, &q).unwrap());
        let ledger: Ledger =
            serde_json::from_slice(&fs::read(root.join("progress.json")).unwrap()).unwrap();
        assert_eq!(ledger.next, 1);
        assert_eq!(ledger.attempts, vec![10]);
        fs::remove_dir_all(root).unwrap();
    }
}
