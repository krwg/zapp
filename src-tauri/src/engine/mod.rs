// ╔════════════════════════════════════════════════════╗
// ║                    engine/mod                    ║
// ╚════════════════════════════════════════════════════╝

mod discover;
mod elevate;
mod faults;
mod github;
mod managed;
mod preflight;
mod probe;
mod strategy;
mod updates;

pub use discover::{resolve_engine_layout, scan_strategy_profiles};
pub use elevate::is_elevated;
pub use faults::{EngineFault, FaultCode};
pub use managed::{EngineLayout, ManagedEngine};
pub use preflight::{run_preflight, PreflightReport};
pub use probe::{probe_matrix_host, probe_tcp, score_probes};
pub use strategy::parse_strategy_bat;
pub use updates::UpdateManager;
