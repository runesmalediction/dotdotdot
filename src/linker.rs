//! Creates symlinks from config files to their targets in the home directory.

use std::path::Path;

use anyhow::Result;

use crate::allowedlist::{AllowedList, ManageType};

/// Responsible for linking managed files to their targets in the home directory.
pub struct Linker<'a> {
    /// The list of files to link, as declared in the config.
    allowed_list: &'a AllowedList,
    /// The user's home directory, used as the root for symlink targets.
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
        for (config_path, manage) in &self.allowed_list.files {
            match manage {
                ManageType::None => {}
                ManageType::Linked(target_rel) => {
                    let target = self.home_dir.join(target_rel);
                    self.link_if_needed(config_path, &target)?;
                }
            }
        }
        Ok(())
    }

    /// Creates a symlink at `target` pointing to `source`, warning if the target already exists.
    fn link_if_needed(&self, source: &Path, target: &Path) -> Result<()> {
        if target.is_symlink() {
            if std::fs::read_link(target).ok().as_deref() != Some(source) {
                tracing::warn!(
                    "{} is already a symlink pointing elsewhere: back it up and remove it to allow linking",
                    target.display()
                );
            }
            return Ok(());
        }

        if target.exists() {
            tracing::warn!(
                "{} already exists: back it up and remove it to allow linking",
                target.display()
            );
            return Ok(());
        }

        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::os::unix::fs::symlink(source, target)?;
        tracing::info!("linked {} -> {}", target.display(), source.display());
        Ok(())
    }
}
