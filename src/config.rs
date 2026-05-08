use std::path::Path;

use anyhow::Result;
use serde::Deserialize;

use crate::allowedlist::{AllowedList, ManageType};

#[derive(Deserialize)]
struct FileEntry {
    path: String,
    manage: ManageType,
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(default)]
    pub git: bool,
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
