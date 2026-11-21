// ╔════════════════════════════════════════════════════╗
// ║                      win32                       ║
// ╚════════════════════════════════════════════════════╝

//! Thin Windows helpers — prefer these over PowerShell for elevation / process start.

#![cfg(windows)]

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows_sys::Win32::Foundation::{CloseHandle, ERROR_CANCELLED, GetLastError, FALSE, HANDLE};
use windows_sys::Win32::Security::{
    GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows_sys::Win32::UI::Shell::{
    ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::SW_HIDE;

/// Attach a console for log output.
/// - always in debug builds
/// - or when `ZAPP_CONSOLE=1` / `true` / `yes` (release troubleshooting)
pub fn maybe_attach_console() {
    let want = cfg!(debug_assertions)
        || std::env::var("ZAPP_CONSOLE")
            .map(|v| {
                matches!(
                    v.trim().to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                )
            })
            .unwrap_or(false);
    if !want {
        return;
    }
    unsafe {
        use windows_sys::Win32::System::Console::{AllocConsole, ATTACH_PARENT_PROCESS, AttachConsole};
        // Prefer parent console (cargo/tauri:dev); otherwise allocate a new window.
        if AttachConsole(ATTACH_PARENT_PROCESS) == FALSE {
            let _ = AllocConsole();
        }
    }
}

/// True when the current process token is elevated (admin).
pub fn is_process_elevated() -> bool {
    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == FALSE {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION {
            TokenIsElevated: 0,
        };
        let mut ret_len = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            (&raw mut elevation).cast(),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut ret_len,
        );
        let _ = CloseHandle(token);
        ok != FALSE && elevation.TokenIsElevated != 0
    }
}

fn wide_null(s: impl AsRef<OsStr>) -> Vec<u16> {
    s.as_ref()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[derive(Debug)]
pub enum ElevateError {
    Cancelled,
    Failed(String),
}

/// Launch `exe` elevated via ShellExecuteEx `runas` (UAC). Does not wait for exit.
pub fn shell_execute_runas(
    exe: &Path,
    args: &str,
    work_dir: &Path,
) -> Result<(), ElevateError> {
    let file = wide_null(exe.as_os_str());
    let params = wide_null(OsStr::new(args));
    let dir = wide_null(work_dir.as_os_str());
    let verb = wide_null(OsStr::new("runas"));

    unsafe {
        let mut info: SHELLEXECUTEINFOW = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
        info.fMask = SEE_MASK_NOCLOSEPROCESS;
        info.lpVerb = verb.as_ptr();
        info.lpFile = file.as_ptr();
        info.lpParameters = params.as_ptr();
        info.lpDirectory = dir.as_ptr();
        info.nShow = SW_HIDE;

        let ok = ShellExecuteExW(&mut info);
        if ok == FALSE {
            let err = GetLastError();
            if err == ERROR_CANCELLED {
                return Err(ElevateError::Cancelled);
            }
            return Err(ElevateError::Failed(format!(
                "ShellExecuteEx runas failed (GetLastError={err})"
            )));
        }
        if !info.hProcess.is_null() {
            let _ = CloseHandle(info.hProcess);
        }
    }
    Ok(())
}
