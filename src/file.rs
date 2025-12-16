use crate::app::{Event, send_progress_event};
use color_eyre::{Result, eyre::eyre};
use curl::easy::{Easy, WriteError};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
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

pub fn copy_dir(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    if !src.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "source is not a directory",
        ));
    }

    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path: PathBuf = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir(&src_path, &dst_path)?;
        } else if file_type.is_file() {
            fs::copy(&src_path, &dst_path)?;
        }
    }

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

pub fn install_dmg(dmg_path: &str, mount_tag: &str) -> Result<()> {
    Command::new("open").arg(dmg_path).status()?;

    let mount_point = wait_for_mount(mount_tag, 30)?;
    let app_name = find_app(&mount_point)?;
    let app_src = format!("{}/{}", mount_point, app_name);
    let app_dst = format!("/Applications/{}", app_name);

    if !fs::exists(&app_dst).unwrap() {
        let result = Command::new("cp")
            .args(["-R", &app_src, &app_dst])
            .status()?;

        if !result.success() {
            return Err(eyre!("Failed to copy app"));
        }
    }

    Command::new("hdiutil")
        .args(["detach", &mount_point])
        .status()?;

    Ok(())
}

fn wait_for_mount(mount_tag: &str, max_attempts: u32) -> Result<String> {
    for attempt in 0..max_attempts {
        let output = Command::new("ls").arg("/Volumes/").output()?;
        let volumes = String::from_utf8_lossy(&output.stdout);

        if let Some(volume) = volumes.lines().find(|line| line.contains(mount_tag)) {
            let mount_point = format!("/Volumes/{}", volume.trim());

            if Path::new(&mount_point).exists() {
                return Ok(mount_point);
            }
        }

        let delay = Duration::from_millis(100 * 2u64.pow(attempt.min(3)));
        thread::sleep(delay);
    }

    Err(eyre!(
        "Timed out waiting for DMG to mount. Expected volume containing '{}'",
        mount_tag
    ))
}

fn find_app(mount_point: &str) -> Result<String> {
    // Read directory contents
    let entries =
        fs::read_dir(mount_point).map_err(|e| eyre!("Failed to read volume directory: {}", e))?;

    // Find .app bundles
    let apps: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "app")
                .unwrap_or(false)
        })
        .filter_map(|entry| entry.file_name().to_str().map(|s| s.to_string()))
        .collect();

    // Return the first .app found
    apps.into_iter()
        .next()
        .ok_or_else(|| eyre!("No .app bundle found in mounted volume"))
}
