use crate::{app::Event, file, git::*, scut};
use color_eyre::{
    Result,
    eyre::{OptionExt, eyre},
};
use std::{fs, os, sync::mpsc::Sender};

pub trait Installable {
    fn install(&self, tx: &Sender<Event>) -> Result<()>;
}

#[derive(Clone)]
pub enum Installer {
    Obs(Obs),
    Vmb(Vmb),
    Ja2(Ja2),
    Khs(Khs),
    Sbs(Sbs),
}

#[derive(Default, Clone)]
pub struct Obs {
    pub version: Option<String>,
}

impl Installable for Obs {
    fn install(&self, tx: &Sender<Event>) -> Result<()> {
        let github_api_client = GithubApiClient::new()?;

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

        // Get latest release assets
        let git_release = github_api_client.get_release(&crate::OBS_GIT_REPO, &self.version)?;

        // Filter assets using search tags
        let git_assets = git_release
            .assets
            .into_iter()
            .filter(|asset| {
                let name = asset.name.to_lowercase();
                inc.iter().all(|i| name.contains(i))
                    && !exc.iter().any(|e| name.contains(e))
                    && arch.iter().any(|a| name.contains(a))
            })
            .collect::<Vec<GithubAsset>>();

        let git_asset = git_assets.first().ok_or_eyre("GithubAsset vec is empty!")?;

        // Build paths
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path.parent().unwrap();
        let file_path = exe_dir.join(&git_asset.name);
        let file_name = file_path.file_stem().unwrap();
        let extract_dir = exe_dir.join(&file_name);

        if extract_dir.exists() || file_path.exists() {
            opener::open(exe_dir)?;
            return Err(eyre!("OBS is already installed!"));
        }

        // Download asset
        file::download(&git_asset.browser_download_url, &file_path, tx)?;

        // Windows setup
        #[cfg(target_os = "windows")]
        {
            // Extract ZIP
            file::extract_zip(&file_path, &extract_dir)?;
            fs::remove_file(&file_path)?;

            // Enable portable mode
            fs::File::create(extract_dir.join("portable_mode"))?;

            // Create config true folder
            let true_config = exe_dir.join("obs-config");
            if !true_config.exists() {
                fs::create_dir(&true_config)?;
            }

            // Symlink config link folder
            let link_config = extract_dir.join("config");
            os::windows::fs::symlink_dir(&true_config, &link_config)?;

            // OBS ASIO Plugin
            {
                // Get latest release assets
                let git_release =
                    github_api_client.get_release(&crate::OBS_ASIO_GIT_REPO, &self.version)?;

                // Filter assets using search tags
                let git_assets = git_release
                    .assets
                    .iter()
                    .cloned()
                    .filter(|asset| asset.name.to_lowercase().contains("zip"))
                    .collect::<Vec<GithubAsset>>();

                let git_asset = git_assets.first().ok_or_eyre("GithubAsset vec is empty!")?;

                // Download asset
                let zip_path = exe_dir.join(&git_asset.name);
                if !zip_path.exists() {
                    file::download(&git_asset.browser_download_url, &zip_path, tx)?;
                }
                file::extract_zip(&zip_path, &extract_dir)?;
                fs::remove_file(&zip_path)?;
            }

            // OBS Profile & Scene Collection
            {
                let zip_path = exe_dir.join("daw-obs-config-master.zip");
                let zip_name = exe_dir.join("daw-obs-config-master");
                let from = zip_name.join("obs-studio");
                let to = true_config.join("obs-studio");

                if !to.exists() {
                    if !zip_path.exists() {
                        file::download(&crate::OBS_CONFIG.to_string(), &zip_path, tx)?;
                    }
                    file::extract_zip(&zip_path, &exe_dir.to_path_buf())?;
                    fs::rename(&from, &to)?;
                    fs::remove_file(&zip_path)?;
                    fs::remove_dir_all(&zip_name)?;
                }
            }

            // OBS shortcut
            {
                let scut_path = exe_dir.join("OBS.lnk");
                if !scut_path.exists() {
                    let target_path = extract_dir.join("bin/64bit/obs64.exe");
                    scut::create_shortcut(scut_path, target_path)?;
                }
            }
        }

        // Unix setup
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

        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct Ja2 {
    pub version: Option<String>,
}

impl Installable for Ja2 {
    fn install(&self, tx: &Sender<Event>) -> Result<()> {
        // Build search tags per operating system
        #[cfg(target_os = "windows")]
        let inc = vec!["win"];
        #[cfg(target_os = "macos")]
        let inc = vec!["macos"];
        #[cfg(target_os = "linux")]
        let inc = vec!["linux"];

        // Build search tags per cpu architecture
        #[cfg(any(target_arch = "x86"))]
        let arch = vec!["intel", "32"];
        #[cfg(any(target_arch = "x86_64"))]
        let arch = vec!["intel", "64"];
        #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
        let arch = vec!["universal"];

        // Get latest release assets
        let git_release =
            GithubApiClient::new()?.get_release(&crate::JA2_GIT_REPO, &self.version)?;

        // Filter assets using search tags
        let git_assets = git_release
            .assets
            .into_iter()
            .filter(|asset| {
                let name = asset.name.to_lowercase();
                inc.iter().all(|i| name.contains(i)) && arch.iter().any(|a| name.contains(a))
            })
            .collect::<Vec<GithubAsset>>();

        let git_asset = git_assets.first().ok_or_eyre("GithubAsset vec is empty!")?;

        // Build paths
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path.parent().unwrap();
        let file_path = exe_dir.join(&git_asset.name);

        // Download asset
        file::download(&git_asset.browser_download_url, &file_path, tx)?;

        // Install & clean up
        file::run(&file_path)?;
        fs::remove_file(&file_path)?;

        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct Vmb;

impl Installable for Vmb {
    fn install(&self, tx: &Sender<Event>) -> Result<()> {
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path.parent().unwrap();
        let zip_path = exe_dir.join("voicemeeter_banana_installer.zip");

        // Install & clean up
        if !zip_path.exists() {
            file::download(&crate::VMB_URL.to_string(), &zip_path, tx)?;
            file::extract_zip(&zip_path, &exe_dir.to_path_buf())?;
            fs::remove_file(&zip_path)?;
        }

        // Open directory
        let file_path = exe_dir.join("voicemeeterprosetup.exe");
        file::run(&file_path)?;
        fs::remove_file(&file_path)?;

        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct Khs;

impl Installable for Khs {
    fn install(&self, tx: &Sender<Event>) -> Result<()> {
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path.parent().unwrap();

        #[cfg(target_os = "windows")]
        let file_path = exe_dir.join("kilohearts_installer.exe");
        #[cfg(target_os = "macos")]
        let file_path = exe_dir.join("kilohearts_installer.dmg");

        // Download and run
        if !file_path.exists() {
            file::download(&crate::KHS_URL.to_string(), &file_path, tx)?;
        }
        file::run(&file_path)?;
        fs::remove_file(&file_path)?;

        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct Sbs {
    pub version: Option<String>,
}

impl Installable for Sbs {
    fn install(&self, tx: &Sender<Event>) -> Result<()> {
        // Build search tags per operating system
        #[cfg(target_os = "windows")]
        let inc = vec!["win", "exe"];
        #[cfg(target_os = "macos")]
        let inc = vec!["mac", "dmg"];

        // Get latest OBS release assets
        let git_release =
            GithubApiClient::new()?.get_release(&crate::SBS_GIT_REPO, &self.version)?;

        // Filter OBS assets using search tags
        let git_assets = git_release
            .assets
            .into_iter()
            .filter(|asset| {
                let name = asset.name.to_lowercase();
                inc.iter().all(|i| name.contains(i))
            })
            .collect::<Vec<GithubAsset>>();

        let git_asset = git_assets.first().ok_or_eyre("GithubAsset vec is empty!")?;

        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path.parent().unwrap();
        let file_path = exe_dir.join(&git_asset.name);

        // Download & install
        file::download(&git_asset.browser_download_url, &file_path, tx)?;
        file::run(&file_path)?;
        fs::remove_file(&file_path)?;

        Ok(())
    }
}
