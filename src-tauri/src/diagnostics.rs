// ╔════════════════════════════════════════════════════╗
// ║                   diagnostics                    ║
// ╚════════════════════════════════════════════════════╝

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager};

use crate::engine::{
    diag_targets, kill_orphan_winws, latency_stats, probe_resolved_parallel,
    rank_score_full, resolve_diag_targets, run_preflight_scoped, score_by_service,
    soft_contribution, strategy_prior_bonus, PreflightScope,
};
use crate::model::*;
use crate::state::{AppState, Store};
use crate::util::{new_id, now_iso};

const PASS_THRESHOLD: u32 = 55;

struct Snapshot {
    was_running: bool,
    previous: Option<String>,
}

fn cancelled(cancel: &Arc<Mutex<bool>>) -> bool {
    *cancel.lock().unwrap_or_else(|e| e.into_inner())
}

fn superseded(state: &AppState, run_id: &str) -> bool {
    state.inner.lock().expect("state lock").diagnostics.id != run_id
}

fn alive(state: &AppState, run_id: &str, cancel: &Arc<Mutex<bool>>) -> bool {
    !cancelled(cancel) && !superseded(state, run_id)
}

fn mutate_if_alive(
    state: &AppState,
    run_id: &str,
    cancel: &Arc<Mutex<bool>>,
    f: impl FnOnce(&mut Store),
) -> bool {
    if cancelled(cancel) {
        return false;
    }
    let mut store = state.inner.lock().expect("state lock");
    if store.diagnostics.id != run_id || cancelled(cancel) {
        return false;
    }
    f(&mut store);
    true
}

/// Exit path when this run is no longer active: cancel → restore; superseded → silent stop.
fn bail(
    app: &AppHandle,
    state: &AppState,
    run_id: &str,
    cancel: &Arc<Mutex<bool>>,
    snapshot: &Snapshot,
) {
    let _ = state.engine.stop();
    if superseded(state, run_id) {
        return;
    }
    if !cancelled(cancel) {
        return;
    }
    restore_previous(state, snapshot);
    {
        let mut store = state.inner.lock().expect("state lock");
        if store.diagnostics.id != run_id {
            return;
        }
        store.diagnostics.phase = DiagnosticPhase::Cancelled;
        store.diagnostics.current_strategy = None;
        store.diagnostics.finished_at = Some(now_iso());
    }
    state.history.push("diagnostics", "Diagnostics cancelled");
    let _ = app.emit("diagnostics:completed", ());
}

pub fn run_diagnostics_worker(
    app: AppHandle,
    run_id: String,
    cancel: Arc<Mutex<bool>>,
    prefer_previous: bool,
) {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run_diagnostics_worker_inner(app.clone(), run_id.clone(), cancel.clone(), prefer_previous)
    }));
    if let Err(payload) = result {
        let msg = if let Some(s) = payload.downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "Smart check panicked".into()
        };
        log::error!("diagnostics worker panic: {msg}");
        if let Some(state) = app.try_state::<AppState>() {
            let snapshot = Snapshot {
                was_running: false,
                previous: state.engine.active_strategy(),
            };
            fail_run(&app, &state, &run_id, &cancel, &snapshot, &msg);
        }
    }
}

