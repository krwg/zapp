// ╔════════════════════════════════════════════════════╗
// ║                  engine/faults                   ║
// ╚════════════════════════════════════════════════════╝

use std::sync::atomic::{AtomicU8, Ordering};

use serde::{Deserialize, Serialize};

use crate::error::AppError;

const LOCALE_EN: u8 = 0;
const LOCALE_RU: u8 = 1;

static UI_LOCALE: AtomicU8 = AtomicU8::new(LOCALE_EN);


pub fn set_ui_locale(locale: &str) {
    let v = if locale.eq_ignore_ascii_case("ru") {
        LOCALE_RU
    } else {
        LOCALE_EN
    };
    UI_LOCALE.store(v, Ordering::Relaxed);
}

fn ui_is_ru() -> bool {
    UI_LOCALE.load(Ordering::Relaxed) == LOCALE_RU
}

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


    pub fn message(&self) -> String {
        format!("{} — {}", self.title, self.hint)
    }


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
    if ui_is_ru() {
        catalog_ru(code)
    } else {
        catalog_en(code)
    }
}

fn catalog_en(code: &FaultCode) -> (&'static str, &'static str, bool) {
    match code {
        FaultCode::WinwsBinaryMissing => (
            "Bypass binary missing",
            "Open Updates → Engine and reinstall the pack.",
            false,
        ),
        FaultCode::WinDivertDriverMissing => (
            "Driver files missing",
            "Reinstall Engine from Updates.",
            false,
        ),
        FaultCode::StrategyBatMissing => (
            "Strategy not found",
            "Pick another mode or update Strategies.",
            true,
        ),
        FaultCode::StrategyParseFailed => (
            "Could not read strategy",
            "Try another mode or update Strategies.",
            true,
        ),
        FaultCode::StrategyPayloadMissing => (
            "Strategy files missing",
            "Update Engine or Strategies from Updates.",
            false,
        ),
        FaultCode::UacCancelled => (
            "Administrator prompt declined",
            "Turn bypass on again and accept the Windows admin prompt.",
            true,
        ),
        FaultCode::UacLaunchFailed => (
            "Could not get administrator rights",
            "Accept the admin prompt. If no window appears, check antivirus or restart zapp.",
            true,
        ),
        FaultCode::WinwsExitedImmediately => (
            "Bypass did not start",
            "Close other VPN/bypass tools and try again. If it repeats, report the bug.",
            true,
        ),
        FaultCode::WinDivertConflict => (
            "Conflict with another bypass",
            "Stop other bypass tools, reset the engine, then turn bypass on again.",
            true,
        ),
        FaultCode::AccessDenied => (
            "Access denied",
            "Accept the administrator prompt when turning bypass on.",
            true,
        ),
        FaultCode::ConflictingBypass => (
            "Another bypass is already running",
            "Close parallel bypass tools and try again.",
            true,
        ),
        FaultCode::LikelyBlockedByAv => (
            "Antivirus may be blocking",
            "Add the Engine folder to antivirus exclusions and try again.",
            true,
        ),
        FaultCode::StopFailed => (
            "Could not stop",
            "Turn bypass off again or reset the engine.",
            true,
        ),
        FaultCode::EngineNotInstalled => (
            "Engine not installed",
            "Open Updates → Engine and install the pack.",
            false,
        ),
        FaultCode::DiagnosticsNeedsAdmin => (
            "Administrator rights required for the check",
            "Turn bypass on once (accept the admin prompt), then run the check again.",
            true,
        ),
        FaultCode::LaunchRefused => (
            "Could not start bypass",
            "Check Engine in Updates and try again. If it repeats, report the bug.",
            true,
        ),
        FaultCode::Unknown => (
            "Action failed",
            "Try again. If it keeps happening, use Report a bug.",
            true,
        ),
    }
}

fn catalog_ru(code: &FaultCode) -> (&'static str, &'static str, bool) {
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
