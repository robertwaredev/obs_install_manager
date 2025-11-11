#[cfg(windows)]
use color_eyre::{Result, eyre::eyre};
use std::path::Path;

#[cfg(windows)]
use mslnk::ShellLink;

#[cfg(windows)]
pub fn create_shortcut<P: AsRef<Path>>(shortcut_path: P, target_path: P) -> Result<()> {
    ShellLink::new(target_path)?
        .create_lnk(shortcut_path)
        .map_err(|e| eyre!("Shortcut creation error: {}", e))
}
