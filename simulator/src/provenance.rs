// SPDX-License-Identifier: RPL-1.5
//! Validate every file consumed from a pinned upstream checkout.
use anyhow::{Context, Result, ensure};
use std::{path::Path, process::Command};
pub(crate) fn git(directory: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(directory)
        .args(args)
        .output()?;
    ensure!(
        output.status.success(),
        "git source validation failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).context("git output is not UTF-8")
}
pub(crate) fn verify(directory: &Path, revision: &str, consumed: &str) -> Result<()> {
    ensure!(
        git(directory, &["rev-parse", "HEAD"])?.trim() == revision,
        "upstream revision mismatch"
    );
    git(directory, &["diff", "--exit-code", "HEAD", "--", consumed])?;
    for options in [
        vec!["ls-files", "--others", "--exclude-standard", "--", consumed],
        vec![
            "ls-files",
            "--others",
            "--ignored",
            "--exclude-standard",
            "--",
            consumed,
        ],
    ] {
        ensure!(
            git(directory, &options)?.is_empty(),
            "untracked or ignored files in consumed upstream path {consumed}"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_tracked_untracked_and_ignored_source_changes() {
        let dir = std::env::temp_dir().join(format!("source-check-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        git(&dir, &["init", "-q"]).unwrap();
        std::fs::write(dir.join("profile"), "original").unwrap();
        std::fs::write(dir.join(".gitignore"), "ignored\n").unwrap();
        git(&dir, &["add", "."]).unwrap();
        git(
            &dir,
            &[
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.invalid",
                "commit",
                "-qm",
                "fixture",
            ],
        )
        .unwrap();
        let revision = git(&dir, &["rev-parse", "HEAD"]).unwrap();
        verify(&dir, revision.trim(), ".").unwrap();
        for name in ["extra", "ignored"] {
            std::fs::write(dir.join(name), "injected").unwrap();
            assert!(verify(&dir, revision.trim(), ".").is_err());
            std::fs::remove_file(dir.join(name)).unwrap();
        }
        std::fs::write(dir.join("profile"), "modified").unwrap();
        assert!(verify(&dir, revision.trim(), ".").is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
