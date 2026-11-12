// ╔════════════════════════════════════════════════════╗
// ║                   diagnostics                    ║
// ╚════════════════════════════════════════════════════╝

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};

use crate::engine::{probe_matrix_host, score_probes};
use crate::model::*;
use crate::state::AppState;
use crate::util::{new_id, now_iso};

fn diag_targets() -> Vec<(&'static str, u16)> {
    vec![("youtube.com", 443), ("discord.com", 443)]
}

pub fn run_diagnostics_worker(app: AppHandle, cancel: Arc<Mutex<bool>>) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };

    // One UAC for helper, then all profile cycles are cheap IPC.
    if let Err(e) = crate::helper::ensure_helper(state.engine.app_data()) {
        let mut store = state.inner.lock().expect("state lock");
        store.diagnostics.phase = DiagnosticPhase::Failed;
        store.diagnostics.current_strategy = None;
        store.diagnostics.finished_at = Some(now_iso());
        store.diagnostics.is_demo = false;
        state.history.push("diagnostics", e.message());
        let _ = app.emit("diagnostics:completed", ());
        return;
    }

    let profiles: Vec<(String, String, String)> = {
        let store = state.inner.lock().expect("state lock");
        store
            .profiles
            .iter()
            // Prefer stable generals first; EXP last to reduce early load.
            .map(|p| (p.id.clone(), p.name.clone(), p.strategy_type.clone()))
            .collect()
    };

    let mut profiles = profiles;
    profiles.sort_by(|a, b| {
        let score = |n: &str| {
            let l = n.to_ascii_lowercase();
            if l.contains("fake tls auto") {
                0
            } else if l.contains("exp") {
                2
            } else {
                1
            }
        };
        score(&a.1).cmp(&score(&b.1)).then(a.1.cmp(&b.1))
    });

    let total = profiles.len() as u32;
    let was_running = matches!(state.engine.status(), EngineStatus::Running);
    let previous = state.engine.active_strategy();

    {
        let mut store = state.inner.lock().expect("state lock");
        store.diagnostics.phase = DiagnosticPhase::Preparing;
        store.diagnostics.total_count = total;
        store.diagnostics.tested_count = 0;
        store.diagnostics.working_count = 0;
        store.diagnostics.results.clear();
        store.diagnostics.is_demo = false;
        store.diagnostics.current_strategy = Some("Preparing helper…".into());
        store.diagnostics.progress = 2;
    }
    let _ = app.emit("diagnostics:progress", ());
    state.history.push("diagnostics", "Started profile diagnostics via helper");

    let _ = state.engine.set_running(false);
    thread::sleep(Duration::from_millis(200));

    {
        let mut store = state.inner.lock().expect("state lock");
        store.diagnostics.phase = DiagnosticPhase::Testing;
        store.diagnostics.current_strategy = Some("Testing strategies…".into());
    }
    let _ = app.emit("diagnostics:progress", ());

    let timeout = {
        let store = state.inner.lock().expect("state lock");
        Duration::from_millis(store.settings.network_timeout_ms.max(1500) as u64)
    };

    let mut best: Option<(String, u32)> = None;

    for (index, (profile_id, name, strategy_type)) in profiles.into_iter().enumerate() {
        if *cancel.lock().expect("flag") {
            finalize_cancelled(&app, &state);
            restore_previous(&state, was_running, previous.as_deref());
            return;
        }

        {
            let mut store = state.inner.lock().expect("state lock");
            store.diagnostics.current_strategy = Some(name.clone());
            store.diagnostics.progress =
                (((index as u32) * 90) / total.max(1)).saturating_add(5);
        }
        let _ = app.emit("diagnostics:progress", ());

        // Helper already elevated — start_no_elevate talks to it without new UAC.
        let start_res = state.engine.start_no_elevate(Some(&name));
        // Give WinDivert a moment before probing.
        thread::sleep(Duration::from_millis(450));

        let mut probes = Vec::new();
        if start_res.is_ok() {
            let budget = timeout.min(Duration::from_millis(1800));
            for (host, _) in diag_targets() {
                // Prefer HTTPS as the adaptive signal; TCP as fallback score.
                probes.extend(probe_matrix_host(host, budget));
            }
        }

        let _ = state.engine.stop();
        thread::sleep(Duration::from_millis(150));

        let score = if start_res.is_ok() {
            score_probes(&probes)
        } else {
            0
        };
        let success = score >= 50;
        let latency_ms = probes.iter().filter_map(|p| p.latency_ms).min();
        let notes = if let Err(e) = start_res {
            format!("Start failed: {e}")
        } else if success {
            let ok = probes.iter().filter(|p| p.ok).count();
            format!("{ok}/{} targets reachable", probes.len())
        } else {
            probes
                .iter()
                .find(|p| !p.ok)
                .map(|p| p.note.clone())
                .unwrap_or_else(|| "Targets unreachable".into())
        };

        let row = DiagnosticStrategyResult {
            id: new_id("r"),
            name: name.clone(),
            strategy_type,
            success,
            latency_ms,
            score,
            notes,
            tested_at: now_iso(),
        };

        {
            let mut store = state.inner.lock().expect("state lock");
            if let Some(p) = store.profiles.iter_mut().find(|p| p.id == profile_id) {
                p.last_checked_at = Some(now_iso());
            }
            store.diagnostics.results.push(row);
            store.diagnostics.tested_count = (index as u32) + 1;
            store.diagnostics.working_count = store
                .diagnostics
                .results
                .iter()
                .filter(|r| r.success)
                .count() as u32;
            store.diagnostics.progress =
                ((((index as u32) + 1) * 90) / total.max(1)).saturating_add(5);
        }
        let _ = app.emit("diagnostics:progress", ());

        if success {
            match &best {
                Some((_, best_score)) if score <= *best_score => {}
                _ => best = Some((name, score)),
            }
        }
    }

    if *cancel.lock().expect("flag") {
        finalize_cancelled(&app, &state);
        restore_previous(&state, was_running, previous.as_deref());
        return;
    }

    {
        let mut store = state.inner.lock().expect("state lock");
        store.diagnostics.phase = DiagnosticPhase::Analyzing;
        store.diagnostics.current_strategy = Some("Selecting best strategy…".into());
        store.diagnostics.progress = 96;
        // Best first.
        store.diagnostics.results.sort_by(|a, b| b.score.cmp(&a.score));
    }
    let _ = app.emit("diagnostics:progress", ());
    thread::sleep(Duration::from_millis(250));

    if let Some((best_name, score)) = best {
        apply_best(&state, &best_name);
        state.history.push(
            "diagnostics",
            format!("Selected best strategy '{best_name}' (score {score})"),
        );
        // Auto-enable the winning strategy — one-click UX.
        let _ = state.engine.start_no_elevate(Some(&best_name));
    } else {
        state.history.push(
            "diagnostics",
            "No working strategy found; restoring previous state",
        );
        restore_previous(&state, was_running, previous.as_deref());
    }

    {
        let mut store = state.inner.lock().expect("state lock");
        store.diagnostics.phase = DiagnosticPhase::Completed;
        store.diagnostics.progress = 100;
        store.diagnostics.current_strategy = None;
        store.diagnostics.finished_at = Some(now_iso());
        store.diagnostics.is_demo = false;
    }
    let _ = app.emit("diagnostics:completed", ());
}

