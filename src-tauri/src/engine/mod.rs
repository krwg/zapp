// ╔════════════════════════════════════════════════════╗
// ║                    engine/mod                    ║
// ╚════════════════════════════════════════════════════╝

mod discover;
mod elevate;
mod faults;
mod github;
mod managed;
mod pack;
mod preflight;
mod probe;
mod strategy;
mod updates;

pub use discover::{resolve_engine_layout, scan_strategy_profiles};
pub use elevate::is_elevated;
pub use faults::{EngineFault, FaultCode};
pub use managed::{EngineLayout, ManagedEngine};
pub use pack::{ensure_flowseal_sidecars, pack_payloads_present};
pub use preflight::{run_preflight, PreflightReport};
pub use probe::{
    diag_targets, probe_matrix_host, probe_targets_parallel, probe_tcp, rank_score,
    score_by_service, score_probes, strategy_prior_bonus, AdaptiveScore, ProbeResult,
};
pub use strategy::parse_strategy_bat;
pub use updates::{UpdateManager, UpdateOutcome};
