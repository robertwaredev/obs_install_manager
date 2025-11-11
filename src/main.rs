use crate::{
    app::{App, Result},
    git::GithubRepo,
};

pub mod app;
pub mod file;
pub mod git;
pub mod install;
pub mod scut;
pub mod ui;

pub const OBS_GIT_REPO: GithubRepo = GithubRepo {
    author: "obsproject",
    name: "obs-studio",
};

pub const OBS_ASIO_GIT_REPO: GithubRepo = GithubRepo {
    author: "andersama",
    name: "obs-asio",
};

pub const OBS_ATK_AUDIO_REPO: GithubRepo = GithubRepo {
    author: "atkAudio",
    name: "PluginForObsRelease",
};

pub const BLACKHOLE_REPO: GithubRepo = GithubRepo {
    author: "ExistentialAudio",
    name: "BlackHole",
};

pub const JA2_GIT_REPO: GithubRepo = GithubRepo {
    author: "jackaudio",
    name: "jack2-releases",
};

pub const SBS_GIT_REPO: GithubRepo = GithubRepo {
    author: "sonosaurus",
    name: "sonobus",
};

pub const OBS_CONFIG: &str =
    "https://github.com/robertwaredev/daw-obs-config/archive/refs/heads/master.zip";

pub const VMB_URL: &str = "https://download.vb-audio.com/Download_CABLE/VoicemeeterSetup_v2119.zip";

pub const REA_URL: &str = "https://www.reaper.fm/reaplugs/reaplugs236_x64-install.exe";

#[cfg(target_os = "windows")]
pub const KHS_URL: &str = "https://kilohearts.com/data/install/_/win";
#[cfg(target_os = "macos")]
pub const KHS_URL: &str = "https://kilohearts.com/data/install/_/mac";

pub const EAB_URL: &str = "https://existential.audio/blackhole/";

fn main() -> Result<()> {
    color_eyre::install()?;
    let term = ratatui::init();
    let res = App::new().run(term);
    ratatui::restore();
    res
}
