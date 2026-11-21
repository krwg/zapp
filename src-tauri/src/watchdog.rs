// ╔════════════════════════════════════════════════════╗
// ║                    watchdog                      ║
// ╚════════════════════════════════════════════════════╝

use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::engine::{probe_resolved_parallel, resolve_diag_targets, watchdog_targets};
use crate::model::EngineStatus;
use crate::state::AppState;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchdogEvent {
    pub kind: String,
    pub detail: String,
    pub auto_recheck: bool,
}

pub fn spawn_watchdog(app: AppHandle) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(30));
            let Some(state) = app.try_state::<AppState>() else {
                continue;
            };
            let snap = state.snapshot();
            if !snap.settings.watchdog_enabled || snap.settings.watchdog_interval_minutes == 0 {
                continue;
            }
            let session = crate::persist::load_session(state.engine.app_data());
            if !session.bypass_desired {
                continue;
            }
            if !matches!(snap.connection.status, EngineStatus::Running) {
                let _ = app.emit(
                    "watchdog:bypass_stale",
                    WatchdogEvent {
                        kind: "engine_down".into(),
                        detail: "Bypass should be on, but the engine is not running.".into(),
                        auto_recheck: snap.settings.watchdog_auto_recheck,
                    },
                );
                if snap.settings.watchdog_auto_recheck {
                    let _ = state.try_restore_bypass("watchdog");
                    let _ = state.start_diagnostics_with(app.clone(), true);
                }
                continue;
            }


            let timeout = Duration::from_millis(snap.settings.network_timeout_ms.max(1500) as u64);
            let targets = resolve_diag_targets(&watchdog_targets());
            let results = probe_resolved_parallel(&targets, timeout, None);
            let critical_fail = results.iter().any(|(t, probes)| {
                t.critical && probes.iter().all(|p| !p.ok)
            });
            if critical_fail {
                let _ = app.emit(
                    "watchdog:bypass_stale",
                    WatchdogEvent {
                        kind: "probe_fail".into(),
                        detail: "Sites look down while bypass is on — the path may have broken."
                            .into(),
                        auto_recheck: snap.settings.watchdog_auto_recheck,
                    },
                );
                if snap.settings.watchdog_auto_recheck {
                    let _ = state.start_diagnostics_with(app.clone(), true);
                }
            }


            let mins = snap.settings.watchdog_interval_minutes.max(1) as u64;
            let rest = mins.saturating_mul(60).saturating_sub(30);
            if rest > 0 {
                std::thread::sleep(Duration::from_secs(rest));
            }
        }
    });
}
