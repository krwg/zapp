// ╔════════════════════════════════════════════════════╗
// ║                  engine/elevate                  ║
// ╚════════════════════════════════════════════════════╝

use std::path::Path;
use std::process::{Command, Stdio};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::engine::faults::{classify_spawn_failure, EngineFault, FaultCode};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn is_elevated() -> bool {
    #[cfg(windows)]
    {
        Command::new("net")
            .arg("session")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        true
    }
}

fn ps_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

pub fn start_winws_elevated(
    winws: &Path,
    work_dir: &Path,
    args: &[String],
) -> Result<(), EngineFault> {
    #[cfg(windows)]
    {
        let winws_s = winws.to_string_lossy();
        let dir_s = work_dir.to_string_lossy();

        let arg_list = if args.is_empty() {
            "$null".to_string()
        } else {
            let parts: Vec<String> = args.iter().map(|a| ps_quote(a)).collect();
            format!("@({})", parts.join(","))
        };

        
        let script = format!(
            "$ErrorActionPreference='Stop'; try {{ Start-Process -FilePath {file} -WorkingDirectory {dir} -ArgumentList {args} -Verb RunAs -WindowStyle Hidden; exit 0 }} catch {{ if ($_.Exception.Message -match 'canceled|cancelled|1223') {{ exit 1223 }}; Write-Error $_; exit 1 }}",
            file = ps_quote(&winws_s),
            dir = ps_quote(&dir_s),
            args = arg_list,
        );

        let output = Command::new("powershell")
            .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| EngineFault::new(FaultCode::UacLaunchFailed, e.to_string()))?;

        let code = output.status.code().unwrap_or(1);
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let combined = format!("{stderr} {stdout}").trim().to_string();

        if code == 0 {
            return Ok(());
        }
        if code == 1223 {
            return Err(EngineFault::new(
                FaultCode::UacCancelled,
                "User declined the UAC prompt",
            ));
        }

        let classified = classify_spawn_failure(&combined);
        Err(EngineFault::new(
            if classified == FaultCode::Unknown {
                FaultCode::UacLaunchFailed
            } else {
                classified
            },
            if combined.is_empty() {
                format!("Elevated Start-Process failed (exit {code})")
            } else {
                combined
            },
        ))
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
