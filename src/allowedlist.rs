//! Tracks which files in the config directory are explicitly managed.

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Deserialize;

/// Determines how files in the config are managed.
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ManageType {
    /// File is not used outside of config.
    None,
    /// File is linked to a target path in the home directory.
    Linked(PathBuf),
}

/// Ensures that this application only handles explicitly listed files.
pub struct AllowedList {
    /// Each managed file paired with its management type.
    pub files: Vec<(Box<Path>, ManageType)>,
}

impl AllowedList {
    /// Creates a new `AllowedList` from a vector of file paths and their management types.
    pub fn new(files: Vec<(Box<Path>, ManageType)>) -> Self {
        Self { files }
    }

    /// Check if there are top-level entries in the config dir that are not managed.
    pub fn check_all_managed(&self, config_dir: &Path) -> Result<()> {
        let ignored = [".git", ".gitignore", "config.toml", "vars"];

        for entry in std::fs::read_dir(config_dir)? {
            let path = entry?.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if ignored.contains(&name) {
                continue;
            }
            if !self.files.iter().any(|item| item.0.as_ref() == path) {
                tracing::warn!("non-managed entry {} in config directory", path.display());
            }
        }
        Ok(())
    }
}
