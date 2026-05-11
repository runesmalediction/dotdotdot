//! Parses `config.toml` and builds the list of managed files.

use std::path::Path;

use anyhow::Result;
use serde::Deserialize;

use crate::allowedlist::{AllowedList, ManageType};

/// A single entry in the `[[managed]]` array of `config.toml`.
#[derive(Deserialize)]
struct FileEntry {
    /// Path to the file relative to the config directory.
    path: String,
    /// How the file is managed (linked or none).
    manage: ManageType,
    /// Names of files in `vars/` that this entry depends on.
    #[serde(default)]
    requires: Vec<String>,
}

/// Parsed representation of `config.toml`.
#[derive(Deserialize)]
pub struct Config {
    /// Whether to commit, pull, and push the config directory on each run.
    #[serde(default)]
    pub git: bool,
    /// All entries listed under `[[managed]]`.
    #[serde(default)]
    managed: Vec<FileEntry>,
}

impl Config {
    /// Retrieve the config from 'config_dir/config.toml', creating it if absent.
    pub fn retrieve(config_dir: &Path) -> Result<Self> {
        let config_path = config_dir.join("config.toml");
        let s = config_path.display().to_string();

        if !config_path.try_exists()? {
            std::fs::write(&config_path, "")?;
            tracing::info!("created config file at {s}");
        }

        let contents = std::fs::read_to_string(&config_path)?;
        let config: Self = toml::from_str(&contents)?;

        tracing::debug!("loaded config from {s}");

        Ok(config)
    }

    /// Build an [`AllowedList`] from the entries declared in this config.
    pub fn into_allowed_list(self, config_dir: &Path) -> Result<AllowedList> {
        let vars_dir = config_dir.join("vars");

        // check vars
        let mut required: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for entry in self.managed.iter() {
            for name in &entry.requires {
                required.insert(name);
                if !vars_dir.join(name).try_exists()? {
                    tracing::warn!("required var file {name} not found in vars/");
                }
            }
        }
        if vars_dir.try_exists()? {
            for entry in std::fs::read_dir(&vars_dir)? {
                let name = entry?.file_name();
                let name = name.to_string_lossy();
                if !required.contains(name.as_ref() as &str) {
                    tracing::warn!("var file {name} in vars/ is not required by any entry");
                }
            }
        }

        // build allowed list from managed entries
        let mut files = vec![];
        for entry in self.managed {
            let path = config_dir.join(&entry.path);
            if !path.try_exists()? {
                tracing::warn!("non-existent file {} in config", path.display());
            } else {
                files.push((path.into_boxed_path(), entry.manage));
            }
        }
        Ok(AllowedList::new(files))
    }
}
