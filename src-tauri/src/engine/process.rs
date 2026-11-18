// ╔════════════════════════════════════════════════════╗
// ║                  engine/process                  ║
// ╚════════════════════════════════════════════════════╝
//! Fast process lookup via ToolHelp (~1ms). Avoid tasklist/taskkill on hot paths.

use std::time::Instant;

/// True if an image with this name is running (case-insensitive).
pub fn process_running(image: &str) -> bool {
    #[cfg(windows)]
    {
        process_running_toolhelp(image)
    }
    #[cfg(not(windows))]
    {
        let _ = image;
        false
    }
}

/// True if a specific PID is still alive.
pub fn process_pid_running(pid: u32) -> bool {
    #[cfg(windows)]
    {
        process_pid_running_win(pid)
    }
    #[cfg(not(windows))]
    {
        let _ = pid;
        false
    }
}

/// Force-terminate every process matching `image`. Prefer `Child::kill` on owned processes.
pub fn kill_image(image: &str) {
    let t0 = Instant::now();
    #[cfg(windows)]
    {
        kill_image_toolhelp(image);
    }
    #[cfg(not(windows))]
    {
        let _ = image;
    }
    log::info!(
        "timing kill_image({image}): {:.0}ms",
        t0.elapsed().as_secs_f64() * 1000.0
    );
}

pub fn kill_orphan_winws() {
    kill_image("winws.exe");
}

#[cfg(windows)]
fn process_pid_running_win(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, WAIT_TIMEOUT};
    use windows_sys::Win32::System::Threading::{
        OpenProcess, WaitForSingleObject, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
    };

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE, 0, pid);
        if handle.is_null() {
            return false;
        }
        let alive = WaitForSingleObject(handle, 0) == WAIT_TIMEOUT;
        CloseHandle(handle);
        alive
    }
}

#[cfg(windows)]
fn process_running_toolhelp(image: &str) -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE {
            return false;
        }
        let mut e: PROCESSENTRY32W = std::mem::zeroed();
        e.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut found = false;
        let mut ok = Process32FirstW(snap, &mut e);
        while ok != 0 {
            let n = e
                .szExeFile
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(e.szExeFile.len());
            if String::from_utf16_lossy(&e.szExeFile[..n]).eq_ignore_ascii_case(image) {
                found = true;
                break;
            }
            ok = Process32NextW(snap, &mut e);
        }
        CloseHandle(snap);
        found
    }
}

#[cfg(windows)]
fn kill_image_toolhelp(image: &str) {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE {
            return;
        }
        let mut e: PROCESSENTRY32W = std::mem::zeroed();
        e.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut pids = Vec::new();
        let mut ok = Process32FirstW(snap, &mut e);
        while ok != 0 {
            let n = e
                .szExeFile
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(e.szExeFile.len());
            if String::from_utf16_lossy(&e.szExeFile[..n]).eq_ignore_ascii_case(image) {
                pids.push(e.th32ProcessID);
            }
            ok = Process32NextW(snap, &mut e);
        }
        CloseHandle(snap);

        for pid in pids {
            let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
            if !handle.is_null() && handle != INVALID_HANDLE_VALUE {
                let _ = TerminateProcess(handle, 1);
                CloseHandle(handle);
            }
        }
    }
}
