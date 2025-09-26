use crate::{app::Event, file, git::*, scut};
use color_eyre::{Result, eyre::eyre};
use std::{fs, os, sync::mpsc};

pub fn install_obs(progress_tx: mpsc::Sender<Event>) -> Result<()> {
    // Build search tags per operating system
    #[cfg(target_os = "windows")]
    let (inc, exc) = (vec!["windows", "zip"], vec!["pdb"]);
    #[cfg(target_os = "macos")]
    let (inc, exc) = (vec!["macos", "dmg"], vec![]);
    #[cfg(target_os = "linux")]
    let (inc, exc) = (vec!["ubuntu", "deb"], vec!["ddeb"]);

    // Build search tags per cpu architecture
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    let arch = vec!["intel", "x86", "x64"];
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    let arch = vec!["arm", "apple"];

    // TODO: Prompt user for version instead of defaulting to latest.
    // Get latest OBS release assets
    let git_release = GithubApiClient::new()
        .get_latest(crate::OBS_GIT_REPO)
        .expect("Could not get latest OBS git release!");

    // Filter OBS assets using search tags
    let git_assets = git_release
        .assets
        .iter()
        .cloned()
        .filter(|asset| {
            let name = asset.name.to_lowercase();
            inc.iter().all(|i| name.contains(i))
                && !exc.iter().any(|e| name.contains(e))
                && arch.iter().any(|a| name.contains(a))
        })
        .collect::<Vec<GithubAsset>>();

    // TODO: Prompt user in event of multiple files
    assert_eq!(git_assets.len(), 1);
    let git_asset = git_assets[0].clone();

    // Build paths
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap();
    let file_path = exe_dir.join(&git_asset.name);
    let file_name = file::remove_extension(&git_asset.name);
    let obs_dir = exe_dir.join(file_name);

    if obs_dir.exists() || file_path.exists() {
        opener::open(exe_dir)?;
        return Err(eyre!("OBS is already installed!"));
    }

    // Download asset
    file::download_file(
        git_asset.browser_download_url.as_str(),
        &file_path,
        progress_tx.clone(),
    )?;

    #[cfg(target_os = "windows")]
    {
        // Extract ZIP
        file::extract_zip(&file_path, &obs_dir)?;
        fs::remove_file(file_path)?;

        // Enable portable mode
        fs::File::create(obs_dir.join("portable_mode"))?;

        // Create config true folder
        let true_config = exe_dir.join("obs-config");
        if !true_config.exists() {
            fs::create_dir(&true_config)?;
        }

        // Symlink config link folder
        let link_config = obs_dir.join("config");
        os::windows::fs::symlink_dir(true_config, link_config)?;

        // Create shortcut
        let scut_path = exe_dir.join("OBS.lnk");
        let target_path = obs_dir.join("bin/64bit/obs64.exe");
        scut::create_shortcut(scut_path, target_path)?;
    }

    #[cfg(target_family = "unix")]
    {
        // TODO: Run and rename installation according to version number

        // Create config true folder
        let true_config = exe_dir.join("obs-config");
        if !true_config.exists() {
            fs::create_dir(&true_config)?;
        }

        // Symlink config folder
        let link_config = obs_dir.join("config");
        os::unix::fs::symlink(true_config, link_config)?;

        // TODO: Create shortcut
    }

    // Open directory
    opener::open(exe_dir)?;

    // TODO: Download pre-built OBS config

    Ok(())
}

pub fn install_ja2(progress_tx: mpsc::Sender<Event>) -> Result<()> {
    // Build search tags per operating system
    #[cfg(target_os = "windows")]
    let inc = vec!["win"];
    #[cfg(target_os = "macos")]
    let inc = vec!["macos"];
    #[cfg(target_os = "linux")]
    let inc = vec!["ubuntu"];

    // Build search tags per cpu architecture
    #[cfg(any(target_arch = "x86"))]
    let arch = vec!["intel", "32"];
    #[cfg(any(target_arch = "x86_64"))]
    let arch = vec!["intel", "64"];
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    let arch = vec!["universal"];

    // TODO: Prompt user for version instead of defaulting to latest.
    // Get latest OBS release assets
    let git_release = GithubApiClient::new()
        .get_latest(crate::JACK2_GIT_REPO)
        .expect("Could not get latest JACK2 git release!");

    // Filter assets using search tags
    let git_assets = git_release
        .assets
        .iter()
        .cloned()
        .filter(|asset| {
            let name = asset.name.to_lowercase();
            inc.iter().all(|i| name.contains(i)) && arch.iter().any(|a| name.contains(a))
        })
        .collect::<Vec<GithubAsset>>();

    // TODO: Prompt user in event of multiple files
    assert_eq!(git_assets.len(), 1);
    let git_asset = git_assets[0].clone();

    // Build paths
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap();
    let file_path = exe_dir.join(&git_asset.name);

    // Download asset
    file::download_file(
        git_asset.browser_download_url.as_str(),
        &file_path,
        progress_tx.clone(),
    )?;

    // Install & clean up
    file::run_command(file_path.as_path())?;
    fs::remove_file(file_path)?;

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn install_vmb(_progress_tx: mpsc::Sender<Event>) -> Result<()> {
    Ok(file::winget(&[
        "install --id=VB-Audio.Voicemeeter.Banana  -e",
    ])?)
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn install_khs(progress_tx: mpsc::Sender<Event>) -> Result<()> {
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap();

    #[cfg(target_os = "windows")]
    let file_path = exe_dir.join("kilohearts_installer.exe");
    #[cfg(target_os = "macos")]
    let file_path = exe_dir.join("kilohearts_installer.dmg");

    // Install & clean up
    file::download_file(crate::KHS_URL, &file_path, progress_tx)?;
    file::run_command(file_path.as_path())?;
    fs::remove_file(file_path)?;

    Ok(())
}
