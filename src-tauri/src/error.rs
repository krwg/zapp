// ╔════════════════════════════════════════════════════╗
// ║                      error                       ║
// ╚════════════════════════════════════════════════════╝

use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Plain user-facing message (also used as log text).
    #[error("{0}")]
    Message(String),
    /// Toast/UI see `user`; history / logs / bug report get `log`.
    #[error("{user}")]
    Detailed { user: String, log: String },
}

impl AppError {
    pub fn user_message(&self) -> &str {
        match self {
            Self::Message(m) => m,
            Self::Detailed { user, .. } => user,
        }
    }

    pub fn log_message(&self) -> &str {
        match self {
            Self::Message(m) => m,
            Self::Detailed { log, .. } => log,
        }
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.user_message())
    }
}

impl From<&str> for AppError {
    fn from(value: &str) -> Self {
        Self::Message(value.to_string())
    }
}

impl From<String> for AppError {
    fn from(value: String) -> Self {
        Self::Message(value)
    }
}

pub type AppResult<T> = Result<T, AppError>;
