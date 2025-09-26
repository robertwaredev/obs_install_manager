use color_eyre::eyre::Result;
use mslnk::ShellLink;
use std::path::Path;

#[cfg(target_os = "windows")]
pub fn create_shortcut<P: AsRef<Path>>(shortcut_path: P, target_path: P) -> Result<()> {
    Ok(ShellLink::new(target_path)?.create_lnk(shortcut_path)?)
}
