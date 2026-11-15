// ╔════════════════════════════════════════════════════╗
// ║                       lib                        ║
// ╚════════════════════════════════════════════════════╝

mod commands;
mod diagnostics;
mod engine;
mod error;
pub mod helper;
mod history;
mod model;
mod state;
mod tray;
mod util;

use state::AppState;
use tauri::{Emitter, Manager, WindowEvent};

const ARG_MINIMIZED: &str = "--minimized";

fn launched_minimized() -> bool {
    std::env::args().any(|a| a == ARG_MINIMIZED)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::show_main_window(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![ARG_MINIMIZED]),
        ))
        .on_window_event(|window, event| {
            if window.label() != "main" {
                return;
            }
            if let WindowEvent::CloseRequested { api, .. } = event {
                let hide_to_tray = window
                    .app_handle()
                    .try_state::<AppState>()
                    .map(|s| s.snapshot().settings.minimize_to_tray)
                    .unwrap_or(true);
                if hide_to_tray {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let app_data = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::temp_dir().join("zapp"));
            std::fs::create_dir_all(&app_data).ok();

            let state = AppState::new(app_data);
            let layout = state.engine.layout();
            let elevated = engine::is_elevated();
            log::info!(
                "zapp 0.2 ready · engine {} · {} · installed={} · elevated={}",
                layout.version,
                layout.root.display(),
                layout.is_installed(),
                elevated
            );
            state.history.push(
                "app",
                format!(
                    "zapp started (engine {}, elevated={elevated})",
                    layout.version
                ),
            );

            let auto_check = state.snapshot().settings.auto_check_updates;
            let auto_install = state.snapshot().settings.auto_install_updates;
            let start_with_system = state.snapshot().settings.start_with_system;
            app.manage(state);

            tray::setup_tray(app.handle())?;

            {
                use tauri_plugin_autostart::ManagerExt;
                let autolaunch = app.autolaunch();
                if start_with_system {
                    if let Err(e) = autolaunch.enable() {
                        log::warn!("autostart enable failed: {e}");
                    }
                } else if let Err(e) = autolaunch.disable() {
                    log::warn!("autostart disable failed: {e}");
                }
            }

            if launched_minimized() {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            // First-run repair: Flowseal is not embedded in the installer.
            // Always ensure sidecars / download pack ASAP (don't wait for Updates UI).
            {
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    let Some(state) = handle.try_state::<AppState>() else {
                        return;
                    };
                    match state.bootstrap_engine_if_needed() {
                        Ok(true) => {
                            log::info!("Flowseal engine bootstrapped into app data");
                            let _ = handle.emit("engine:bootstrapped", ());
                        }
                        Ok(false) => log::info!("Flowseal engine pack OK"),
                        Err(e) => log::warn!("engine bootstrap failed: {e}"),
                    }
                });
            }

            if auto_check {
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    let Some(state) = handle.try_state::<AppState>() else {
                        return;
                    };
                    match state.check_updates_and_auto_install(auto_install) {
                        Ok(true) => {
                            log::info!("application installer launched — exiting for update");
                            let _ = handle.emit("app:quit-for-update", ());
                            std::thread::sleep(std::time::Duration::from_millis(900));
                            handle.exit(0);
                        }
                        Ok(false) => {
                            log::info!("update check finished (auto_install={auto_install})");
                        }
                        Err(e) => log::warn!("update check failed: {e}"),
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::get_diagnostics,
            commands::get_updates,
            commands::set_engine_running,
            commands::set_active_profile,
            commands::start_diagnostics,
            commands::cancel_diagnostics,
            commands::select_diagnostic_strategy,
            commands::check_service,
            commands::check_all_services,
            commands::check_updates,
            commands::update_channel,
            commands::update_settings,
            commands::set_theme,
            commands::run_preflight,
            commands::reset_engine,
        ])
        .run(tauri::generate_context!())
        .expect("error while running zapp");
}