fn apply_best(state: &AppState, best_name: &str) {
    let mut store = state.inner.lock().expect("state lock");
    for p in store.profiles.iter_mut() {
        let active = p.name == best_name;
        p.is_active = active;
        if active {
            p.is_auto_selected = true;
            p.last_checked_at = Some(now_iso());
        }
    }
    state.engine.set_active_strategy(best_name.to_string());
}

fn restore_previous(state: &AppState, was_running: bool, previous: Option<&str>) {
    if let Some(name) = previous {
        state.engine.set_active_strategy(name.to_string());
        let mut store = state.inner.lock().expect("state lock");
        for p in store.profiles.iter_mut() {
            p.is_active = p.name == name;
        }
    }
    if was_running {
        if let Some(name) = previous {
            let _ = state.engine.start_no_elevate(Some(name));
        } else {
            let _ = state.engine.start_no_elevate(None);
        }
    }
}

fn finalize_cancelled(app: &AppHandle, state: &AppState) {
    let mut store = state.inner.lock().expect("state lock");
    store.diagnostics.phase = DiagnosticPhase::Cancelled;
    store.diagnostics.current_strategy = None;
    store.diagnostics.finished_at = Some(now_iso());
    state.history.push("diagnostics", "Diagnostics cancelled");
    let _ = app.emit("diagnostics:completed", ());
}
