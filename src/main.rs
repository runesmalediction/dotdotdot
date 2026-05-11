//! Manages dotfiles.

use std::path::Path;

use anyhow::{Result, anyhow};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use crate::{config::Config, linker::Linker};

mod allowedlist;
mod config;
mod git;
mod linker;

fn main() {
    // Start tracing
    init_tracing();

    // Run the application and use tracing to display errors
    if let Err(e) = run() {
        tracing::error!("{}", e);
    }
}

/// Run the application.
fn run() -> Result<()> {
    // Start by getting directories
    let home_dir = get_home_dir()?;
    let config_dir = get_config_dir(&home_dir)?;

    let config = Config::retrieve(&config_dir)?;
    if config.git {
        git::ensure_gitignored(&config_dir, &["vars"])?;
        git::sync(&config_dir)?;
    }

    // Get all managed files and check that that there are no unmanaged files
    let allowed = config.into_allowed_list(&config_dir)?;
    allowed.check_all_managed(&config_dir)?;

    // Link all managed files
    let linker = Linker::new(&allowed, &home_dir);
    linker.check_and_link()?;
    Ok(())
}

/// Setup of the default tracing subscriber.
fn init_tracing() {
    let level = if cfg!(debug_assertions) { Level::TRACE } else { Level::INFO };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .without_time()
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

/// Retrieve home directory.
fn get_home_dir() -> Result<Box<Path>> {
    let home_dir = std::env::home_dir();
    if home_dir.is_none() {
        return Err(anyhow!("could not locate home directory"));
    }
    Ok(home_dir.unwrap().into_boxed_path())
}

/// Retrieves the config directory at '~/.config/dotdotdot/'.
fn get_config_dir(home_dir: &Path) -> Result<Box<Path>> {
    let mut dir = home_dir.to_path_buf();

    dir.push(".config/dotdotdot");

    // check config dir
    let e = dir.as_path().try_exists()?;
    let s = dir.display().to_string();
    if !e {
        std::fs::create_dir(dir.as_path())?;
        tracing::info!("created config directory at {s}.");
    }

    if !dir.is_dir() {
        return Err(anyhow!("path at {s} is not a directory"));
    }

    tracing::debug!("found config directory at {s}.");

    Ok(dir.into_boxed_path())
}
