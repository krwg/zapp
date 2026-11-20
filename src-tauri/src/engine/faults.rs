// ╔════════════════════════════════════════════════════╗
// ║                  engine/faults                   ║
// ╚════════════════════════════════════════════════════╝

use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum FaultCode {
    WinwsBinaryMissing,
    WinDivertDriverMissing,
    StrategyBatMissing,
    StrategyParseFailed,
    StrategyPayloadMissing,
    UacCancelled,
    UacLaunchFailed,
    WinwsExitedImmediately,
    WinDivertConflict,
    AccessDenied,
    ConflictingBypass,
    LikelyBlockedByAv,
    StopFailed,
    EngineNotInstalled,
    DiagnosticsNeedsAdmin,
    /// Helper refused the launch (path / policy). Detail is for logs only.
    LaunchRefused,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineFault {
    pub code: FaultCode,
    pub title: String,
    pub detail: String,
    pub hint: String,
    pub retryable: bool,
}

impl EngineFault {
    pub fn new(code: FaultCode, detail: impl Into<String>) -> Self {
        let (title, hint, retryable) = catalog(&code);
        Self {
            code,
            title: title.into(),
            detail: detail.into(),
            hint: hint.into(),
            retryable,
        }
    }

    /// Short user-facing text for toasts / UI (no raw paths or internal refusal strings).
    pub fn message(&self) -> String {
        format!("{} — {}", self.title, self.hint)
    }

    /// Full line for logs / history (includes technical detail).
    pub fn log_line(&self) -> String {
        format!(
            "[{:?}] {} | detail={} | hint={}",
            self.code, self.title, self.detail, self.hint
        )
    }
}

impl From<EngineFault> for AppError {
    fn from(value: EngineFault) -> Self {
        AppError::Detailed {
            user: value.message(),
            log: value.log_line(),
        }
    }
}

fn catalog(code: &FaultCode) -> (&'static str, &'static str, bool) {
    match code {
        FaultCode::WinwsBinaryMissing => (
            "Не найден файл обхода",
            "Откройте Updates → Engine и переустановите пакет.",
            false,
        ),
        FaultCode::WinDivertDriverMissing => (
            "Не хватает файлов драйвера",
            "Переустановите Engine в разделе Updates.",
            false,
        ),
        FaultCode::StrategyBatMissing => (
            "Стратегия не найдена",
            "Выберите другой режим или обновите Strategies.",
            true,
        ),
        FaultCode::StrategyParseFailed => (
            "Не удалось прочитать стратегию",
            "Попробуйте другой режим или обновите Strategies.",
            true,
        ),
        FaultCode::StrategyPayloadMissing => (
            "Не хватает файлов стратегии",
            "Обновите Engine или Strategies в разделе Updates.",
            false,
        ),
        FaultCode::UacCancelled => (
            "Запрос прав отклонён",
            "Нажмите включение ещё раз и подтвердите запрос администратора.",
            true,
        ),
        FaultCode::UacLaunchFailed => (
            "Не удалось получить права",
            "Подтвердите запрос администратора. Если окно не появилось — проверьте антивирус.",
            true,
        ),
        FaultCode::WinwsExitedImmediately => (
            "Обход не запустился",
            "Закройте другие VPN/обходы и попробуйте снова. Если повторится — сообщите об ошибке.",
            true,
        ),
        FaultCode::WinDivertConflict => (
            "Конфликт с другим обходом",
            "Остановите другие программы обхода, нажмите «Сбросить ядро», затем включите снова.",
            true,
        ),
        FaultCode::AccessDenied => (
            "Не хватает прав",
            "Подтвердите запрос администратора при включении обхода.",
            true,
        ),
        FaultCode::ConflictingBypass => (
            "Уже запущен другой обход",
            "Закройте параллельные программы обхода и попробуйте снова.",
            true,
        ),
        FaultCode::LikelyBlockedByAv => (
            "Похоже, мешает антивирус",
            "Добавьте папку Engine в исключения антивируса и попробуйте снова.",
            true,
        ),
        FaultCode::StopFailed => (
            "Не удалось остановить",
            "Нажмите выключение ещё раз или сбросьте ядро.",
            true,
        ),
        FaultCode::EngineNotInstalled => (
            "Engine не установлен",
            "Откройте Updates → Engine и установите пакет.",
            false,
        ),
        FaultCode::DiagnosticsNeedsAdmin => (
            "Для проверки нужны права",
            "Один раз включите обход (с подтверждением прав), затем повторите проверку.",
            true,
        ),
        FaultCode::LaunchRefused => (
            "Не удалось запустить обход",
            "Проверьте Engine в Updates и включите снова. Если повторится — сообщите об ошибке.",
            true,
        ),
        FaultCode::Unknown => (
            "Не удалось выполнить действие",
            "Попробуйте ещё раз. Если ошибка повторяется — нажмите «Сообщить об ошибке».",
            true,
        ),
    }
}

pub fn classify_spawn_failure(raw: &str) -> FaultCode {
    let lower = raw.to_ascii_lowercase();
    if lower.contains("canceled") || lower.contains("cancelled") || lower.contains("1223") {
        FaultCode::UacCancelled
    } else if lower.contains("access is denied") || lower.contains("отказ в доступе") {
        FaultCode::AccessDenied
    } else if lower.contains("antivirus") || lower.contains("virus") || lower.contains("defender") {
        FaultCode::LikelyBlockedByAv
    } else if lower.contains("uac") {
        FaultCode::UacLaunchFailed
    } else if lower.contains("refused:") {
        FaultCode::LaunchRefused
    } else {
        FaultCode::Unknown
    }
}

pub fn fault_from_helper_message(message: String) -> EngineFault {
    let code = if message.to_ascii_lowercase().contains("refused:") {
        FaultCode::LaunchRefused
    } else {
        FaultCode::WinwsExitedImmediately
    };
    EngineFault::new(code, message)
}
