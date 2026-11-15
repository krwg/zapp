// ╔════════════════════════════════════════════════════╗
// ║                     commands                     ║
// ╚════════════════════════════════════════════════════╝

use tauri::{AppHandle, Emitter, State};

use crate::error::AppResult;
use crate::model::*;
use crate::state::AppState;

fn quit_after_installer(app: &AppHandle) {
    let _ = app.emit("app:quit-for-update", ());
    let handle = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(900));
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

#[tauri::command]
pub fn update_channel(
    app: AppHandle,
    state: State<'_, AppState>,
    channel_id: UpdateChannelId,
) -> AppResult<UpdateChannel> {
    let (channel, restart) = state.update_channel(channel_id)?;
    if restart {
        state.history.push(
            "updates",
            "Application installer launched — quitting so files can update",
        );
        quit_after_installer(&app);
    }
    Ok(channel)
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
