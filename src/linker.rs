use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::allowedlist::{AllowedList, ManageType};

/// Responsible for linking up all relevant files with their equivalent in home directory.
pub struct Linker<'a> {
    allowed_list: &'a AllowedList,
    home_dir: &'a Path,
}

impl<'a> Linker<'a> {
    /// Creates a new [Linker] instance.
    pub fn new(allowed_list: &'a AllowedList, home_dir: &'a Path) -> Self {
        Self {
            allowed_list,
            home_dir,
        }
    }

    /// Links all unlinked but managed files.
    pub fn check_and_link(&self) -> Result<()> {
        for (config_path, target) in self.find_unlinked_files() {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::os::unix::fs::symlink(config_path, &target)?;
            tracing::info!("linked {} -> {}", target.display(), config_path.display());
        }
        Ok(())
    }

    /// Returns `Linked` entries whose target path does not yet exist as a symlink.
    fn find_unlinked_files(&self) -> Vec<(&Path, PathBuf)> {
        self.allowed_list.files.iter()
            .filter_map(|(config_path, manage)| {
                let ManageType::Linked(target) = manage else { return None };
                let target = self.home_dir.join(target);

                if target.is_symlink() {
                    if std::fs::read_link(&target).ok().as_deref() != Some(config_path.as_ref()) {
                        tracing::warn!(
                            "{} is already a symlink pointing elsewhere: back it up and remove it to allow linking",
                            target.display()
                        );
                    }
                    return None;
                }

                if target.exists() {
                    tracing::warn!(
                        "{} already exists: back it up and remove it to allow linking",
                        target.display()
                    );
                    return None;
                }

                Some((config_path.as_ref(), target))
            })
            .collect()
    }
}
