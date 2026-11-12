// ╔════════════════════════════════════════════════════╗
// ║                 helper/protocol                  ║
// ╚════════════════════════════════════════════════════╝

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum HelperRequest {
    Ping { token: String },
    Status { token: String },
    Start {
        token: String,
        winws: String,
        #[serde(alias = "workDir")]
        work_dir: String,
        args: Vec<String>,
        strategy: String,
    },
    Stop { token: String },
    Reset { token: String },
    Shutdown { token: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HelperStatus {
    pub running: bool,
    pub strategy: Option<String>,
    pub pid: Option<u32>,
    pub elevated: bool,
    #[serde(default)]
    pub exe_path: Option<String>,
    #[serde(default)]
    pub exe_mtime: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HelperResponse {
    pub ok: bool,
    pub message: String,
    pub status: Option<HelperStatus>,
}
