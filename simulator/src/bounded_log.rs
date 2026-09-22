// SPDX-License-Identifier: RPL-1.5
//! Drain subprocess output continuously, retaining a bounded prefix on disk.
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
    process::Stdio,
};
const LIMIT: usize = 16 * 1024 * 1024;
const MARKER: &[u8] = b"\n[simulator: log truncated at 16 MiB; remaining bytes discarded]\n";
pub(crate) struct Completion(std::sync::mpsc::Receiver<std::io::Result<()>>);
impl Completion {
    pub(crate) fn finish(self) -> std::io::Result<()> {
        self.0
            .recv_timeout(std::time::Duration::from_secs(2))
            .map_err(|e| std::io::Error::other(format!("log drain did not finish: {e}")))?
    }
}
#[cfg(feature = "vm")]
pub(crate) fn output(path: &Path) -> std::io::Result<Stdio> {
    capture(path).map(|(stdio, _)| stdio)
}
pub(crate) fn capture(path: &Path) -> std::io::Result<(Stdio, Completion)> {
    let file = File::create(path)?;
    let (reader, writer) = std::io::pipe()?;
    let (done, completion) = std::sync::mpsc::channel();
    std::thread::Builder::new()
        .name("bounded-subprocess-log".into())
        .spawn(move || {
            let _ = done.send(collect(reader, file, LIMIT));
        })?;
    Ok((Stdio::from(writer), Completion(completion)))
}
fn collect(mut reader: impl Read, mut file: impl Write, limit: usize) -> std::io::Result<()> {
    let mut buffer = [0u8; 8192];
    let mut retained = 0;
    let mut truncated = false;
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        let keep = count.min(limit.saturating_sub(MARKER.len()).saturating_sub(retained));
        file.write_all(&buffer[..keep])?;
        retained += keep;
        if keep < count && !truncated {
            file.write_all(MARKER)?;
            truncated = true;
        }
        file.flush()?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    #[test]
    fn noisy_child_output_is_drained_with_a_bounded_file_and_marker() {
        let mut result = Vec::new();
        super::collect(std::io::repeat(b'x').take(100_000), &mut result, 1024).unwrap();
        assert_eq!(result.len(), 1024);
        assert!(result.ends_with(super::MARKER));
    }
    use std::io::Read;
}
