// ╔════════════════════════════════════════════════════╗
// ║                 engine/integrity                 ║
// ╚════════════════════════════════════════════════════╝

use std::fs::File;
use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::engine::github::GithubAsset;
use crate::error::{AppError, AppResult};

const MIN_ZIP_BYTES: u64 = 1024;

pub fn file_sha256_hex(path: &Path) -> AppResult<String> {
    let mut file = File::open(path).map_err(|e| AppError::from(e.to_string()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| AppError::from(e.to_string()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

pub fn parse_sha256_digest(digest: &str) -> Option<String> {
    let d = digest.trim();
    let hex = if let Some(rest) = d.strip_prefix("sha256:") {
        rest
    } else if let Some(rest) = d.strip_prefix("SHA256:") {
        rest
    } else if d.len() == 64 && d.chars().all(|c| c.is_ascii_hexdigit()) {
        d
    } else {
        return None;
    };
    let hex = hex.trim();
    if hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(hex.to_ascii_lowercase())
    } else {
        None
    }
}


pub fn expected_app_sha256(asset: &GithubAsset, body: Option<&str>) -> AppResult<String> {
    if let Some(d) = asset.digest.as_deref().and_then(parse_sha256_digest) {
        return Ok(d);
    }
    if let Some(body) = body {
        if let Some(h) = sha256_from_release_body(body, &asset.name) {
            return Ok(h);
        }
    }
    Err(AppError::from(format!(
        "No SHA256 for asset '{}' (GitHub digest missing and not found in release notes). Refusing to install.",
        asset.name
    )))
}

pub fn verify_file_sha256(path: &Path, expected_hex: &str) -> AppResult<()> {
    let actual = file_sha256_hex(path)?;
    let expected = expected_hex.trim().to_ascii_lowercase();
    if actual != expected {
        return Err(AppError::from(format!(
            "SHA256 mismatch for {}: expected {expected}, got {actual}",
            path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("file")
        )));
    }
    Ok(())
}


pub fn verify_zip_sanity(path: &Path) -> AppResult<()> {
    let meta = std::fs::metadata(path).map_err(|e| AppError::from(e.to_string()))?;
    if meta.len() < MIN_ZIP_BYTES {
        return Err(AppError::from(format!(
            "Downloaded archive too small ({} bytes)",
            meta.len()
        )));
    }
    let mut file = File::open(path).map_err(|e| AppError::from(e.to_string()))?;
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)
        .map_err(|e| AppError::from(format!("Cannot read zip header: {e}")))?;
    if magic != *b"PK\x03\x04" && magic != *b"PK\x05\x06" && magic != *b"PK\x07\x08" {
        return Err(AppError::from(
            "Downloaded file is not a ZIP archive (bad magic)",
        ));
    }
    Ok(())
}

fn sha256_from_release_body(body: &str, asset_name: &str) -> Option<String> {
    let lower_name = asset_name.to_ascii_lowercase();
    let lines: Vec<&str> = body.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.to_ascii_lowercase().contains(&lower_name) {

            for next in lines.iter().skip(i + 1).take(4) {
                let candidate = next.trim();
                if candidate.is_empty() || candidate.starts_with('`') && candidate.len() < 8 {
                    continue;
                }
                let candidate = candidate.trim_matches('`').trim();
                if let Some(h) = parse_sha256_digest(candidate) {
                    return Some(h);
                }
            }
        }

        if let Some(rest) = trimmed.strip_prefix("SHA256:") {
            if let Some(h) = parse_sha256_digest(rest.trim()) {

                let window = lines[i.saturating_sub(6)..=i].join("\n").to_ascii_lowercase();
                if window.contains(&lower_name) {
                    return Some(h);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_digest_variants() {
        assert_eq!(
            parse_sha256_digest(
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            )
            .as_deref(),
            Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        );
        assert!(parse_sha256_digest("md5:abc").is_none());
    }

    #[test]
    fn body_lookup() {
        let body = r#"
## Downloads
zapp_0.2.6_x64-setup.exe
CA635F8F9C73A2A5BB73A5CA3B967B2B7EDD790FA1273606D943237EDBF8BBB0
"#;
        let h = sha256_from_release_body(body, "zapp_0.2.6_x64-setup.exe").unwrap();
        assert_eq!(
            h,
            "ca635f8f9c73a2a5bb73a5ca3b967b2b7edd790fa1273606d943237edbf8bbb0"
        );
    }
}
