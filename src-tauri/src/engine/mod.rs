// ╔════════════════════════════════════════════════════╗
// ║                    engine/mod                    ║
// ╚════════════════════════════════════════════════════╝

mod discover;
mod elevate;
mod faults;
mod github;
mod integrity;
mod managed;
mod pack;
mod preflight;
mod process;
mod probe;
mod strategy;
mod updates;

pub use discover::{resolve_engine_layout, scan_strategy_profiles};
pub use elevate::is_elevated;
pub use faults::{fault_from_helper_message, set_ui_locale, EngineFault, FaultCode};
pub use managed::{EngineLayout, ManagedEngine};
pub use pack::{ensure_flowseal_sidecars, pack_payloads_present};
pub use preflight::{
    list_conflicting_images, resolve_conflicting_bypass, run_preflight, run_preflight_scoped,
    ConflictSnapshot, PreflightReport, PreflightScope,
};
pub use process::{kill_orphan_winws, process_running};
pub use probe::{
    diag_targets, invalidate_dns_cache, latency_stats, path_verdict_from_results, probe_matrix_host,
    probe_resolved_parallel, probe_targets_parallel, probe_tcp, rank_score,
    rank_score_with_latency, resolve_diag_targets, rank_score_full, score_by_service, score_probes,
    soft_contribution, strategy_prior_bonus, watchdog_targets, AdaptiveScore, ProbeResult,
};

pub use strategy::parse_strategy_bat;
pub use updates::{UpdateManager, UpdateOutcome};
