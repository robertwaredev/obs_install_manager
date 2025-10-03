use color_eyre::{Result, eyre::eyre};
use reqwest::{IntoUrl, blocking::Client};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::path::PathBuf;

pub const GIT_REPO_API: &str = "https://api.github.com/repos";

pub struct GithubRepo {
    pub author: &'static str,
    pub name: &'static str,
}

impl GithubRepo {
    pub fn url(&self) -> PathBuf {
        PathBuf::new()
            .join(GIT_REPO_API)
            .join(self.author)
            .join(self.name)
            .join("releases")
    }
}

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

pub struct GithubApiClient(Client);

impl GithubApiClient {
    pub fn new() -> Result<Self> {
        let client = Self(
            reqwest::blocking::Client::builder()
                .user_agent("github-api-client/1.0")
                .build()
                .map_err(|e| eyre!("Could not create GithubApiClient. {}", e))?,
        );
        Ok(client)
    }

    pub fn get_releases(&self, repo: &GithubRepo) -> Result<Vec<GithubRelease>> {
        let url = repo.url();
        let url = url
            .to_str()
            .ok_or_else(|| eyre!("GithubRepo is not valid unicode."))?;
        self.parse_json::<Vec<GithubRelease>>(url)
    }

    pub fn get_release(&self, repo: &GithubRepo, id: &Option<String>) -> Result<GithubRelease> {
        let mut url = repo.url();

        if let Some(id) = id {
            url.push(id);
        } else {
            url.push("latest");
        }

        let url = url
            .to_str()
            .ok_or_else(|| eyre!("GithubRepo or id is not valid unicode."))?;
        self.parse_json::<GithubRelease>(url)
    }

    #[rustfmt::skip]
    fn parse_json<T: DeserializeOwned>(&self, url: impl IntoUrl) -> Result<T> {
        let response = self.0.get(url).send()?;
        match response.status() {
            reqwest::StatusCode::OK => response.json::<T>().map_err(|e| eyre!("JSON decode error: {}", e)),
            reqwest::StatusCode::NOT_FOUND => Err(eyre!("(404) Repository not found.")),
            reqwest::StatusCode::FORBIDDEN => Err(eyre!("(403) Rate limited or access denied.")),
            status => Err(eyre!("HTTP {}: {}", status, response.text()?)),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_git_api_client_get_releases() {
        println!(
            "{:?}",
            GithubApiClient::new()
                .unwrap()
                .get_releases(&crate::OBS_GIT_REPO)
                .unwrap()
        );
    }

    #[test]
    fn test_git_api_client_get_release() {
        println!(
            "{:?}",
            GithubApiClient::new()
                .unwrap()
                .get_release(&crate::OBS_GIT_REPO, &None)
                .unwrap()
        );
    }
}
