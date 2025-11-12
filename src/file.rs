use crate::app::{Event, send_progress_event};
use color_eyre::Result;
use curl::easy::{Easy, WriteError};
use std::{
    fs,
    io::{self, Write},
    path::Path,
    process::{Command, ExitStatus},
    sync::mpsc,
    thread,
    time::Duration,
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
use color_eyre::eyre::eyre;

pub fn install_dmg(dmg_path: &str, app_name: &str, mount_tag: &str) -> Result<()> {
    // Open the DMG (macOS will mount it automatically)
    Command::new("open").arg(dmg_path).status()?;

    // Wait for the volume to appear with retry logic
    let mount_point = wait_for_mount(mount_tag, 30)?;
    let app_source = format!("{}/{}", mount_point, app_name);

    // Verify the app exists in the mounted volume
    if !Path::new(&app_source).exists() {
        return Err(eyre!("App not found at: {}", app_source));
    }

    // Copy the app
    let cp_result = Command::new("cp")
        .args(["-R", &app_source, "/Applications/"])
        .status()?;

    if !cp_result.success() {
        return Err(eyre!("Failed to copy app"));
    }

    // Eject the volume
    Command::new("hdiutil")
        .args(["detach", &mount_point])
        .status()?;

    Ok(())
}

fn wait_for_mount(mount_tag: &str, max_attempts: u32) -> Result<String> {
    for attempt in 0..max_attempts {
        // List volumes
        let output = Command::new("ls").arg("/Volumes/").output()?;
        let volumes = String::from_utf8_lossy(&output.stdout);

        // Look for a volume that matches the app name
        if let Some(volume) = volumes.lines().find(|line| line.contains(mount_tag)) {
            let mount_point = format!("/Volumes/{}", volume.trim());

            // Verify it's actually mounted and accessible
            if Path::new(&mount_point).exists() {
                return Ok(mount_point);
            }
        }

        // Exponential backoff: 100ms, 200ms, 400ms, 800ms, then cap at 1s
        let delay = Duration::from_millis(100 * 2u64.pow(attempt.min(3)));
        thread::sleep(delay);
    }

    Err(eyre!(
        "Timed out waiting for DMG to mount. Expected volume containing '{}'",
        mount_tag
    ))
}
