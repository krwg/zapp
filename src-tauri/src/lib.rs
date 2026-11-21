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
mod network;
mod persist;
mod state;
mod tray;
mod util;
mod watchdog;
#[cfg(windows)]
mod win32;

use state::AppState;
use tauri::{Emitter, Manager, RunEvent, WindowEvent};

const ARG_MINIMIZED: &str = "--minimized";

fn launched_minimized() -> bool {
    std::env::args().any(|a| a == ARG_MINIMIZED)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(windows)]
    win32::maybe_attach_console();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::show_main_window(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![ARG_MINIMIZED]),
        ))
        .plugin(tauri_plugin_updater::Builder::new().build())
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

                    let _ = state.try_restore_bypass("login");
                });
            }

            if auto_check {
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    let Some(state) = handle.try_state::<AppState>() else {
                        return;
                    };
                    match state.check_updates_and_auto_install_with_app(&handle, auto_install) {
                        Ok(true) => {
                            log::info!("application installer launched — exiting for update");
                            let _ = handle.emit("app:quit-for-update", ());
                            state.engine.shutdown_for_exit();
                            std::thread::sleep(std::time::Duration::from_millis(400));
                            handle.exit(0);
                        }
                        Ok(false) => {
                            log::info!("update check finished (auto_install={auto_install})");
                        }
                        Err(e) => log::warn!("update check failed: {e}"),
                    }
                });
            }

            watchdog::spawn_watchdog(app.handle().clone());


            {
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    use std::time::{Duration, SystemTime};
                    let mut last = SystemTime::now();
                    let mut last_net = crate::network::current_fingerprint().key();
                    {
                        if let Some(state) = handle.try_state::<AppState>() {
                            let mut session =
                                crate::persist::load_session(state.engine.app_data());
                            if session.network_key.is_none() {
                                session.network_key = Some(last_net.clone());
                                crate::persist::save_session(state.engine.app_data(), &session);
                            } else if let Some(prev) = session.network_key.clone() {
                                last_net = prev;
                            }
                        }
                    }
                    loop {
                        std::thread::sleep(Duration::from_secs(5));
                        let now = SystemTime::now();
                        let gap = now.duration_since(last).unwrap_or_default();
                        last = now;
                        let Some(state) = handle.try_state::<AppState>() else {
                            continue;
                        };


                        if gap > Duration::from_secs(25) {
                            std::thread::sleep(Duration::from_secs(2));
                            let _ = state.try_restore_bypass("resume");
                            let _ = handle.emit("engine:restored", ());
                        }

                        let fp = crate::network::current_fingerprint();
                        let key = fp.key();
                        if key != last_net && fp.gateway.is_some() {
                            log::info!(
                                "network changed: {} -> {} (ssid={:?})",
                                last_net,
                                key,
                                fp.ssid
                            );
                            state.history.push(
                                "network",
                                format!(
                                    "Network changed (gw={}, ssid={})",
                                    fp.gateway.as_deref().unwrap_or("-"),
                                    fp.ssid.as_deref().unwrap_or("-")
                                ),
                            );
                            let mut session =
                                crate::persist::load_session(state.engine.app_data());
                            session.network_key = Some(key.clone());
                            crate::persist::save_session(state.engine.app_data(), &session);
                            last_net = key;
                            std::thread::sleep(Duration::from_secs(2));
                            crate::engine::invalidate_dns_cache();
                            if state.try_restore_bypass("network") {
                                let _ = handle.emit("engine:restored", ());
                            }
                            let _ = handle.emit("network:changed", ());
                        }
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
            commands::open_external_url,
            commands::get_recent_errors,
            commands::get_bug_report_bundle,
            commands::import_engine_pack,
            commands::apply_working_mode,
            commands::resolve_conflicts,
            commands::get_conflicts,
        ])
        .build(tauri::generate_context!())
        .expect("error while building zapp")
        .run(|app, event| {
            if matches!(event, RunEvent::Exit | RunEvent::ExitRequested { .. }) {
                if let Some(state) = app.try_state::<AppState>() {
                    state.engine.shutdown_for_exit();
                }
            }
        });
}