fn run_diagnostics_worker_inner(
    app: AppHandle,
    run_id: String,
    cancel: Arc<Mutex<bool>>,
    prefer_previous: bool,
) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };

    let snapshot = Snapshot {
        was_running: matches!(state.engine.status(), EngineStatus::Running),
        previous: state.engine.active_strategy(),
    };

    if let Err(e) = crate::helper::ensure_helper(state.engine.app_data()) {
        fail_run(&app, &state, &run_id, &cancel, &snapshot, e.message());
        return;
    }

    if !alive(&state, &run_id, &cancel) {
        bail(&app, &state, &run_id, &cancel, &snapshot);
        return;
    }

    let (profiles_raw, mode) = {
        let store = state.inner.lock().expect("state lock");
        let profiles: Vec<(String, String, String)> = store
            .profiles
            .iter()
            .map(|p| (p.id.clone(), p.name.clone(), p.strategy_type.clone()))
            .collect();
        (profiles, store.settings.diagnostics_mode.clone())
    };

    let mut profiles = profiles_raw;
    // Manual Smart check uses normal ranking. Prefer-previous is only for background re-check:
    // otherwise Fast mode re-selects the current (often UDP-broken) strategy and changes nothing.
    profiles.sort_by(|a, b| {
        let prev_ord = if prefer_previous {
            let a_prev = snapshot.previous.as_deref() == Some(a.1.as_str());
            let b_prev = snapshot.previous.as_deref() == Some(b.1.as_str());
            b_prev.cmp(&a_prev)
        } else {
            std::cmp::Ordering::Equal
        };
        prev_ord
            .then(strategy_prior_bonus(&b.1).cmp(&strategy_prior_bonus(&a.1)))
            .then(a.1.cmp(&b.1))
    });
    let total = profiles.len() as u32;

    if !mutate_if_alive(&state, &run_id, &cancel, |store| {
        store.diagnostics.phase = DiagnosticPhase::Preparing;
        store.diagnostics.total_count = total;
        store.diagnostics.tested_count = 0;
        store.diagnostics.working_count = 0;
        store.diagnostics.results.clear();
        store.diagnostics.is_demo = false;
        store.diagnostics.current_strategy = Some("Preparing helper…".into());
        store.diagnostics.progress = 2;
    }) {
        bail(&app, &state, &run_id, &cancel, &snapshot);
        return;
    }
    let _ = app.emit("diagnostics:progress", ());
    state.history.push(
        "diagnostics",
        format!(
            "Started adaptive diagnostics ({})",
            match mode {
                DiagnosticsMode::Fast => "fast",
                DiagnosticsMode::Thorough => "thorough",
            }
        ),
    );

    let _ = state.engine.set_running(false);

    // Our orphaned winws from a previous helper session must not look like Flowseal.
    // Clean orphans first, then env-preflight treats any leftover winws as foreign.
    kill_orphan_winws();
    thread::sleep(Duration::from_millis(60));

    {
        let layout = state.engine.layout();
        let sample = profiles
            .first()
            .map(|(_, n, _)| n.as_str())
            .unwrap_or("general (FAKE TLS AUTO)");
        let env = run_preflight_scoped(&layout.root, sample, PreflightScope::Environment);
        if let Some(fault) = env.blocking_fault {
            fail_run(
                &app,
                &state,
                &run_id,
                &cancel,
                &snapshot,
                fault.message(),
            );
            return;
        }
    }

    if !alive(&state, &run_id, &cancel) {
        bail(&app, &state, &run_id, &cancel, &snapshot);
        return;
    }

    if !mutate_if_alive(&state, &run_id, &cancel, |store| {
        store.diagnostics.phase = DiagnosticPhase::Testing;
        store.diagnostics.current_strategy = Some("Testing strategies…".into());
    }) {
        bail(&app, &state, &run_id, &cancel, &snapshot);
        return;
    }
    let _ = app.emit("diagnostics:progress", ());

    let timeout = {
        let store = state.inner.lock().expect("state lock");
        Duration::from_millis(store.settings.network_timeout_ms.max(900) as u64)
    };
    let targets = diag_targets();
    let resolved = resolve_diag_targets(&targets);
    let dns_ok = resolved.iter().any(|r| !r.addrs.is_empty());
    if !dns_ok {
        fail_run(
            &app,
            &state,
            &run_id,
            &cancel,
            &snapshot,
            "No network: DNS failed for all probe hosts",
        );
        return;
    }
    let mut best: Option<(String, u32)> = None;
    let mut tested = 0u32;

    for (index, (profile_id, name, strategy_type)) in profiles.into_iter().enumerate() {
        let t_strategy = Instant::now();
        if !alive(&state, &run_id, &cancel) {
            bail(&app, &state, &run_id, &cancel, &snapshot);
            return;
        }

        if matches!(mode, DiagnosticsMode::Thorough) {
            if let Some((_, best_score)) = &best {
                let remaining_exp_only = name.to_ascii_lowercase().contains("exp");
                if *best_score >= 92 && tested >= 3 && remaining_exp_only {
                    break;
                }
                if *best_score >= 96 && tested >= 5 {
                    break;
                }
            }
        }

        if !mutate_if_alive(&state, &run_id, &cancel, |store| {
            store.diagnostics.current_strategy = Some(name.clone());
            store.diagnostics.progress =
                (((index as u32) * 90) / total.max(1)).saturating_add(5);
        }) {
            bail(&app, &state, &run_id, &cancel, &snapshot);
            return;
        }
        let _ = app.emit("diagnostics:progress", ());

        let t_start = Instant::now();
        let start_res = state.engine.start_diag(Some(&name));
        log::info!(
            "timing diag.start('{name}'): {:.0}ms ok={}",
            t_start.elapsed().as_secs_f64() * 1000.0,
            start_res.is_ok()
        );

        if !alive(&state, &run_id, &cancel) {
            let _ = state.engine.stop();
            bail(&app, &state, &run_id, &cancel, &snapshot);
            return;
        }

        let host_results = if start_res.is_ok() {
            // Short settle so WinDivert filters attach before probes.
            thread::sleep(Duration::from_millis(150));
            let budget = timeout.min(Duration::from_millis(1100));
            let t_probe = Instant::now();
            let results = probe_resolved_parallel(&resolved, budget, Some(cancel.clone()));
            log::info!(
                "timing diag.probe('{name}'): {:.0}ms",
                t_probe.elapsed().as_secs_f64() * 1000.0
            );
            results
        } else {
            Vec::new()
        };

        // 250ms start poll can miss a late driver/AV crash — don't score as "weak services".
        let died_during_probes = start_res.is_ok() && !state.engine.winws_alive();

        let t_stop = Instant::now();
        let _ = state.engine.stop();
        log::info!(
            "timing diag.stop('{name}'): {:.0}ms",
            t_stop.elapsed().as_secs_f64() * 1000.0
        );

        if !alive(&state, &run_id, &cancel) {
            bail(&app, &state, &run_id, &cancel, &snapshot);
            return;
        }

        let startup_failed = start_res.is_err() || died_during_probes;
        let adaptive = if start_res.is_ok() && !died_during_probes {
            score_by_service(&host_results, PASS_THRESHOLD)
        } else {
            crate::engine::AdaptiveScore {
                score: 0,
                success: false,
                services: Vec::new(),
                all_probes: Vec::new(),
            }
        };
        let (latency_min, latency_median_ms, latency_max_ms) =
            latency_stats(&adaptive.all_probes);
        let soft_score = if startup_failed {
            0
        } else {
            soft_contribution(&host_results)
        };
        let score = if startup_failed {
            0
        } else {
            rank_score_full(adaptive.score, &name, latency_min, soft_score)
        };
        let success = !startup_failed && adaptive.success && score >= PASS_THRESHOLD;
        let latency_ms = latency_min;

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

        let failed_hosts: Vec<&str> = host_results
            .iter()
            .filter(|(t, probes)| t.critical && probes.iter().all(|p| !p.ok))
            .map(|(t, _)| t.host)
            .collect();
        let soft_fail_notes: Vec<String> = host_results
            .iter()
            .filter(|(t, probes)| !t.critical && probes.iter().all(|p| !p.ok))
            .filter_map(|(t, probes)| {
                probes.first().map(|p| format!("{} ({})", t.host, p.note))
            })
            .collect();

        let services: Vec<DiagnosticServiceScore> = adaptive
            .services
            .iter()
            .map(|s| {
                let latency_ms = host_results
                    .iter()
                    .filter(|(t, _)| t.service == s.service)
                    .flat_map(|(_, probes)| probes.iter().filter_map(|p| p.latency_ms))
                    .min();
                DiagnosticServiceScore {
                    service: s.service.clone(),
                    score: s.score,
                    ok: s.ok,
                    latency_ms,
                }
            })
            .collect();

        let hosts: Vec<DiagnosticHostDetail> = host_results
            .iter()
            .flat_map(|(target, probes)| {
                probes.iter().map(|p| DiagnosticHostDetail {
                    host: if target.port == 443 {
                        target.host.to_string()
                    } else {
                        format!("{}:{}", target.host, target.port)
                    },
                    service: target.service.to_string(),
                    ok: p.ok,
                    latency_ms: p.latency_ms,
                    note: p.note.clone(),
                    critical: target.critical,
                })
            })
            .collect();

        let notes = if let Err(e) = &start_res {
            format!("Start failed: {e}")
        } else if died_during_probes {
            "Startup failed: winws exited during probes (driver/AV?)".into()
        } else if success {
            let ok = adaptive.all_probes.iter().filter(|p| p.ok).count();
            let soft_warn = if soft_fail_notes.is_empty() {
                String::new()
            } else {
                format!(" · soft gaps: {}", soft_fail_notes.join(", "))
            };
            format!(
                "{ok}/{} probes ok · soft {soft_score} · rank {score} · {service_summary}{soft_warn}",
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
            let hosts = if failed_hosts.is_empty() {
                String::new()
            } else {
                format!(" · down: {}", failed_hosts.join(", "))
            };
            format!("Weak: {failed}{hosts} · {service_summary}")
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
            latency_median_ms,
            latency_max_ms,
            score,
            soft_score,
            services,
            hosts,
            notes,
            tested_at: now_iso(),
        };

        tested += 1;
        if !mutate_if_alive(&state, &run_id, &cancel, |store| {
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
        }) {
            bail(&app, &state, &run_id, &cancel, &snapshot);
            return;
        }
        let _ = app.emit("diagnostics:progress", ());

        log::info!(
            "timing diag.strategy('{name}'): {:.0}ms success={success} score={score}",
            t_strategy.elapsed().as_secs_f64() * 1000.0
        );

        if success {
            match &best {
                Some((_, best_score)) if score <= *best_score => {}
                _ => best = Some((name.clone(), score)),
            }
            // Fast mode: stop on a strong pass (not merely the first weak pass).
            if matches!(mode, DiagnosticsMode::Fast) {
                let strong = score >= 88 && soft_score >= 40;
                let solid = score >= 92;
                if strong || solid {
                    break;
                }
            }
        }
    }

    if !alive(&state, &run_id, &cancel) {
        bail(&app, &state, &run_id, &cancel, &snapshot);
        return;
    }

    if !mutate_if_alive(&state, &run_id, &cancel, |store| {
        store.diagnostics.phase = DiagnosticPhase::Analyzing;
        store.diagnostics.current_strategy = Some("Selecting best strategy…".into());
        store.diagnostics.progress = 96;
        store
            .diagnostics
            .results
            .sort_by(|a, b| {
                b.score
                    .cmp(&a.score)
                    .then(b.soft_score.cmp(&a.soft_score))
                    .then(a.latency_ms.cmp(&b.latency_ms))
            });
    }) {
        bail(&app, &state, &run_id, &cancel, &snapshot);
        return;
    }
    let _ = app.emit("diagnostics:progress", ());

    if !alive(&state, &run_id, &cancel) {
        bail(&app, &state, &run_id, &cancel, &snapshot);
        return;
    }

    let ranked: Vec<(String, u32)> = {
        let store = state.inner.lock().expect("state lock");
        store
            .diagnostics
            .results
            .iter()
            .filter(|r| r.success)
            .map(|r| (r.name.clone(), r.score))
            .collect()
    };

    if ranked.is_empty() {
        state.history.push(
            "diagnostics",
            "No strategy passed both YouTube and Discord; restoring previous state",
        );
        if !alive(&state, &run_id, &cancel) {
            bail(&app, &state, &run_id, &cancel, &snapshot);
            return;
        }
        restore_previous(&state, &snapshot);
        complete_ok(&app, &state, &run_id, &cancel, &snapshot);
        return;
    }

    let mut started = false;
    let mut last_err = String::new();
    for (name, score) in &ranked {
        if !alive(&state, &run_id, &cancel) {
            bail(&app, &state, &run_id, &cancel, &snapshot);
            return;
        }
        apply_best(&state, name);
        match state.engine.start_no_elevate(Some(name)) {
            Ok(_) => {
                state.history.push(
                    "diagnostics",
                    format!(
                        "Selected '{name}' (adaptive score {score}; YouTube+Discord required)"
                    ),
                );
                started = true;
                break;
            }
            Err(e) => {
                last_err = e.to_string();
                state.history.push(
                    "diagnostics",
                    format!("Could not start '{name}': {e}"),
                );
            }
        }
    }

    if !alive(&state, &run_id, &cancel) {
        bail(&app, &state, &run_id, &cancel, &snapshot);
        return;
    }

    if !started {
        fail_run(
            &app,
            &state,
            &run_id,
            &cancel,
            &snapshot,
            format!("Could not start a working strategy: {last_err}"),
        );
        return;
    }

    complete_ok(&app, &state, &run_id, &cancel, &snapshot);
}

