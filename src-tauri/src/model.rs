// ╔════════════════════════════════════════════════════╗
// ║                      model                       ║
// ╚════════════════════════════════════════════════════╝

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EngineStatus {
    Running,
    Stopped,
    Starting,
    Stopping,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AvailabilityStatus {
    Available,
    Unavailable,
    Checking,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UpdateChannelStatus {
    UpToDate,
    Available,
    Downloading,
    Installing,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DiagnosticPhase {
    Idle,
    Preparing,
    Testing,
    Analyzing,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum StrategySource {
    Bundled,
    Remote,
    Auto,
    User,
    Imported,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ProfileKind {
    General,
    Youtube,
    Discord,
    Custom,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionState {
    pub status: EngineStatus,
    pub strategy_name: Option<String>,
    pub strategy_id: Option<String>,
    pub session_started_at: Option<String>,
    pub uptime_seconds: u64,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub kind: ProfileKind,
    pub strategy_type: String,
    pub source: StrategySource,
    pub description: String,
    pub is_active: bool,
    pub is_auto_selected: bool,
    pub last_checked_at: Option<String>,
    pub last_updated_at: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceCheckHistoryItem {
    pub id: String,
    pub checked_at: String,
    pub status: AvailabilityStatus,
    pub latency_ms: Option<u32>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitoredService {
    pub id: String,
    pub name: String,
    pub host: String,
    pub status: AvailabilityStatus,
    pub last_checked_at: Option<String>,
    pub latency_ms: Option<u32>,
    pub diagnostic_summary: String,
    pub history: Vec<ServiceCheckHistoryItem>,
    pub is_built_in: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub app_version: String,
    pub engine_version: String,
    pub strategies_version: String,
    pub last_update_check_at: Option<String>,
    pub auto_update_enabled: bool,
    pub elevated: bool,

    pub engine_installed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticHostDetail {
    pub host: String,
    pub service: String,
    pub ok: bool,
    pub latency_ms: Option<u32>,
    pub note: String,
    pub critical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticServiceScore {
    pub service: String,
    pub score: u32,
    pub ok: bool,
    pub latency_ms: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticStrategyResult {
    pub id: String,
    pub name: String,
    pub strategy_type: String,
    pub success: bool,

    pub latency_ms: Option<u32>,
    pub latency_median_ms: Option<u32>,
    pub latency_max_ms: Option<u32>,
    pub score: u32,
    pub soft_score: u32,
    pub services: Vec<DiagnosticServiceScore>,
    pub hosts: Vec<DiagnosticHostDetail>,
    pub notes: String,
    pub tested_at: String,
    #[serde(default)]
    pub path_verdict: Option<PathVerdict>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathVerdict {
    pub youtube: AvailabilityStatus,
    pub discord_web: AvailabilityStatus,
    pub discord_voice: AvailabilityStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkingModeEntry {
    pub strategy_name: String,
    pub score: u32,
    pub soft_score: u32,
    pub network_key: Option<String>,
    pub tested_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictInfo {
    pub images: Vec<String>,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticRun {
    pub id: String,
    pub phase: DiagnosticPhase,
    pub progress: u32,
    pub current_strategy: Option<String>,
    pub tested_count: u32,
    pub total_count: u32,
    pub working_count: u32,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub results: Vec<DiagnosticStrategyResult>,
    pub is_demo: bool,

    #[serde(default)]
    pub path_verdict: Option<PathVerdict>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum UpdateChannelId {
    Application,
    Engine,
    Strategies,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateChannel {
    pub id: UpdateChannelId,
    pub title: String,
    pub description: String,
    pub current_version: String,
    pub latest_version: String,
    pub status: UpdateChannelStatus,
    pub last_checked_at: Option<String>,
    pub download_progress: Option<u32>,
    pub release_notes: String,
    pub independent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DiagnosticsMode {

    Fast,

    Thorough,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub appearance: ThemeMode,
    pub start_with_system: bool,
    pub minimize_to_tray: bool,

    #[serde(default = "default_true")]
    pub restore_bypass_on_login: bool,

    #[serde(default = "default_true")]
    pub restore_bypass_on_resume: bool,

    #[serde(default = "default_true")]
    pub restore_bypass_on_network_change: bool,
    pub auto_check_updates: bool,
    pub auto_install_updates: bool,
    pub auto_diagnostics: bool,
    pub diagnostics_interval_hours: u32,
    pub diagnostics_mode: DiagnosticsMode,

    #[serde(default = "default_watchdog_minutes")]
    pub watchdog_interval_minutes: u32,
    #[serde(default = "default_true")]
    pub watchdog_enabled: bool,

    #[serde(default)]
    pub watchdog_auto_recheck: bool,

    #[serde(default = "default_true")]
    pub reapply_working_mode_on_network_change: bool,
    #[serde(default)]
    pub wizard_completed: bool,

    #[serde(default = "default_ui_locale")]
    pub ui_locale: String,
    pub network_timeout_ms: u32,
    pub network_retries: u32,
    pub advanced_logging: bool,
    pub engine_path: Option<String>,
    pub strategies_path: Option<String>,
}

fn default_watchdog_minutes() -> u32 {
    5
}

fn default_true() -> bool {
    true
}

fn default_ui_locale() -> String {
    "en".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppAbout {
    pub name: String,
    pub version: String,
    pub license: String,
    pub authors: Vec<String>,
    pub github_url: String,
    pub homepage_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshot {
    pub connection: ConnectionState,
    pub active_profile: Option<Profile>,
    pub profiles: Vec<Profile>,
    pub services: Vec<MonitoredService>,
    pub system: SystemInfo,
    pub diagnostics: DiagnosticRun,
    pub updates: Vec<UpdateChannel>,
    pub settings: AppSettings,
    pub about: AppAbout,
    #[serde(default)]
    pub working_modes: Vec<WorkingModeEntry>,
    #[serde(default)]
    pub conflicts: Option<ConflictInfo>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPatch {
    pub appearance: Option<ThemeMode>,
    pub start_with_system: Option<bool>,
    pub minimize_to_tray: Option<bool>,
    pub restore_bypass_on_login: Option<bool>,
    pub restore_bypass_on_resume: Option<bool>,
    pub restore_bypass_on_network_change: Option<bool>,
    pub auto_check_updates: Option<bool>,
    pub auto_install_updates: Option<bool>,
    pub auto_diagnostics: Option<bool>,
    pub diagnostics_interval_hours: Option<u32>,
    pub diagnostics_mode: Option<DiagnosticsMode>,
    pub watchdog_interval_minutes: Option<u32>,
    pub watchdog_enabled: Option<bool>,
    pub watchdog_auto_recheck: Option<bool>,
    pub reapply_working_mode_on_network_change: Option<bool>,
    pub wizard_completed: Option<bool>,
    pub ui_locale: Option<String>,
    pub network_timeout_ms: Option<u32>,
    pub network_retries: Option<u32>,
    pub advanced_logging: Option<bool>,
    pub engine_path: Option<Option<String>>,
    pub strategies_path: Option<Option<String>>,
}
