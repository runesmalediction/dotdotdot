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

/// Ensure that this application only handles certain allowed files by explicetly listing them.
pub struct AllowedList {
    pub files: Vec<(Box<Path>, ManageType)>,
}

impl AllowedList {
    /// Creates a new `AllowedList` from a vector of file paths and their management types.
    pub fn new(files: Vec<(Box<Path>, ManageType)>) -> Self {
        Self { files }
    }

    /// Check if there are files in the config dir that are not managed.
    pub fn check_all_managed(&self, config_dir: &Path) -> Result<()> {
        crate::visit_dirs(config_dir, &|entry| {
            if !self
                .files
                .iter()
                .any(|item| item.0.as_ref() == entry.path().as_path())
            {
                let s = entry.path().as_path().display().to_string();
                tracing::warn!("non-managed file {s} in config directory")
            }
        })?;

        Ok(())
    }
}
