use color_eyre::{Result, eyre::eyre};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const GIT_REPO_API: &str = "https://api.github.com/repos";

pub struct GithubRepo {
    pub author: &'static str,
    pub name: &'static str,
}

pub struct GithubApiClient(Client);

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct GithubRelease {
    pub url: String,
    pub assets_url: String,
    pub upload_url: String,
    pub html_url: String,
    pub id: u64,
    pub node_id: String,
    pub tag_name: String,
    pub target_commitish: String,
    pub name: String,
    pub draft: bool,
    pub prerelease: bool,
    pub created_at: String,
    pub published_at: String,
    pub assets: Vec<GithubAsset>,
    pub tarball_url: String,
    pub zipball_url: String,
    pub body: String,
    pub author: GithubAuthor,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct GithubAsset {
    pub url: String,
    pub id: u64,
    pub node_id: String,
    pub name: String,
    pub label: Option<String>,
    pub uploader: GithubAuthor,
    pub content_type: String,
    pub state: String,
    pub size: u64,
    pub download_count: u64,
    pub created_at: String,
    pub updated_at: String,
    pub browser_download_url: String,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct GithubAuthor {
    pub login: String,
    pub id: u64,
    pub node_id: String,
    pub avatar_url: String,
    pub gravatar_id: String,
    pub url: String,
    pub html_url: String,
    pub followers_url: String,
    pub following_url: String,
    pub gists_url: String,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub organizations_url: String,
    pub repos_url: String,
    pub events_url: String,
    pub received_events_url: String,
    #[serde(rename = "type")]
    pub user_type: String,
    pub site_admin: bool,
}

impl GithubApiClient {
    pub fn new() -> Self {
        Self(
            reqwest::blocking::Client::builder()
                .user_agent("github-api-http-client/1.0")
                .build()
                .expect("Could not create Github API HTTP client."),
        )
    }

    pub fn get_releases(&self, repo: GithubRepo) -> Result<Vec<GithubRelease>> {
        let mut url = PathBuf::new();
        url.push(GIT_REPO_API);
        url.push(repo.author);
        url.push(repo.name);
        url.push("releases");
        let url = url.to_str().expect("GitRepo struct is not valid unicode.");

        let response = self.0.get(url).send()?;
        match response.status() {
            reqwest::StatusCode::OK => response
                .json::<Vec<GithubRelease>>()
                .map_err(|e| eyre!("JSON decode error: {}", e)),
            reqwest::StatusCode::NOT_FOUND => Err(eyre!("(404) Repository not found.")),
            reqwest::StatusCode::FORBIDDEN => Err(eyre!("(403) Rate limited or access denied.")),
            status => {
                let error_body = response.text()?;
                Err(eyre!("HTTP {}: {}", status, error_body))
            }
        }
    }

    pub fn get_version(&self, repo: GithubRepo, version: &String) -> Result<GithubRelease> {
        let mut url = PathBuf::new();
        url.push(GIT_REPO_API);
        url.push(repo.author);
        url.push(repo.name);
        url.push("releases");
        url.push(version);
        let url = url
            .to_str()
            .expect("GithubRepo struct is not valid unicode.");

        let response = self.0.get(url).send()?;
        match response.status() {
            reqwest::StatusCode::OK => response
                .json::<GithubRelease>()
                .map_err(|e| eyre!("JSON decode error: {}", e)),
            reqwest::StatusCode::NOT_FOUND => Err(eyre!("(404) Repository not found.")),
            reqwest::StatusCode::FORBIDDEN => Err(eyre!("(403) Rate limited or access denied.")),
            status => {
                let error_body = response.text()?;
                Err(eyre!("HTTP {}: {}", status, error_body))
            }
        }
    }

    pub fn get_latest(&self, repo: GithubRepo) -> Result<GithubRelease> {
        let mut url = PathBuf::new();
        url.push(GIT_REPO_API);
        url.push(repo.author);
        url.push(repo.name);
        url.push("releases/latest");
        let url = url
            .to_str()
            .expect("GithubRepo struct is not valid unicode.");

        let response = self.0.get(url).send()?;
        match response.status() {
            reqwest::StatusCode::OK => response
                .json::<GithubRelease>()
                .map_err(|e| eyre!("JSON decode error: {}", e)),
            reqwest::StatusCode::NOT_FOUND => Err(eyre!("(404) Repository not found.")),
            reqwest::StatusCode::FORBIDDEN => Err(eyre!("(403) Rate limited or access denied.")),
            status => {
                let error_body = response.text()?;
                Err(eyre!("HTTP {}: {}", status, error_body))
            }
        }
    }
}

impl GithubAsset {
    pub fn size_bytes(&self) -> f64 {
        self.size as f64
    }

    pub fn size_kilobytes(&self) -> f64 {
        self.size_bytes() / 1024.0
    }

    pub fn size_megabytes(&self) -> f64 {
        self.size_kilobytes() / 1024.0
    }

    pub fn size_gigabytes(&self) -> f64 {
        self.size_megabytes() / 1024.0
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::OBS_GIT_REPO;

    #[test]
    fn test_git_api_client_get_releases() {
        println!(
            "{:?}",
            GithubApiClient::new().get_releases(OBS_GIT_REPO).unwrap()
        );
    }

    #[test]
    fn test_git_api_client_get_latest() {
        println!(
            "{:?}",
            GithubApiClient::new().get_latest(OBS_GIT_REPO).unwrap()
        );
    }
}
