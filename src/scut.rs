use color_eyre::eyre::{Result, eyre};
use mslnk::ShellLink;
use std::path::Path;

#[cfg(target_os = "windows")]
pub fn create_shortcut<P: AsRef<Path>>(shortcut_path: P, target_path: P) -> Result<()> {
    ShellLink::new(target_path)?
        .create_lnk(shortcut_path)
        .map_err(|e| eyre!("Shortcut creation error: {}", e))
}
