// ╔════════════════════════════════════════════════════╗
// ║                    helper/mod                    ║
// ╚════════════════════════════════════════════════════╝

mod client;
mod protocol;
mod server;

pub use client::{
    ensure_helper, helper_ping, helper_reset, helper_shutdown, helper_start, helper_status,
    helper_stop,
};
pub use protocol::{HelperRequest, HelperResponse, HelperStatus};

use std::path::PathBuf;

pub const HELPER_PORT: u16 = 39221;
pub const HELPER_FLAG: &str = "--helper";

pub fn helper_state_dir() -> PathBuf {
    let base = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    base.join("zapp")
}

pub fn token_path() -> PathBuf {
    helper_state_dir().join("helper.token")
}

pub fn helper_exe_path() -> PathBuf {
    helper_state_dir().join("zapp-helper.exe")
}

pub fn run_helper() {
    if let Err(e) = server::serve() {
        eprintln!("zapp helper failed: {e}");
        std::process::exit(1);
    }
}
