//! Keep a Flowseal pack usable for winws launches (sidecars the BAT expects).

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::error::{AppError, AppResult};

/// User lists that Flowseal `service.bat :load_user_lists` would create.
/// Our helper/winws path skips that bat, so we create them ourselves.
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

/// Create missing Flowseal sidecar files. Safe to call repeatedly.
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

/// True when pack looks complete enough to start (winws + divert + any .bin payload).
pub fn pack_payloads_present(root: &Path) -> bool {
    let bin = root.join("bin");
    if !bin.join("winws.exe").is_file() {
        return false;
    }
    if !bin.join("WinDivert.dll").is_file() || !bin.join("WinDivert64.sys").is_file() {
        return false;
    }
    // At least one fake payload used by strategies.
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
