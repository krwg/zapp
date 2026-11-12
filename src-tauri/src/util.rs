// ╔════════════════════════════════════════════════════╗
// ║                       util                       ║
// ╚════════════════════════════════════════════════════╝

use chrono::Utc;
use uuid::Uuid;

pub fn now_iso() -> String {
    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

pub fn new_id(prefix: &str) -> String {
    format!("{prefix}-{}", Uuid::new_v4())
}
