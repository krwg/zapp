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

    pub fn message(&self) -> String {
        format!("{} — {}. {}", self.title, self.detail, self.hint)
    }
}

impl From<EngineFault> for AppError {
    fn from(value: EngineFault) -> Self {
        AppError::from(value.message())
    }
}

fn catalog(code: &FaultCode) -> (&'static str, &'static str, bool) {
    match code {
        FaultCode::WinwsBinaryMissing => (
            "winws не найден",
            "Проверьте Engine pack / обновление Engine в разделе Updates.",
            false,
        ),
        FaultCode::WinDivertDriverMissing => (
            "Нет драйвера WinDivert",
            "В пакете должны быть WinDivert64.sys и WinDivert.dll рядом с winws.exe.",
            false,
        ),
        FaultCode::StrategyBatMissing => (
            "Стратегия не найдена",
            "Выберите другой профиль или обновите Strategies.",
            true,
        ),
        FaultCode::StrategyParseFailed => (
            "Не разобрать стратегию",
            "BAT повреждён или имеет нестандартный формат. Попробуйте другую стратегию.",
            true,
        ),
        FaultCode::StrategyPayloadMissing => (
            "Не хватает файлов стратегии",
            "Отсутствуют списки или payload (lists/*.txt, bin/*.bin), на которые ссылается профиль. Обновите Engine или переустановите пакет Flowseal.",
            false,
        ),
        FaultCode::UacCancelled => (
            "UAC отклонён",
            "Нажмите Start ещё раз и подтвердите запрос прав администратора.",
            true,
        ),
        FaultCode::UacLaunchFailed => (
            "Elevated-запуск не удался",
            "Подтвердите UAC. Если окно не появилось — антивирус мог заблокировать winws.",
            true,
        ),
        FaultCode::WinwsExitedImmediately => (
            "winws сразу завершился",
            "Частые причины: конфликт WinDivert, AV, битый драйвер, неверные аргументы стратегии.",
            true,
        ),
        FaultCode::WinDivertConflict => (
            "Конфликт WinDivert",
            "Остановите другие обходы (GoodbyeDPI, zapret, ZapretGUI) и нажмите Stop, затем Start.",
            true,
        ),
        FaultCode::AccessDenied => (
            "Нет прав",
            "Для WinDivert нужен elevated-запуск winws. Подтвердите UAC.",
            true,
        ),
        FaultCode::ConflictingBypass => (
            "Уже запущен другой обход",
            "zapp остановит чужой winws при Stop. Закройте параллельные GUI-обходы вручную.",
            true,
        ),
        FaultCode::LikelyBlockedByAv => (
            "Похоже на блок антивируса",
            "Добавьте папку Engine (winws.exe, WinDivert*) в исключения Defender/AV.",
            true,
        ),
        FaultCode::StopFailed => (
            "Не удалось остановить",
            "Попробуйте Stop ещё раз. При необходимости завершите winws.exe в диспетчере задач.",
            true,
        ),
        FaultCode::EngineNotInstalled => (
            "Engine не установлен",
            "Откройте Updates → Engine и установите пакет Flowseal.",
            false,
        ),
        FaultCode::DiagnosticsNeedsAdmin => (
            "Диагностике нужны права",
            "Один раз запустите Start (UAC), затем повторите Diagnostics.",
            true,
        ),
        FaultCode::Unknown => (
            "Ошибка движка",
            "Смотрите детали ниже. Повторите Start или запустите Diagnostics.",
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
    } else {
        FaultCode::Unknown
    }
}
