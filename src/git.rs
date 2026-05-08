use std::path::Path;
use std::process::Command;

use anyhow::Result;

/// If `dir` is a git repository, sync with the remote: pull if behind, push if ahead.
pub fn sync(dir: &Path) -> Result<()> {
    // Check that this is a git directory
    if !dir.join(".git").exists() {
        return Ok(());
    }
    tracing::debug!("config directory is a git repository, checking for updates");

    commit_changes(dir)?;

    // Try `git fetch`
    let fetch = Command::new("git").args(["fetch"]).current_dir(dir).output();
    let fetch = match fetch {
        Ok(output) => output,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::debug!("git not found, skipping sync");
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };
    if !fetch.status.success() {
        tracing::warn!("git fetch failed: {}", String::from_utf8_lossy(&fetch.stderr).trim());
        return Ok(());
    }

    // Sync with upstream
    let behind = commit_count(dir, "HEAD..@{u}")?;
    let ahead = commit_count(dir, "@{u}..HEAD")?;
    match (behind, ahead) {
        (None, _) => {
            tracing::debug!("no upstream configured, skipping sync");
        }
        (Some(0), Some(0)) => {
            tracing::debug!("config repository is up to date");
        }
        (Some(behind), Some(ahead)) if behind > 0 && ahead > 0 => {
            tracing::warn!(
                "config repository has diverged ({behind} behind, {ahead} ahead): manual intervention required"
            );
        }
        (Some(behind), _) if behind > 0 => {
            tracing::info!("config repository is {behind} commit(s) behind, pulling");

            let pull = Command::new("git")
                .args(["pull", "--ff-only"])
                .current_dir(dir)
                .output()?;

            if pull.status.success() {
                tracing::info!("config repository updated successfully");
            } else {
                tracing::warn!(
                    "git pull failed: {}",
                    String::from_utf8_lossy(&pull.stderr).trim()
                );
            }
        }
        (Some(0), Some(ahead)) if ahead > 0 => {
            tracing::info!("config repository is {ahead} commit(s) ahead, pushing");

            let push = Command::new("git").args(["push"]).current_dir(dir).output()?;

            if push.status.success() {
                tracing::info!("config repository pushed successfully");
            } else {
                tracing::warn!(
                    "git push failed: {}",
                    String::from_utf8_lossy(&push.stderr).trim()
                );
            }
        }
        _ => {}
    }

    Ok(())
}

/// Stage all changes and commit them if the working tree is dirty.
fn commit_changes(dir: &Path) -> Result<()> {
    let status = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(dir)
        .output()?;

    if status.stdout.is_empty() {
        return Ok(());
    }

    Command::new("git").args(["add", "-A"]).current_dir(dir).output()?;

    let commit = Command::new("git")
        .args(["commit", "-m", "auto: update dotfiles"])
        .current_dir(dir)
        .output()?;

    if commit.status.success() {
        tracing::info!("committed local changes");
    } else {
        tracing::warn!(
            "git commit failed: {}",
            String::from_utf8_lossy(&commit.stderr).trim()
        );
    }

    Ok(())
}

/// Returns the number of commits in `range`, or `None` if the range is invalid (e.g. no upstream).
fn commit_count(dir: &Path, range: &str) -> Result<Option<u32>> {
    let output = Command::new("git")
        .args(["rev-list", range, "--count"])
        .current_dir(dir)
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let count = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .unwrap_or(0);

    Ok(Some(count))
}
