// ╔════════════════════════════════════════════════════╗
// ║                  engine/elevate                  ║
// ╚════════════════════════════════════════════════════╝

use std::path::Path;

use crate::engine::faults::{EngineFault, FaultCode};

pub fn is_elevated() -> bool {
    #[cfg(windows)]
    {
        crate::win32::is_process_elevated()
    }
    #[cfg(not(windows))]
    {
        true
    }
}

/// Quote a single Windows command-line argument (CommandLineToArgvW-compatible-ish).
fn quote_arg(arg: &str) -> String {
    if arg.is_empty() {
        return "\"\"".into();
    }
    let needs_quotes = arg.bytes().any(|b| b == b' ' || b == b'\t' || b == b'"');
    if !needs_quotes {
        return arg.to_string();
    }
    let mut out = String::from('"');
    let mut backslashes = 0u32;
    for ch in arg.chars() {
        if ch == '\\' {
            backslashes += 1;
            continue;
        }
        if ch == '"' {
            for _ in 0..(backslashes * 2 + 1) {
                out.push('\\');
            }
            out.push('"');
            backslashes = 0;
            continue;
        }
        for _ in 0..backslashes {
            out.push('\\');
        }
        backslashes = 0;
        out.push(ch);
    }
    for _ in 0..(backslashes * 2) {
        out.push('\\');
    }
    out.push('"');
    out
}

fn join_args(args: &[String]) -> String {
    args.iter().map(|a| quote_arg(a)).collect::<Vec<_>>().join(" ")
}

/// Direct elevated winws launch (ShellExecuteEx). Normal Start goes through the helper pipe;
/// kept for diagnostics / fallback callers.
#[allow(dead_code)]
pub fn start_winws_elevated(
    winws: &Path,
    work_dir: &Path,
    args: &[String],
) -> Result<(), EngineFault> {
    #[cfg(windows)]
    {
        let params = join_args(args);
        match crate::win32::shell_execute_runas(winws, &params, work_dir) {
            Ok(()) => Ok(()),
            Err(crate::win32::ElevateError::Cancelled) => Err(EngineFault::new(
                FaultCode::UacCancelled,
                "User declined the UAC prompt",
            )),
            Err(crate::win32::ElevateError::Failed(msg)) => {
                Err(EngineFault::new(FaultCode::UacLaunchFailed, msg))
            }
        }
    }

    #[cfg(not(windows))]
    {
        let _ = (winws, work_dir, args);
        Err(EngineFault::new(
            FaultCode::Unknown,
            "Elevation is only implemented on Windows",
        ))
    }
}
