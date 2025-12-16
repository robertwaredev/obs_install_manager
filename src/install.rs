#[cfg(windows)]
use crate::scut;
use crate::{app::Event, file, git::*};
use color_eyre::{Result, eyre::OptionExt};
use std::{fs, os, sync::mpsc::Sender};

// OBS (Open Broadcast Software)
pub fn obs(tx: Sender<Event>) -> Result<()> {
    let github_api_client = GithubApiClient::new()?;

    // Search tags per operating system
    #[cfg(target_os = "windows")]
    let (incl, excl) = (vec!["windows", "zip"], vec!["pdb"]);
    #[cfg(target_os = "macos")]
    let (incl, excl) = (vec!["macos", "dmg"], vec!["tar"]);
    #[cfg(target_os = "linux")]
    let (incl, excl) = (vec!["ubuntu", "deb"], vec!["ddeb"]);

    // Search tags per cpu architecture
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    let arch = vec!["intel", "x86", "x64"];
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    let arch = vec!["arm", "apple"];

    // Get latest asset infos
    let git_release = github_api_client.get_release(&crate::OBS_REPO, None)?;
    let git_assets = git_release.get_assets(Some(incl), Some(excl), Some(arch));
    let git_asset = git_assets.first().ok_or_eyre("Git asset not found!")?;

    // Build paths
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap();
    let asset_path = exe_dir.join(&git_asset.name);
    let asset_name = asset_path.file_stem().unwrap();

    // Download asset
    if asset_path.exists() {
        fs::remove_file(&asset_path)?;
    }
    file::download(&git_asset.browser_download_url, &asset_path, &tx)?;

    // Windows setup
    #[cfg(target_os = "windows")]
    {
        // Extract zip
        let asset_dir = exe_dir.join(&asset_name);
        if asset_dir.exists() {
            fs::remove_dir_all(&asset_dir)?;
        }
        file::extract_zip(&asset_path, &asset_dir)?;
        fs::remove_file(&asset_path)?;

        // Enable portable mode
        fs::File::create(asset_dir.join("portable_mode"))?;

        // Setup config folder
        let cfg_dir = exe_dir.join("obs-config");
        if !cfg_dir.exists() {
            fs::create_dir(&cfg_dir)?;
        }
        os::windows::fs::symlink_dir(&cfg_dir, &asset_dir.join("config"))?;

        // Download & extract OBS config
        let cfg_path = exe_dir.join("daw-obs-config-master.zip");
        let cfg_name = exe_dir.join("daw-obs-config-master");
        let cfg_src = cfg_name.join("obs-studio");
        let cfg_dst = cfg_dir.join("obs-studio");

        if cfg_path.exists() {
            fs::remove_file(&cfg_path)?;
        }
        if cfg_name.exists() {
            fs::remove_dir_all(&cfg_name)?;
        }

        file::download(&crate::OBS_CONFIG_URL.to_string(), &cfg_path, &tx)?;
        file::extract_zip(&cfg_path, &exe_dir.to_path_buf())?;
        file::copy_dir(&cfg_src, &cfg_dst)?;
        fs::remove_file(&cfg_path)?;
        fs::remove_dir_all(&cfg_name)?;

        // OBS ASIO plugin
        {
            // Get latest asset infos
            let git_release = github_api_client.get_release(&crate::OBS_ASIO_REPO, None)?;
            let git_assets = git_release.get_assets(Some(vec!["zip"]), None, None);
            let git_asset = git_assets.first().ok_or_eyre("Git asset not found!")?;

            // Download asset
            let asset_path = exe_dir.join(&git_asset.name);
            if !asset_path.exists() {
                file::download(&git_asset.browser_download_url, &asset_path, &tx)?;
            }

            // Extract zip
            file::extract_zip(&asset_path, &asset_dir)?;
            fs::remove_file(&asset_path)?;
        }

        // OBS atkAudio plugin
        {
            // Get latest asset infos
            let git_release = github_api_client.get_release(&crate::OBS_ATK_REPO, None)?;
            let git_assets = git_release.get_assets(Some(vec!["zip"]), None, None);
            let git_asset = git_assets.first().ok_or_eyre("Git asset not found!")?;

            // Download asset
            let asset_path = exe_dir.join(&git_asset.name);
            if !asset_path.exists() {
                file::download(&git_asset.browser_download_url, &asset_path, &tx)?;
            }

            // Extract zip into sub folder
            let asset_dir = exe_dir.join("atk_audio");
            file::extract_zip(&asset_path, &asset_dir)?;
            fs::remove_file(&asset_path)?;

            // Filter entries for platform and extract zip
            for entry in fs::read_dir(&asset_dir)? {
                let entry_path = entry?.path();
                let entry_name = entry_path.to_str().unwrap().to_lowercase();
                if entry_name.contains("windows") && entry_name.contains("zip") {
                    file::extract_zip(&entry_path, &asset_dir)?;
                }
            }

            fs::remove_dir_all(&asset_dir)?;
        }

        // Create OBS shortcut
        {
            let scut_path = exe_dir.join("OBS.lnk");
            if scut_path.exists() {
                fs::remove_file(&scut_path)?;
            }
            let target_path = asset_dir.join("bin/64bit/obs64.exe");
            scut::create_shortcut(scut_path, target_path)?;
        }

        // Open exe directory
        opener::open(exe_dir)?;
    }

    // MacOS setup
    #[cfg(target_os = "macos")]
    {
        // Install DMG
        file::install_dmg(&asset_path.to_str().unwrap(), "OBS")?;
        fs::remove_file(&asset_path)?;

        // Get home path from env variable
        let home = std::env::var("HOME")?;
        let home = std::path::PathBuf::from(&home);

        // Download & extract OBS config
        let cfg_path = exe_dir.join("daw-obs-config-macos-master.zip");
        let cfg_name = exe_dir.join("daw-obs-config-macos-master");
        let cfg_src = cfg_name.join("obs-studio");
        let cfg_dst = home.join("Library/Application Support/obs-studio");

        if cfg_path.exists() {
            fs::remove_file(&cfg_path)?;
        }
        if cfg_name.exists() {
            fs::remove_dir_all(&cfg_name)?;
        }

        // Extract zip and move contents
        file::download(&crate::OBS_CONFIG_URL.to_string(), &cfg_path, &tx)?;
        file::extract_zip(&cfg_path, &exe_dir.to_path_buf())?;
        file::copy_dir(&cfg_src, &cfg_dst)?;
        fs::remove_file(&cfg_path)?;
        fs::remove_dir_all(&cfg_name)?;
    }

    Ok(())
}

