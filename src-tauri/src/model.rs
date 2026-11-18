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
    /// Best (min) critical latency — shown as Response in the collapsed UI.
    pub latency_ms: Option<u32>,
    pub latency_median_ms: Option<u32>,
    pub latency_max_ms: Option<u32>,
    pub score: u32,
    pub soft_score: u32,
    pub services: Vec<DiagnosticServiceScore>,
    pub hosts: Vec<DiagnosticHostDetail>,
    pub notes: String,
    pub tested_at: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum UpdateChannelId {
    Application,
    Engine,
    Strategies,
    TelegramProxy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelegramProxyState {
    pub status: EngineStatus,
    pub installed: bool,
    pub version: String,
    pub host: String,
    pub port: u16,
    pub session_started_at: Option<String>,
    pub uptime_seconds: u64,
    pub last_error: Option<String>,
    pub proxy_link: String,
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
    /// Stop at the first strategy that passes YouTube + Discord.
    Fast,
    /// Rank strategies; keep searching for a stronger score.
    Thorough,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub appearance: ThemeMode,
    pub start_with_system: bool,
    pub minimize_to_tray: bool,
    pub auto_check_updates: bool,
    pub auto_install_updates: bool,
    pub auto_diagnostics: bool,
    pub diagnostics_interval_hours: u32,
    pub diagnostics_mode: DiagnosticsMode,
    pub network_timeout_ms: u32,
    pub network_retries: u32,
    pub advanced_logging: bool,
    pub engine_path: Option<String>,
    pub strategies_path: Option<String>,
    pub telegram_proxy_enabled: bool,
    pub telegram_offer_link_on_start: bool,
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
    pub telegram: TelegramProxyState,
    pub active_profile: Option<Profile>,
    pub profiles: Vec<Profile>,
    pub services: Vec<MonitoredService>,
    pub system: SystemInfo,
    pub diagnostics: DiagnosticRun,
    pub updates: Vec<UpdateChannel>,
    pub settings: AppSettings,
    pub about: AppAbout,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPatch {
    pub appearance: Option<ThemeMode>,
    pub start_with_system: Option<bool>,
    pub minimize_to_tray: Option<bool>,
    pub auto_check_updates: Option<bool>,
    pub auto_install_updates: Option<bool>,
    pub auto_diagnostics: Option<bool>,
    pub diagnostics_interval_hours: Option<u32>,
    pub diagnostics_mode: Option<DiagnosticsMode>,
    pub network_timeout_ms: Option<u32>,
    pub network_retries: Option<u32>,
    pub advanced_logging: Option<bool>,
    pub engine_path: Option<Option<String>>,
    pub strategies_path: Option<Option<String>>,
    pub telegram_proxy_enabled: Option<bool>,
    pub telegram_offer_link_on_start: Option<bool>,
}
