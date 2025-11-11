use color_eyre::{Result, eyre::eyre};
use mslnk::ShellLink;
use std::path::Path;

pub fn create_shortcut<P: AsRef<Path>>(shortcut_path: P, target_path: P) -> Result<()> {
    ShellLink::new(target_path)?
        .create_lnk(shortcut_path)
        .map_err(|e| eyre!("Shortcut creation error: {}", e))
}
