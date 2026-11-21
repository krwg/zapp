// ╔════════════════════════════════════════════════════╗
// ║                  engine/github                   ║
// ╚════════════════════════════════════════════════════╝

use std::env;

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

    #[serde(default)]
    pub digest: Option<String>,
}





fn github_api_bases() -> Vec<String> {
    let mut bases = Vec::new();
    if let Ok(one) = env::var("ZAPP_GITHUB_API_BASE") {
        let t = one.trim().trim_end_matches('/');
        if !t.is_empty() {
            bases.push(t.to_string());
        }
    }
    if let Ok(list) = env::var("ZAPP_GITHUB_API_MIRRORS") {
        for part in list.split(',') {
            let t = part.trim().trim_end_matches('/');
            if !t.is_empty() && !bases.iter().any(|b| b == t) {
                bases.push(t.to_string());
            }
        }
    }
    let official = "https://api.github.com".to_string();
    if !bases.iter().any(|b| b == &official) {
        bases.push(official);
    }
    bases
}



fn rewrite_asset_url(url: &str) -> String {
    if let Ok(prefix) = env::var("ZAPP_GITHUB_ASSET_PREFIX") {
        let p = prefix.trim().trim_end_matches('/');
        if !p.is_empty() {
            return format!("{p}/{url}");
        }
    }
    url.to_string()
}

pub fn fetch_latest_release(repo: &str) -> AppResult<GithubRelease> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("zapp/0.2 (+https://github.com/krwg/zapp)")
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| AppError::from(e.to_string()))?;

    let mut last_err = AppError::from(format!("GitHub request failed ({repo})"));
    for base in github_api_bases() {
        let url = format!("{base}/repos/{repo}/releases/latest");
        match client
            .get(&url)
            .header("Accept", "application/vnd.github+json")
            .send()
        {
            Ok(response) if response.status().is_success() => {
                return response.json::<GithubRelease>().map_err(|e| {
                    AppError::from(format!("GitHub JSON parse failed ({repo}): {e}"))
                });
            }
            Ok(response) => {
                last_err = AppError::from(format!(
                    "GitHub API {repo} via {base} status {}",
                    response.status()
                ));
            }
            Err(e) => {
                last_err =
                    AppError::from(format!("GitHub request failed ({repo} via {base}): {e}"));
            }
        }
    }
    Err(last_err)
}

pub fn fetch_flowseal_latest() -> AppResult<GithubRelease> {
    fetch_latest_release(FLOWSEAL_REPO)
}

pub fn fetch_zapp_latest() -> AppResult<GithubRelease> {
    fetch_latest_release(ZAPP_REPO)
}


pub fn asset_download_url(asset: &GithubAsset) -> String {
    rewrite_asset_url(&asset.browser_download_url)
}

impl GithubRelease {
    pub fn preferred_zip_asset(&self) -> Option<&GithubAsset> {
        self.assets
            .iter()
            .find(|a| a.name.ends_with(".zip"))
            .or_else(|| self.assets.iter().find(|a| a.name.ends_with(".tar.gz")))
    }



    pub fn preferred_app_asset(&self) -> Option<&GithubAsset> {
        let rank = |name: &str| -> u8 {
            let n = name.to_ascii_lowercase();
            if n.contains("setup") && n.ends_with(".exe") {
                0
            } else if n.ends_with(".msi") {
                1
            } else if n.ends_with(".exe") {
                2
            } else if n.ends_with(".zip") {
                3
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemverCmp {
    Lt,
    Eq,
    Gt,
}

pub fn compare_semver(a: &str, b: &str) -> SemverCmp {
    let parse = |s: &str| -> Vec<u64> {
        s.trim()
            .trim_start_matches('v')
            .split(|c: char| !c.is_ascii_digit())
            .filter(|p| !p.is_empty())
            .filter_map(|p| p.parse().ok())
            .collect()
    };
    let pa = parse(a);
    let pb = parse(b);
    let n = pa.len().max(pb.len());
    for i in 0..n {
        let x = pa.get(i).copied().unwrap_or(0);
        let y = pb.get(i).copied().unwrap_or(0);
        if x < y {
            return SemverCmp::Lt;
        }
        if x > y {
            return SemverCmp::Gt;
        }
    }
    SemverCmp::Eq
}

impl SemverCmp {
    pub fn is_gt(self) -> bool {
        matches!(self, SemverCmp::Gt)
    }
}
