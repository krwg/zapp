// ╔════════════════════════════════════════════════════╗
// ║                  engine/github                   ║
// ╚════════════════════════════════════════════════════╝

use serde::Deserialize;

use crate::error::{AppError, AppResult};

pub const FLOWSEAL_REPO: &str = "Flowseal/zapret-discord-youtube";
pub const ZAPP_REPO: &str = "krwg/zapp";

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
    /// GitHub Releases asset digest, e.g. `sha256:abcdef…` (optional until publisher sets it).
    #[serde(default)]
    pub digest: Option<String>,
}

pub fn fetch_latest_release(repo: &str) -> AppResult<GithubRelease> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let client = reqwest::blocking::Client::builder()
        .user_agent("zapp/0.2 (+https://github.com/krwg/zapp)")
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| AppError::from(e.to_string()))?;

    let response = client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .map_err(|e| AppError::from(format!("GitHub request failed ({repo}): {e}")))?;

    if !response.status().is_success() {
        return Err(AppError::from(format!(
            "GitHub API {repo} status {}",
            response.status()
        )));
    }

    response
        .json::<GithubRelease>()
        .map_err(|e| AppError::from(format!("GitHub JSON parse failed ({repo}): {e}")))
}

pub fn fetch_flowseal_latest() -> AppResult<GithubRelease> {
    fetch_latest_release(FLOWSEAL_REPO)
}

pub fn fetch_zapp_latest() -> AppResult<GithubRelease> {
    fetch_latest_release(ZAPP_REPO)
}

impl GithubRelease {
    pub fn preferred_zip_asset(&self) -> Option<&GithubAsset> {
        self.assets
            .iter()
            .find(|a| a.name.ends_with(".zip"))
            .or_else(|| self.assets.iter().find(|a| a.name.ends_with(".tar.gz")))
    }

    /// Prefer Windows installer assets for the Application channel.
    /// NSIS setup.exe ranks above MSI (README recommends NSIS; avoids dual uninstall entries).
    pub fn preferred_app_asset(&self) -> Option<&GithubAsset> {
        let rank = |name: &str| -> u8 {
            let n = name.to_ascii_lowercase();
            if n.contains("setup") && n.ends_with(".exe") {
                0
            } else if n.ends_with(".msi") {
                1
            } else if n.ends_with(".exe") {
                2
            } else if n.ends_with(".zip") && n.contains("zapp") {
                3
            } else if n.ends_with(".zip") {
                4
            } else {
                9
            }
        };

        self.assets
            .iter()
            .filter(|a| {
                let n = a.name.to_ascii_lowercase();
                n.ends_with(".msi")
                    || n.ends_with(".exe")
                    || n.ends_with(".zip")
            })
            .min_by_key(|a| rank(&a.name))
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
