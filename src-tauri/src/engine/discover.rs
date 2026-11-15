// ╔════════════════════════════════════════════════════╗
// ║                 engine/discover                  ║
// ╚════════════════════════════════════════════════════╝

use std::fs;
use std::path::{Path, PathBuf};

use crate::engine::managed::EngineLayout;

const FLOWSEAL_PREFIX: &str = "zapret-discord-youtube-";

pub fn resolve_engine_layout(app_data: &Path) -> EngineLayout {
    let layout = resolve_engine_layout_inner(app_data);
    let _ = crate::engine::pack::ensure_flowseal_sidecars(&layout.root);
    layout
}

fn resolve_engine_layout_inner(app_data: &Path) -> EngineLayout {
    for key in ["ZAPP_ENGINE_DIR", "HAPP_ZAPRET_ENGINE_DIR"] {
        if let Ok(dir) = std::env::var(key) {
            let path = PathBuf::from(dir);
            if looks_like_pack(&path) {
                return EngineLayout::from_pack(path);
            }
        }
    }

    let current = app_data.join("engine").join("current");
    if looks_like_pack(&current) {
        return EngineLayout::from_pack(current);
    }

    if let Some(layout) = newest_under(app_data.join("engine")) {
        return layout;
    }

    for candidate in project_candidates() {
        if looks_like_pack(&candidate) {
            return EngineLayout::from_pack(candidate);
        }
    }

    EngineLayout::under_app_data(app_data.to_path_buf(), "0.0.0")
}

fn project_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();

    
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    out.push(manifest_dir.join("../vendor/zapret"));
    out.push(manifest_dir.join("../../vendor/zapret"));

    
    for base in [
        manifest_dir.join(".."),
        manifest_dir.join("../.."),
        manifest_dir.join("../../.."),
    ] {
        if let Ok(entries) = fs::read_dir(&base) {
            let mut packs: Vec<PathBuf> = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| {
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.starts_with(FLOWSEAL_PREFIX) && looks_like_pack(p))
                        .unwrap_or(false)
                })
                .collect();
            packs.sort_by(|a, b| version_key(b).cmp(&version_key(a)));
            out.extend(packs);
        }
    }

    out
}

fn newest_under(engine_root: PathBuf) -> Option<EngineLayout> {
    let entries = fs::read_dir(engine_root).ok()?;
    let mut packs: Vec<PathBuf> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() || path.file_name().and_then(|n| n.to_str()) == Some("current") {
            continue;
        }
        if looks_like_pack(&path) {
            packs.push(path);
            continue;
        }
        // Nested Flowseal zip layout: engine/1.10.4/zapret-discord-youtube-1.10.4/
        if let Ok(nested) = fs::read_dir(&path) {
            for child in nested.flatten() {
                let child_path = child.path();
                if looks_like_pack(&child_path) {
                    packs.push(child_path);
                }
            }
        }
    }
    packs.sort_by(|a, b| version_key(b).cmp(&version_key(a)));
    packs.into_iter().next().map(EngineLayout::from_pack)
}

pub fn looks_like_pack(path: &Path) -> bool {
    path.join("bin").join("winws.exe").is_file() || path.join("bin").join("nfqws").is_file()
}

pub fn read_local_version(root: &Path) -> String {
    let service = root.join("service.bat");
    if let Ok(text) = fs::read_to_string(&service) {
        for line in text.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("set \"LOCAL_VERSION=") {
                let ver = rest.trim_end_matches('"').trim();
                if !ver.is_empty() {
                    return ver.to_string();
                }
            }
            if let Some(rest) = line.strip_prefix("set LOCAL_VERSION=") {
                let ver = rest.trim().trim_matches('"');
                if !ver.is_empty() {
                    return ver.to_string();
                }
            }
        }
    }

    root.file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| n.strip_prefix(FLOWSEAL_PREFIX))
        .unwrap_or("unknown")
        .to_string()
}

fn version_key(path: &Path) -> Vec<u32> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    let ver = name
        .strip_prefix(FLOWSEAL_PREFIX)
        .unwrap_or(name);
    ver.split('.')
        .filter_map(|p| p.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().ok())
        .collect()
}

pub fn scan_strategy_profiles(root: &Path) -> Vec<crate::model::Profile> {
    use crate::model::{Profile, StrategySource};
    use crate::util::now_iso;

    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };

    let mut bats: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension().and_then(|e| e.to_str()) == Some("bat")
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| {
                        let lower = n.to_ascii_lowercase();
                        lower.starts_with("general") && lower != "service.bat"
                    })
                    .unwrap_or(false)
        })
        .collect();

    bats.sort_by(|a, b| {
        a.file_stem()
            .unwrap_or_default()
            .cmp(b.file_stem().unwrap_or_default())
    });

    let has_fake_tls = bats.iter().any(|p| {
        p.file_stem()
            .and_then(|s| s.to_str())
            .is_some_and(|n| n.eq_ignore_ascii_case("general (FAKE TLS AUTO)"))
    });

    let updated = now_iso();
    bats.into_iter()
        .enumerate()
        .map(|(idx, path)| {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("strategy")
                .to_string();
            let kind = classify_kind(&name);
            let is_default = name.eq_ignore_ascii_case("general (FAKE TLS AUTO)")
                || (idx == 0 && !has_fake_tls);
            Profile {
                id: format!("strategy-{}", sanitize_id(&name)),
                name: name.clone(),
                kind,
                strategy_type: "winws / flowseal".into(),
                source: StrategySource::Bundled,
                description: format!("Режим из набора Flowseal · {name}"),
                is_active: is_default,
                is_auto_selected: false,
                last_checked_at: None,
                last_updated_at: updated.clone(),
                tags: tags_for(&name),
            }
        })
        .collect()
}

fn sanitize_id(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn classify_kind(name: &str) -> crate::model::ProfileKind {
    use crate::model::ProfileKind;
    let lower = name.to_ascii_lowercase();
    if lower.contains("youtube") || lower.contains("google") {
        ProfileKind::Youtube
    } else if lower.contains("discord") {
        ProfileKind::Discord
    } else if lower.contains("fake tls auto") {
        ProfileKind::Auto
    } else {
        ProfileKind::General
    }
}

fn tags_for(name: &str) -> Vec<String> {
    let mut tags = vec!["flowseal".into(), "bundled".into()];
    let lower = name.to_ascii_lowercase();
    if lower.contains("fake") {
        tags.push("fake".into());
    }
    if lower.contains("alt") {
        tags.push("alt".into());
    }
    if lower.contains("exp") {
        tags.push("experimental".into());
    }
    tags
}
