use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::{AppError, AppResult};



const USER_LIST_SEEDS: &[(&str, &str)] = &[
    (
        "lists/list-general-user.txt",
        "# Never leave this file empty\r\ndomain.example.abc\r\n",
    ),
    (
        "lists/list-exclude-user.txt",
        "domain.example.abc\r\n",
    ),
    (
        "lists/ipset-exclude-user.txt",
        "203.0.113.113/32\r\n",
    ),
];


pub fn ensure_flowseal_sidecars(root: &Path) -> AppResult<()> {
    if !root.exists() {
        return Ok(());
    }

    let lists = root.join("lists");
    fs::create_dir_all(&lists).map_err(|e| AppError::from(e.to_string()))?;

    for (rel, contents) in USER_LIST_SEEDS {
        let path = root.join(rel);
        if path.is_file() {
            continue;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| AppError::from(e.to_string()))?;
        }
        let mut f = fs::File::create(&path).map_err(|e| AppError::from(e.to_string()))?;
        f.write_all(contents.as_bytes())
            .map_err(|e| AppError::from(e.to_string()))?;
        log::info!("created Flowseal sidecar {}", path.display());
    }

    Ok(())
}


pub fn pack_payloads_present(root: &Path) -> bool {
    let bin = root.join("bin");
    if !bin.join("winws.exe").is_file() {
        return false;
    }
    if !bin.join("WinDivert.dll").is_file() || !bin.join("WinDivert64.sys").is_file() {
        return false;
    }

    let Ok(entries) = fs::read_dir(&bin) else {
        return false;
    };
    entries.flatten().any(|e| {
        e.path()
            .extension()
            .and_then(|x| x.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("bin"))
    })
}



pub fn cleanup_old_engine_packs(engine_root: &Path, keep: usize) {
    if keep == 0 || !engine_root.is_dir() {
        return;
    }

    let Ok(entries) = fs::read_dir(engine_root) else {
        return;
    };

    let mut version_dirs: Vec<(PathBuf, Vec<u32>)> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        if name.eq_ignore_ascii_case("current") || name.eq_ignore_ascii_case("downloads") {
            continue;
        }

        let is_pack = path.join("bin").join("winws.exe").is_file()
            || path.join("bin").join("nfqws").is_file();
        let has_nested = fs::read_dir(&path).ok().is_some_and(|rd| {
            rd.flatten().any(|c| {
                let p = c.path();
                p.is_dir()
                    && (p.join("bin").join("winws.exe").is_file()
                        || p.join("bin").join("nfqws").is_file())
            })
        });
        if !is_pack && !has_nested {
            continue;
        }
        let key = version_sort_key(&name);
        version_dirs.push((path, key));
    }

    version_dirs.sort_by(|a, b| b.1.cmp(&a.1));
    for (path, _) in version_dirs.into_iter().skip(keep) {
        match fs::remove_dir_all(&path) {
            Ok(()) => log::info!("removed old engine pack {}", path.display()),
            Err(e) => log::warn!(
                "could not remove old engine pack {}: {e}",
                path.display()
            ),
        }
    }
}

fn version_sort_key(name: &str) -> Vec<u32> {
    const PREFIX: &str = "zapret-discord-youtube-";
    let ver = name.strip_prefix(PREFIX).unwrap_or(name);
    ver.split('.')
        .filter_map(|p| {
            p.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .ok()
        })
        .collect()
}
