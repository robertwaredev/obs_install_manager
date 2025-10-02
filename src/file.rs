use crate::app::{Event, send_progress_event};
use color_eyre::{Result, eyre::eyre};
use std::{
    fs,
    io::{self, Read, Write},
    path::Path,
    process::{Command, ExitStatus},
    sync::mpsc,
};

pub fn download<P: AsRef<Path>>(
    url: &String,
    file_path: P,
    progress_tx: &mpsc::Sender<Event>,
) -> Result<()> {
    let mut response = reqwest::blocking::get(url)?;
    let mut file = fs::File::create(file_path)?;
    let mut buffer = [0u8; 8192];
    let mut downloaded = 0u64;

    if let Some(total_size) = response.content_length() {
        while let Ok(n) = response.read(&mut buffer) {
            if n == 0 {
                break;
            }
            file.write_all(&buffer[..n])?;
            downloaded += n as u64;
            let ratio = downloaded as f64 / total_size as f64;
            send_progress_event(ratio, progress_tx);
        }

        Ok(send_progress_event(0.0, progress_tx))
    } else {
        Err(eyre!("Total file download size could not be determined!"))
    }
}

// TODO: Set up progress bar for extract
pub fn extract_zip<P: AsRef<Path>>(file_path: P, extract_dir: P) -> Result<()> {
    let file = fs::File::open(file_path)?;
    let reader = io::BufReader::new(file);
    let mut archive = zip::ZipArchive::new(reader)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let extract_path = extract_dir.as_ref().join(file.name());

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&extract_path)?;
        } else {
            if let Some(parent) = extract_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = fs::File::create(&extract_path)?;
            io::copy(&mut file, &mut outfile)?;
        }

        // Set permissions on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(())
}

pub fn run<P: AsRef<Path>>(path: P) -> io::Result<ExitStatus> {
    Ok(Command::new(path.as_ref().as_os_str()).spawn()?.wait()?)
}
