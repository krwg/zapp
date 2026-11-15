// ╔════════════════════════════════════════════════════╗
// ║                   diagnostics                    ║
// ╚════════════════════════════════════════════════════╝

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};

use crate::engine::{
    diag_targets, probe_targets_parallel, rank_score, score_by_service, strategy_prior_bonus,
};
use crate::model::*;
use crate::state::AppState;
use crate::util::{new_id, now_iso};

const PASS_THRESHOLD: u32 = 55;

pub fn run_diagnostics_worker(app: AppHandle, cancel: Arc<Mutex<bool>>) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };

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
            .map(|p| (p.id.clone(), p.name.clone(), p.strategy_type.clone()))
            .collect()
    };

    let mut profiles = profiles;
    profiles.sort_by(|a, b| {
        strategy_prior_bonus(&b.1)
            .cmp(&strategy_prior_bonus(&a.1))
            .then(a.1.cmp(&b.1))
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
    state
        .history
        .push("diagnostics", "Started adaptive profile diagnostics");

    let _ = state.engine.set_running(false);
    thread::sleep(Duration::from_millis(80));

    {
        let mut store = state.inner.lock().expect("state lock");
        store.diagnostics.phase = DiagnosticPhase::Testing;
        store.diagnostics.current_strategy = Some("Testing strategies…".into());
    }
    let _ = app.emit("diagnostics:progress", ());

    let timeout = {
        let store = state.inner.lock().expect("state lock");
        Duration::from_millis(store.settings.network_timeout_ms.max(900) as u64)
    };

    let targets = diag_targets();
    let mut best: Option<(String, u32)> = None;
    let mut tested = 0u32;

    for (index, (profile_id, name, strategy_type)) in profiles.into_iter().enumerate() {
        if *cancel.lock().expect("flag") {
            finalize_cancelled(&app, &state);
            restore_previous(&state, was_running, previous.as_deref());
            return;
        }

        // Early stop only when every critical service already passes strongly.
        // Prevents locking onto a YouTube-only winner while Discord is still broken.
        if let Some((_, best_score)) = &best {
            let remaining_exp_only = name.to_ascii_lowercase().contains("exp");
            if *best_score >= 92 && tested >= 3 && remaining_exp_only {
                break;
            }
            if *best_score >= 96 && tested >= 5 {
                break;
            }
        }

        {
            let mut store = state.inner.lock().expect("state lock");
            store.diagnostics.current_strategy = Some(name.clone());
            store.diagnostics.progress =
                (((index as u32) * 90) / total.max(1)).saturating_add(5);
        }
        let _ = app.emit("diagnostics:progress", ());

        let start_res = state.engine.start_no_elevate(Some(&name));
        // Give WinDivert filters a moment to attach before probing.
        thread::sleep(Duration::from_millis(200));

        let host_results = if start_res.is_ok() {
            let budget = timeout.min(Duration::from_millis(1100));
            probe_targets_parallel(&targets, budget)
        } else {
            Vec::new()
        };

        let _ = state.engine.stop();
        thread::sleep(Duration::from_millis(50));

        let adaptive = if start_res.is_ok() {
            score_by_service(&host_results, PASS_THRESHOLD)
        } else {
            crate::engine::AdaptiveScore {
                score: 0,
                success: false,
                services: Vec::new(),
                all_probes: Vec::new(),
            }
        };
        let score = rank_score(adaptive.score, &name);
        let success = adaptive.success && score >= PASS_THRESHOLD;
        let latency_ms = adaptive
            .all_probes
            .iter()
            .filter_map(|p| p.latency_ms)
            .min();

        let service_summary = adaptive
            .services
            .iter()
            .map(|s| {
                format!(
                    "{} {}{}",
                    s.service,
                    s.score,
                    if s.ok { "" } else { "!" }
                )
            })
            .collect::<Vec<_>>()
            .join(" · ");

        let notes = if let Err(e) = start_res {
            format!("Start failed: {e}")
        } else if success {
            let ok = adaptive.all_probes.iter().filter(|p| p.ok).count();
            let avg = adaptive
                .all_probes
                .iter()
                .filter_map(|p| p.latency_ms)
                .sum::<u32>()
                .checked_div(
                    adaptive
                        .all_probes
                        .iter()
                        .filter(|p| p.latency_ms.is_some())
                        .count()
                        .max(1) as u32,
                )
                .unwrap_or(0);
            format!(
                "{ok}/{} ok · avg {avg} ms · rank {score} · {service_summary}",
                adaptive.all_probes.len()
            )
        } else if !adaptive.services.is_empty() {
            let failed = adaptive
                .services
                .iter()
                .filter(|s| !s.ok)
                .map(|s| s.service.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            format!("Weak service(s): {failed} · {service_summary}")
        } else {
            adaptive
                .all_probes
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

        tested += 1;
        {
            let mut store = state.inner.lock().expect("state lock");
            if let Some(p) = store.profiles.iter_mut().find(|p| p.id == profile_id) {
                p.last_checked_at = Some(now_iso());
            }
            store.diagnostics.results.push(row);
            store.diagnostics.tested_count = tested;
            store.diagnostics.working_count = store
                .diagnostics
                .results
                .iter()
                .filter(|r| r.success)
                .count() as u32;
            store.diagnostics.progress =
                ((tested * 90) / total.max(1)).saturating_add(5);
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
        store
            .diagnostics
            .results
            .sort_by(|a, b| b.score.cmp(&a.score).then(a.latency_ms.cmp(&b.latency_ms)));
    }
    let _ = app.emit("diagnostics:progress", ());
    thread::sleep(Duration::from_millis(80));

    if let Some((best_name, score)) = best {
        apply_best(&state, &best_name);
        state.history.push(
            "diagnostics",
            format!("Selected '{best_name}' (adaptive score {score}; YouTube+Discord required)"),
        );
        let _ = state.engine.start_no_elevate(Some(&best_name));
    } else {
        state.history.push(
            "diagnostics",
            "No strategy passed both YouTube and Discord; restoring previous state",
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