fn complete_ok(
    app: &AppHandle,
    state: &AppState,
    run_id: &str,
    cancel: &Arc<Mutex<bool>>,
    snapshot: &Snapshot,
) {
    if !mutate_if_alive(state, run_id, cancel, |store| {
        store.diagnostics.phase = DiagnosticPhase::Completed;
        store.diagnostics.progress = 100;
        store.diagnostics.current_strategy = None;
        store.diagnostics.finished_at = Some(now_iso());
        store.diagnostics.is_demo = false;
    }) {
        bail(app, state, run_id, cancel, snapshot);
        return;
    }
    let _ = app.emit("diagnostics:completed", ());
}

fn fail_run(
    app: &AppHandle,
    state: &AppState,
    run_id: &str,
    cancel: &Arc<Mutex<bool>>,
    snapshot: &Snapshot,
    message: impl Into<String>,
) {
    let message = message.into();
    if superseded(state, run_id) {
        return;
    }
    if cancelled(cancel) {
        bail(app, state, run_id, cancel, snapshot);
        return;
    }
    restore_previous(state, snapshot);
    {
        let mut store = state.inner.lock().expect("state lock");
        if store.diagnostics.id != run_id {
            return;
        }
        store.diagnostics.phase = DiagnosticPhase::Failed;
        store.diagnostics.current_strategy = None;
        store.diagnostics.finished_at = Some(now_iso());
        store.diagnostics.is_demo = false;
    }
    state.history.push("diagnostics", message);
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

fn restore_previous(state: &AppState, snapshot: &Snapshot) {
    if let Some(name) = snapshot.previous.as_deref() {
        state.engine.set_active_strategy(name.to_string());
        let mut store = state.inner.lock().expect("state lock");
        for p in store.profiles.iter_mut() {
            p.is_active = p.name == name;
        }
    }
    if snapshot.was_running {
        if let Some(name) = snapshot.previous.as_deref() {
            let _ = state.engine.start_no_elevate(Some(name));
        } else {
            let _ = state.engine.start_no_elevate(None);
        }
    }
}
