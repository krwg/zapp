// ╔════════════════════════════════════════════════════╗
// ║                        tray                      ║
// ╚════════════════════════════════════════════════════╝

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime,
};

use crate::model::EngineStatus;
use crate::state::AppState;

pub fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let show_i = MenuItem::with_id(app, "show", "Открыть zapp", true, None::<&str>)?;
    let toggle_i = MenuItem::with_id(app, "toggle", "Вкл / Выкл обход", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit_i = MenuItem::with_id(app, "quit", "Выйти", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_i, &toggle_i, &sep, &quit_i])?;

    let icon = app
        .default_window_icon()
        .cloned()
        .expect("default window icon");

    let _tray = TrayIconBuilder::with_id("main")
        .icon(icon)
        .menu(&menu)
        .tooltip("zapp — обход DPI")
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_main_window(app),
            "toggle" => {
                if let Some(state) = app.try_state::<AppState>() {
                    let running = matches!(state.engine.status(), EngineStatus::Running);
                    match state.engine.set_running(!running) {
                        Ok(_) => {
                            state.history.push(
                                "engine",
                                if running {
                                    "Engine stopped (tray)"
                                } else {
                                    "Engine started (tray)"
                                },
                            );
                            let _ = app.emit("engine:tray-toggled", ());
                        }
                        Err(e) => log::warn!("tray toggle failed: {e}"),
                    }
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}
