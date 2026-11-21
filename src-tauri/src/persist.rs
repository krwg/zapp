// ╔════════════════════════════════════════════════════╗
// ║                     persist                      ║
// ╚════════════════════════════════════════════════════╝

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::model::AppSettings;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SessionState {
    /// User wants bypass on across reboot / sleep (cleared on explicit Stop / Quit).
    #[serde(default)]
    pub bypass_desired: bool,
    #[serde(default)]
    pub active_strategy: Option<String>,
    /// Last seen `gateway|ssid` fingerprint (§7 network memory).
    #[serde(default)]
    pub network_key: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

fn settings_path(app_data: &Path) -> PathBuf {
    app_data.join("settings.json")
}

fn session_path(app_data: &Path) -> PathBuf {
    app_data.join("session.json")
}

pub fn load_settings(app_data: &Path) -> Option<AppSettings> {
    let raw = fs::read_to_string(settings_path(app_data)).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn save_settings(app_data: &Path, settings: &AppSettings) {
    let _ = fs::create_dir_all(app_data);
    if let Ok(body) = serde_json::to_string_pretty(settings) {
        let _ = fs::write(settings_path(app_data), body);
    }
}

pub fn load_session(app_data: &Path) -> SessionState {
    fs::read_to_string(session_path(app_data))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_session(app_data: &Path, session: &SessionState) {
    let _ = fs::create_dir_all(app_data);
    if let Ok(body) = serde_json::to_string_pretty(session) {
        let _ = fs::write(session_path(app_data), body);
    }
}

pub fn set_bypass_desired(app_data: &Path, desired: bool, strategy: Option<String>) {
    let mut session = load_session(app_data);
    session.bypass_desired = desired;
    if strategy.is_some() {
        session.active_strategy = strategy;
    }
    session.updated_at = Some(crate::util::now_iso());
    save_session(app_data, &session);
}
