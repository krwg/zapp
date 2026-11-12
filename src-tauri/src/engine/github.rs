// ╔════════════════════════════════════════════════════╗
// ║                  engine/github                   ║
// ╚════════════════════════════════════════════════════╝

use serde::Deserialize;

use crate::error::{AppError, AppResult};

pub const FLOWSEAL_REPO: &str = "Flowseal/zapret-discord-youtube";
const RELEASES_LATEST: &str =
    "https://api.github.com/repos/Flowseal/zapret-discord-youtube/releases/latest";

#[derive(Debug, Clone, Deserialize)]
pub struct GithubRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub html_url: Option<String>,
    pub assets: Vec<GithubAsset>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GithubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

pub fn fetch_latest_release() -> AppResult<GithubRelease> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("zapp/0.2 (+https://github.com/krwg/zapp)")
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| AppError::from(e.to_string()))?;

    let response = client
        .get(RELEASES_LATEST)
        .header("Accept", "application/vnd.github+json")
        .send()
        .map_err(|e| AppError::from(format!("GitHub request failed: {e}")))?;

    if !response.status().is_success() {
        return Err(AppError::from(format!(
            "GitHub API status {}",
            response.status()
        )));
    }

    response
        .json::<GithubRelease>()
        .map_err(|e| AppError::from(format!("GitHub JSON parse failed: {e}")))
}

impl GithubRelease {
    pub fn preferred_zip_asset(&self) -> Option<&GithubAsset> {
        self.assets
            .iter()
            .find(|a| a.name.ends_with(".zip"))
            .or_else(|| self.assets.iter().find(|a| a.name.ends_with(".tar.gz")))
    }

    pub fn version(&self) -> &str {
        self.tag_name.trim_start_matches('v')
    }
}

pub fn compare_semver(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |s: &str| -> Vec<u64> {
        s.trim_start_matches('v')
            .split('.')
            .filter_map(|p| {
                p.chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse()
                    .ok()
            })
            .collect()
    };
    parse(a).cmp(&parse(b))
}
