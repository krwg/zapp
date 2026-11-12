// ╔════════════════════════════════════════════════════╗
// ║                     history                      ║
// ╚════════════════════════════════════════════════════╝

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::util::now_iso;

const MAX_EVENTS: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEvent {
    pub id: String,
    pub at: String,
    pub kind: String,
    pub message: String,
}

pub struct HistoryLog {
    path: PathBuf,
    events: Mutex<Vec<HistoryEvent>>,
}

impl HistoryLog {
    pub fn open(app_data: PathBuf) -> Self {
        let path = app_data.join("history.json");
        let events = fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<Vec<HistoryEvent>>(&s).ok())
            .unwrap_or_default();
        Self {
            path,
            events: Mutex::new(events),
        }
    }

    pub fn push(&self, kind: &str, message: impl Into<String>) {
        let event = HistoryEvent {
            id: crate::util::new_id("evt"),
            at: now_iso(),
            kind: kind.into(),
            message: message.into(),
        };
        let mut events = self.events.lock().expect("history lock");
        events.insert(0, event);
        events.truncate(MAX_EVENTS);
        let _ = fs::write(&self.path, serde_json::to_string_pretty(&*events).unwrap_or_default());
    }

    #[allow(dead_code)]
    pub fn list(&self, limit: usize) -> Vec<HistoryEvent> {
        self.events
            .lock()
            .expect("history lock")
            .iter()
            .take(limit)
            .cloned()
            .collect()
    }
}
