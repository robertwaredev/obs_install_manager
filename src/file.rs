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
