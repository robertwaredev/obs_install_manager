use crate::app::{Event, send_progress_event};
use color_eyre::Result;
#[cfg(target_os = "macos")]
use color_eyre::eyre::{WrapErr, eyre};
use curl::easy::{Easy, WriteError};
use std::{
    fs,
    io::{self, Write},
    path::Path,
    process::{Command, ExitStatus},
    sync::mpsc,
};

pub fn download<P: AsRef<Path>>(
    url: &str,
    path: P,
    progress_tx: &mpsc::Sender<Event>,
) -> Result<()> {
    let mut easy = Easy::new();
    easy.url(url)?;
    easy.follow_location(true)?;
    easy.progress(true)?;
    easy.useragent("obs-install-manager-dl/1.0")?;

    let mut file = fs::File::create(path)?;
    let mut transfer = easy.transfer();

    transfer.write_function(move |data| {
        file.write_all(data).map_err(|_| WriteError::Pause)?;
        Ok(data.len())
    })?;

    transfer.progress_function(|dltotal, dlnow, _, _| {
        if dltotal > 0.0 {
            send_progress_event(dlnow / dltotal, progress_tx);
        }
        true
    })?;

    transfer.perform()?;
    send_progress_event(0.0, progress_tx);
    Ok(())
}

// TODO: Set up progress bar for extract
pub fn extract_zip<P: AsRef<Path>>(file_path: P, extract_dir: P) -> Result<()> {
    let mut archive = zip::ZipArchive::new(io::BufReader::new(fs::File::open(file_path)?))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let extract_path = extract_dir.as_ref().join(file.name());

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&extract_path)?;
        } else {
            if let Some(parent) = extract_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            io::copy(&mut file, &mut fs::File::create(&extract_path)?)?;
        }
    }

    Ok(())
}

pub fn run<P: AsRef<Path>>(path: P) -> io::Result<ExitStatus> {
    Ok(Command::new(path.as_ref().as_os_str()).spawn()?.wait()?)
}

#[cfg(target_os = "macos")]
pub fn install_dmg(dmg_path: &str, app_name: &str) -> Result<()> {
    // Mount the DMG
    let output = Command::new("hdiutil")
        .args(["attach", dmg_path, "-nobrowse", "-quiet", "-plist"])
        .output()
        .context("Failed to mount DMG")?;

    if !output.status.success() {
        return Err(eyre!("hdiutil attach failed"));
    }

    let plist_output = String::from_utf8_lossy(&output.stdout);

    // Parse mount point from plist (more reliable)
    // Might want to use a plist parser crate like `plist`
    let mount_point = extract_mount_point_from_plist(&plist_output)?;

    // Ensure cleanup happens even if copy fails
    let result = (|| -> Result<()> {
        let app_source = format!("{}/{}", mount_point, app_name);
        let app_dest = format!("/Applications/{}", app_name);

        // Remove existing app if present
        let _ = Command::new("rm").args(["-rf", &app_dest]).status();

        // Copy the app
        let status = Command::new("cp")
            .args(["-R", &app_source, &app_dest])
            .status()
            .context("Failed to copy app")?;

        if !status.success() {
            return Err(eyre!("cp command failed"));
        }

        Ok(())
    })();

    // Always unmount, even if copy failed
    Command::new("hdiutil")
        .args(["detach", &mount_point, "-force", "-quiet"])
        .status()
        .context("Failed to unmount DMG")?;

    result
}

#[cfg(target_os = "macos")]
fn extract_mount_point_from_plist(plist: &str) -> Result<String> {
    // Simple parsing - in production use the `plist` crate
    for line in plist.lines() {
        if line.contains("<key>mount-point</key>") {
            // Next line should have the value
            continue;
        }
        if line.contains("<string>/Volumes/") {
            return Ok(line
                .trim()
                .trim_start_matches("<string>")
                .trim_end_matches("</string>")
                .to_string());
        }
    }
    Err(eyre!("Could not find mount point in plist"))
}