// Kilohearts Bundle
pub fn khs(tx: Sender<Event>) -> Result<()> {
    // Build paths
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap();
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    let file_path = exe_dir.join("kilohearts_installer.exe");
    #[cfg(target_os = "macos")]
    let file_path = exe_dir.join("kilohearts_installer.dmg");

    // Download & run
    if !file_path.exists() {
        file::download(&crate::KHS_URL.to_string(), &file_path, &tx)?;
    }
    file::run(&file_path)?;
    fs::remove_file(&file_path)?;

    Ok(())
}

// ReaPlugs Bundle
#[cfg(any(target_os = "windows", target_os = "linux"))]
pub fn rea(tx: Sender<Event>) -> Result<()> {
    // Build Paths
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap();
    let file_path = exe_dir.join("reaplugs_installer.exe");

    // Download & run
    if !file_path.exists() {
        file::download(&crate::REA_URL.to_string(), &file_path, &tx)?;
    }
    file::run(&file_path)?;
    fs::remove_file(&file_path)?;

    Ok(())
}

// Voicemeeter Banana
#[cfg(target_os = "windows")]
pub fn vmb(tx: Sender<Event>) -> Result<()> {
    // Build paths
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap();
    let zip_path = exe_dir.join("voicemeeter_banana_installer.zip");

    // Download zip
    if !zip_path.exists() {
        file::download(&crate::VMB_URL.to_string(), &zip_path, &tx)?;
    }

    // Extract zip
    file::extract_zip(&zip_path, &exe_dir.to_path_buf())?;
    fs::remove_file(&zip_path)?;

    // Run installer
    let file_path = exe_dir.join("voicemeeterprosetup.exe");
    file::run(&file_path)?;
    fs::remove_file(&file_path)?;

    Ok(())
}

// BlackHole
#[cfg(target_os = "macos")]
pub fn eab(_: Sender<Event>) -> Result<()> {
    opener::open_browser(&crate::EAB_URL)?;
    Ok(())
}

// Sonobus
#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn sbs(tx: Sender<Event>) -> Result<()> {
    // Search tags per operating system
    #[cfg(target_os = "windows")]
    let incl = vec!["win", "exe"];
    #[cfg(target_os = "macos")]
    let incl = vec!["mac", "dmg"];

    // Get latest OBS asset infos
    let git_release = GithubApiClient::new()?.get_release(&crate::SONOBUS_REPO, None)?;
    let git_assets = git_release.get_assets(Some(incl), None, None);
    let git_asset = git_assets.first().ok_or_eyre("Git asset not found!")?;

    // Build paths
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap();
    let file_path = exe_dir.join(&git_asset.name);

    // Download & run
    if !file_path.exists() {
        file::download(&git_asset.browser_download_url, &file_path, &tx)?;
    }
    file::run(&file_path)?;
    fs::remove_file(&file_path)?;

    Ok(())
}
