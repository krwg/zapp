// ╔════════════════════════════════════════════════════╗
// ║                    helper/mod                    ║
// ╚════════════════════════════════════════════════════╝

mod client;
pub mod paths;
mod protocol;
mod server;

pub use client::{
    ensure_helper, helper_install_engine, helper_ping, helper_reset, helper_shutdown,
    helper_start, helper_status, helper_stop,
};
pub use protocol::{HelperRequest, HelperResponse, HelperStatus};

use std::path::PathBuf;

use interprocess::local_socket::{GenericNamespaced, Name, ToNsName};


#[allow(dead_code)]
pub const HELPER_PORT: u16 = 39221;

pub const HELPER_PIPE: &str = "zapp-helper";
pub const HELPER_FLAG: &str = "--helper";

pub fn helper_pipe_name() -> Result<Name<'static>, String> {
    HELPER_PIPE
        .to_ns_name::<GenericNamespaced>()
        .map_err(|e| format!("helper pipe name: {e}"))
}

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
