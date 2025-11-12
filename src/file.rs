use crate::app::{Event, send_progress_event};
use color_eyre::Result;
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

// #[cfg(target_os = "macos")]
use color_eyre::eyre::{WrapErr, eyre};
use plist::Value;

pub fn install_dmg(dmg_path: &str, app_name: &str) -> Result<()> {
    // Mount the DMG
    let output = Command::new("hdiutil")
        .args(["attach", dmg_path, "-nobrowse", "-quiet", "-plist"])
        .output()
        .wrap_err("Failed to mount DMG")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(eyre!("hdiutil attach failed: {}", stderr));
    }

    // Parse the plist output
    let mount_point =
        extract_mount_point_from_plist(&output.stdout).wrap_err("Failed to extract mount point")?;

    // Ensure cleanup happens even if copy fails
    let result = (|| -> Result<()> {
        let app_source = format!("{}/{}", mount_point, app_name);
        let app_dest = format!("/Applications/{}", app_name);

        // Check if source exists
        if !std::path::Path::new(&app_source).exists() {
            return Err(eyre!("App not found at: {}", app_source));
        }

        // Remove existing app if present
        if std::path::Path::new(&app_dest).exists() {
            let rm_status = Command::new("rm")
                .args(["-rf", &app_dest])
                .status()
                .wrap_err("Failed to remove existing app")?;

            if !rm_status.success() {
                return Err(eyre!("Failed to remove existing app"));
            }
        }

        // Copy the app
        let cp_output = Command::new("cp")
            .args(["-R", &app_source, &app_dest])
            .output()
            .wrap_err("Failed to execute cp command")?;

        if !cp_output.status.success() {
            let stderr = String::from_utf8_lossy(&cp_output.stderr);
            return Err(eyre!("cp command failed: {}", stderr));
        }

        Ok(())
    })();

    // Always unmount, even if copy failed
    let unmount_result = Command::new("hdiutil")
        .args(["detach", &mount_point, "-force", "-quiet"])
        .output()
        .wrap_err("Failed to unmount DMG");

    if let Err(e) = &unmount_result {
        eprintln!("Warning: Failed to unmount: {}", e);
    }

    result
}

fn extract_mount_point_from_plist(plist_data: &[u8]) -> Result<String> {
    // Parse the plist
    let value = Value::from_reader_xml(plist_data).wrap_err("Failed to parse plist")?;

    // The structure is a dictionary with "system-entities" array
    let dict = value
        .as_dictionary()
        .ok_or_else(|| eyre!("Plist root is not a dictionary"))?;

    let entities = dict
        .get("system-entities")
        .and_then(|v| v.as_array())
        .ok_or_else(|| eyre!("No system-entities array in plist"))?;

    // Find the mount point in the entities
    for entity in entities.iter() {
        if let Some(entity_dict) = entity.as_dictionary() {
            if let Some(mount_point) = entity_dict.get("mount-point") {
                if let Some(path) = mount_point.as_string() {
                    return Ok(path.to_string());
                }
            }
        }
    }

    Err(eyre!("Could not find mount point in plist"))
}
