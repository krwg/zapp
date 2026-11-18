// ╔════════════════════════════════════════════════════╗
// ║                     commands                     ║
// ╚════════════════════════════════════════════════════╝

use tauri::{AppHandle, Emitter, Manager, State};

use crate::error::AppResult;
use crate::model::*;
use crate::state::AppState;

fn quit_after_installer(app: &AppHandle) {
    let _ = app.emit("app:quit-for-update", ());
    let handle = app.clone();
    std::thread::spawn(move || {
        if let Some(state) = handle.try_state::<AppState>() {
            state.shutdown_for_exit();
        }
        std::thread::sleep(std::time::Duration::from_millis(400));
        handle.exit(0);
    });
}

#[tauri::command]
pub fn get_snapshot(state: State<'_, AppState>) -> AppSnapshot {
    state.snapshot()
}

#[tauri::command]
pub fn get_diagnostics(state: State<'_, AppState>) -> DiagnosticRun {
    state.diagnostics()
}

#[tauri::command]
pub fn get_updates(state: State<'_, AppState>) -> Vec<UpdateChannel> {
    state.updates.list()
}

#[tauri::command]
pub fn set_engine_running(
    state: State<'_, AppState>,
    running: bool,
) -> AppResult<EngineStatus> {
    let status = state.engine.set_running(running)?;
    state.history.push(
        "engine",
        if running {
            "Engine started (hidden winws)"
        } else {
            "Engine stopped"
        },
    );
    Ok(status)
}

#[tauri::command]
pub fn set_telegram_proxy_running(
    state: State<'_, AppState>,
    running: bool,
) -> AppResult<TelegramProxyState> {
    if running && !state.tg_proxy.is_installed() {
        // Best-effort install from GitHub before first start.
        let _ = state.update_channel(UpdateChannelId::TelegramProxy);
    }
    let status = state.tg_proxy.set_running(running)?;
    state.history.push(
        "telegram",
        if running {
            "Telegram proxy started"
        } else {
            "Telegram proxy stopped"
        },
    );
    let _ = status;
    Ok(state.tg_proxy.snapshot_state())
}

#[tauri::command]
pub fn get_telegram_proxy_link(state: State<'_, AppState>) -> String {
    state.tg_proxy.proxy_link()
}

#[tauri::command]
pub fn set_active_profile(
    state: State<'_, AppState>,
    profile_id: String,
) -> AppResult<Profile> {
    state.set_active_profile(&profile_id)
}

#[tauri::command]
pub fn start_diagnostics(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<DiagnosticRun> {
    state.start_diagnostics(app)
}

#[tauri::command]
pub fn cancel_diagnostics(state: State<'_, AppState>) -> AppResult<DiagnosticRun> {
    state.cancel_diagnostics()
}

#[tauri::command]
pub fn select_diagnostic_strategy(
    state: State<'_, AppState>,
    result_id: String,
) -> AppResult<Profile> {
    state.select_diagnostic_strategy(&result_id)
}

#[tauri::command]
pub fn check_service(
    state: State<'_, AppState>,
    service_id: String,
) -> AppResult<MonitoredService> {
    state.check_service(&service_id)
}

#[tauri::command]
pub fn check_all_services(state: State<'_, AppState>) -> AppResult<Vec<MonitoredService>> {
    state.check_all_services()
}

#[tauri::command]
pub fn check_updates(state: State<'_, AppState>) -> AppResult<Vec<UpdateChannel>> {
    state.check_updates()
}

/// Starts the channel update on a background thread and returns immediately while
/// status is Downloading / Installing (UI polls `get_updates`).
#[tauri::command]
pub fn update_channel(
    app: AppHandle,
    state: State<'_, AppState>,
    channel_id: UpdateChannelId,
) -> AppResult<UpdateChannel> {
    let current = state
        .updates
        .list()
        .into_iter()
        .find(|c| c.id == channel_id)
        .ok_or_else(|| crate::error::AppError::from("Update channel not found"))?;

    if matches!(
        current.status,
        UpdateChannelStatus::Downloading | UpdateChannelStatus::Installing
    ) {
        return Ok(current);
    }

    // Mark Downloading synchronously so the first UI poll sees progress.
    state.updates.mark_downloading(channel_id.clone())?;

    let handle = app.clone();
    let id_for_worker = channel_id.clone();
    std::thread::spawn(move || {
        let Some(state) = handle.try_state::<AppState>() else {
            return;
        };
        match state.update_channel(id_for_worker.clone()) {
            Ok((_channel, restart)) => {
                if restart {
                    state.history.push(
                        "updates",
                        "Application installer launched — quitting so files can update",
                    );
                    quit_after_installer(&handle);
                }
            }
            Err(e) => {
                log::warn!("update_channel({id_for_worker:?}) failed: {e}");
                state.updates.mark_error(id_for_worker, e.to_string());
            }
        }
    });

    Ok(state
        .updates
        .list()
        .into_iter()
        .find(|c| c.id == channel_id)
        .unwrap_or(current))
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    patch: SettingsPatch,
) -> AppSettings {
    let sync_autostart = patch.start_with_system;
    let settings = state.update_settings(patch);
    if let Some(enabled) = sync_autostart {
        use tauri_plugin_autostart::ManagerExt;
        let autolaunch = app.autolaunch();
        let result = if enabled {
            autolaunch.enable()
        } else {
            autolaunch.disable()
        };
        if let Err(e) = result {
            log::warn!("autostart sync failed: {e}");
        }
    }
    settings
}

#[tauri::command]
pub fn set_theme(state: State<'_, AppState>, mode: ThemeMode) -> ThemeMode {
    state.set_theme(mode)
}

#[tauri::command]
pub fn run_preflight(state: State<'_, AppState>) -> crate::engine::PreflightReport {
    let layout = state.engine.layout();
    let strategy = state
        .engine
        .active_strategy()
        .unwrap_or_else(|| "general (FAKE TLS AUTO)".into());
    crate::engine::run_preflight(&layout.root, &strategy)
}

#[tauri::command]
pub fn reset_engine(state: State<'_, AppState>) -> AppResult<EngineStatus> {
    let status = state.engine.reset()?;
    state.history.push("engine", "Hard reset winws / WinDivert");
    Ok(status)
}

#[tauri::command]
pub fn open_external_url(url: String) -> AppResult<()> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &url])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| crate::error::AppError::from(e.to_string()))?;
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        let _ = url;
        Err(crate::error::AppError::from("open_external_url is Windows-only"))
    }
}

#[tauri::command]
pub fn get_recent_errors(state: State<'_, AppState>) -> Vec<String> {
    state
        .history
        .list(32)
        .into_iter()
        .filter(|e| {
            let kind = e.kind.to_ascii_lowercase();
            let msg = e.message.to_ascii_lowercase();
            kind.contains("error")
                || kind.contains("fail")
                || msg.contains("fail")
                || msg.contains("error")
        })
        .take(8)
        .map(|e| format!("[{}] {}", e.kind, e.message))
        .collect()
}
