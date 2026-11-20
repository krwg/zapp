// ╔════════════════════════════════════════════════════╗
// ║                  helper/client                   ║
// ╚════════════════════════════════════════════════════╝

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use uuid::Uuid;

use super::protocol::{HelperRequest, HelperResponse, HelperStatus};
use super::{helper_exe_path, helper_state_dir, token_path, HELPER_FLAG, HELPER_PORT};
use crate::engine::{EngineFault, FaultCode};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn ensure_token(_app_data: &Path) -> Result<String, EngineFault> {
    let dir = helper_state_dir();
    let path = token_path();
    if let Ok(existing) = fs::read_to_string(&path) {
        let t = existing.trim().to_string();
        if !t.is_empty() {
            return Ok(t);
        }
    }
    let token = Uuid::new_v4().to_string();
    fs::create_dir_all(&dir).map_err(|e| {
        EngineFault::new(FaultCode::Unknown, format!("cannot create helper state: {e}"))
    })?;
    fs::write(&path, &token)
        .map_err(|e| EngineFault::new(FaultCode::Unknown, format!("cannot write token: {e}")))?;
    harden_token_acl(&path);
    Ok(token)
}

fn harden_token_acl(path: &Path) {
    #[cfg(windows)]
    {
        // Restrict helper.token to the current user (+ SYSTEM). Not a full security
        // boundary for same-user processes, but blocks casual world-readable tokens.
        let path_s = path.to_string_lossy().replace('\'', "''");
        let script = format!(
            "$p = '{path_s}'; \
             icacls $p /inheritance:r | Out-Null; \
             icacls $p /grant:r \"$env:USERNAME:(R)\" | Out-Null; \
             icacls $p /grant:r \"SYSTEM:(F)\" | Out-Null"
        );
        let _ = Command::new("powershell")
            .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .status();
    }
    #[cfg(not(windows))]
    {
        let _ = path;
    }
}

fn request_with_timeout(
    req: &HelperRequest,
    read_timeout: Duration,
) -> Result<HelperResponse, EngineFault> {
    let mut stream = TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], HELPER_PORT)),
        Duration::from_secs(3),
    )
    .map_err(|e| {
        EngineFault::new(
            FaultCode::UacLaunchFailed,
            format!("helper not reachable: {e}"),
        )
    })?;
    stream.set_read_timeout(Some(read_timeout)).ok();
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .ok();

    let payload = serde_json::to_string(req)
        .map_err(|e| EngineFault::new(FaultCode::Unknown, e.to_string()))?;
    writeln!(stream, "{payload}")
        .map_err(|e| EngineFault::new(FaultCode::Unknown, format!("helper write: {e}")))?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|e| EngineFault::new(FaultCode::Unknown, format!("helper read: {e}")))?;

    serde_json::from_str(line.trim()).map_err(|e| {
        EngineFault::new(
            FaultCode::Unknown,
            format!("helper bad response: {e} ({})", line.trim()),
        )
    })
}

fn request(req: &HelperRequest) -> Result<HelperResponse, EngineFault> {
    request_with_timeout(req, Duration::from_secs(20))
}

fn helper_auth_ok(token: &str) -> Result<bool, EngineFault> {
    let req = HelperRequest::Ping {
        token: token.to_string(),
    };
    match request_with_timeout(&req, Duration::from_secs(3)) {
        Ok(r) if r.ok => Ok(true),
        Ok(r) if r.message.contains("unauthorized") => Ok(false),
        Ok(_) => Ok(false),
        Err(e) if e.message().contains("helper not reachable") => Ok(false),
        Err(e) => Err(e),
    }
}

fn kill_stale_helper() {
    #[cfg(windows)]
    {
        // Only kill zapp-helper / our staged binary — never an unrelated owner of :39221.
        let script = format!(
            "$conns = Get-NetTCPConnection -LocalPort {HELPER_PORT} -ErrorAction SilentlyContinue; \
             foreach ($c in $conns) {{ \
               $procId = $c.OwningProcess; \
               if (-not $procId -or $procId -eq 0) {{ continue }} \
               $p = Get-Process -Id $procId -ErrorAction SilentlyContinue; \
               if ($null -eq $p) {{ continue }} \
               $name = $p.ProcessName; \
               if ($name -eq 'zapp-helper' -or $name -eq 'zapp') {{ \
                 Stop-Process -Id $procId -Force -ErrorAction SilentlyContinue \
               }} \
             }}"
        );
        let _ = Command::new("powershell")
            .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .status();
        std::thread::sleep(Duration::from_millis(250));
    }
}

fn local_exe_mtime_secs() -> Option<u64> {
    let path = std::env::current_exe().ok()?;
    let meta = fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?;
    Some(
        modified
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_secs(),
    )
}

fn helper_is_current(token: &str) -> bool {
    let Ok(res) = request_with_timeout(
        &HelperRequest::Ping {
            token: token.to_string(),
        },
        Duration::from_secs(3),
    ) else {
        return false;
    };
    if !res.ok {
        return false;
    }
    let Some(status) = res.status else {
        return false;
    };
    let helper_bin = helper_exe_path();
    let Some(have_path) = status.exe_path else {
        return false;
    };
    
    if !have_path.eq_ignore_ascii_case(&helper_bin.to_string_lossy()) {
        return false;
    }
    
    match (local_exe_mtime_secs(), helper_bin_mtime_secs()) {
        (Some(ui), Some(helper)) => ui <= helper,
        _ => true,
    }
}

fn helper_bin_mtime_secs() -> Option<u64> {
    let meta = fs::metadata(helper_exe_path()).ok()?;
    let modified = meta.modified().ok()?;
    Some(
        modified
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_secs(),
    )
}

fn request_shutdown(token: &str) {
    let _ = request_with_timeout(
        &HelperRequest::Shutdown {
            token: token.to_string(),
        },
        Duration::from_secs(5),
    );
    std::thread::sleep(Duration::from_millis(300));
}

pub fn ensure_helper(app_data: &Path) -> Result<(), EngineFault> {
    let token = ensure_token(app_data)?;

    if helper_is_current(&token) {
        return Ok(());
    }

    
    
    request_shutdown(&token);
    kill_stale_helper();

    spawn_helper_elevated(app_data)?;

    let deadline = Instant::now() + Duration::from_secs(45);
    while Instant::now() < deadline {
        if helper_is_current(&token) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(300));
    }

    Err(EngineFault::new(
        FaultCode::UacLaunchFailed,
        "helper did not start after UAC — confirm the prompt or check antivirus",
    ))
}

fn spawn_helper_elevated(app_data: &Path) -> Result<(), EngineFault> {
    #[cfg(windows)]
    {
        let _ = app_data;
        let src = std::env::current_exe().map_err(|e| {
            EngineFault::new(FaultCode::Unknown, format!("current_exe: {e}"))
        })?;
        let dir = helper_state_dir();
        fs::create_dir_all(&dir).map_err(|e| {
            EngineFault::new(FaultCode::Unknown, format!("helper dir: {e}"))
        })?;
        let helper_bin = helper_exe_path();
        
        fs::copy(&src, &helper_bin).map_err(|e| {
            EngineFault::new(
                FaultCode::UacLaunchFailed,
                format!("cannot stage helper binary: {e}"),
            )
        })?;

        let exe_s = helper_bin.to_string_lossy().replace('\'', "''");
        let work_s = dir.to_string_lossy().replace('\'', "''");

        let script = format!(
            "$ErrorActionPreference='Stop'; try {{ \
                Start-Process -FilePath '{exe}' -ArgumentList '{flag}' \
                    -WorkingDirectory '{work}' \
                    -Verb RunAs -WindowStyle Hidden; \
                exit 0 \
             }} catch {{ \
                if ($_.Exception.Message -match 'canceled|cancelled|1223') {{ exit 1223 }}; \
                Write-Error $_; exit 1 \
             }}",
            exe = exe_s,
            work = work_s,
            flag = HELPER_FLAG,
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
        if code == 0 {
            return Ok(());
        }
        if code == 1223 {
            return Err(EngineFault::new(
                FaultCode::UacCancelled,
                "User declined the UAC prompt for zapp helper",
            ));
        }
        let err = String::from_utf8_lossy(&output.stderr);
        Err(EngineFault::new(
            FaultCode::UacLaunchFailed,
            if err.trim().is_empty() {
                format!("failed to elevate helper (exit {code})")
            } else {
                err.trim().to_string()
            },
        ))
    }

    #[cfg(not(windows))]
    {
        let _ = app_data;
        Err(EngineFault::new(
            FaultCode::Unknown,
            "helper elevation is Windows-only",
        ))
    }
}

pub fn helper_ping(app_data: &Path) -> Result<HelperResponse, EngineFault> {
    let token = ensure_token(app_data)?;
    request(&HelperRequest::Ping { token })
}

pub fn helper_status(app_data: &Path) -> Result<HelperStatus, EngineFault> {
    let token = ensure_token(app_data)?;
    let res = request_with_timeout(&HelperRequest::Status { token }, Duration::from_secs(5))?;
    res.status.ok_or_else(|| {
        EngineFault::new(
            FaultCode::Unknown,
            if res.ok {
                "helper status empty".into()
            } else {
                res.message
            },
        )
    })
}

pub fn helper_start(
    app_data: &Path,
    winws: &Path,
    work_dir: &Path,
    args: Vec<String>,
    strategy: &str,
) -> Result<HelperStatus, EngineFault> {
    ensure_helper(app_data)?;
    let token = ensure_token(app_data)?;
    
    let res = request_with_timeout(
        &HelperRequest::Start {
            token,
            winws: winws.to_string_lossy().into_owned(),
            work_dir: work_dir.to_string_lossy().into_owned(),
            args,
            strategy: strategy.to_string(),
        },
        Duration::from_secs(60),
    )?;
    if !res.ok {
        log::warn!("helper start refused/failed: {}", res.message);
        return Err(crate::engine::fault_from_helper_message(res.message));
    }
    res.status.ok_or_else(|| {
        EngineFault::new(FaultCode::Unknown, "helper start returned no status")
    })
}

pub fn helper_stop(app_data: &Path) -> Result<(), EngineFault> {
    if !token_path().exists() {
        return Ok(());
    }
    let token = ensure_token(app_data)?;
    if !helper_auth_ok(&token).unwrap_or(false) {
        // Helper unreachable — best-effort orphan cleanup only.
        crate::engine::kill_orphan_winws();
        return Ok(());
    }
    let res = request(&HelperRequest::Stop { token })?;
    if res.ok {
        Ok(())
    } else {
        Err(EngineFault::new(FaultCode::StopFailed, res.message))
    }
}

pub fn helper_reset(app_data: &Path) -> Result<(), EngineFault> {
    ensure_helper(app_data)?;
    let token = ensure_token(app_data)?;
    let res = request_with_timeout(&HelperRequest::Reset { token }, Duration::from_secs(30))?;
    if res.ok {
        Ok(())
    } else {
        Err(EngineFault::new(FaultCode::StopFailed, res.message))
    }
}

/// Elevated copy of a staged pack into `%ProgramData%\zapp\engine\<ver>` + ACL.
/// Returns the installed pack root.
pub fn helper_install_engine(
    app_data: &Path,
    version: &str,
    source_dir: &Path,
) -> Result<std::path::PathBuf, EngineFault> {
    ensure_helper(app_data)?;
    let token = ensure_token(app_data)?;
    let res = request_with_timeout(
        &HelperRequest::InstallEngine {
            token,
            version: version.to_string(),
            source_dir: source_dir.to_string_lossy().into_owned(),
        },
        Duration::from_secs(180),
    )?;
    if !res.ok {
        log::warn!("helper install engine failed: {}", res.message);
        return Err(crate::engine::fault_from_helper_message(res.message));
    }
    let dest = crate::helper::paths::program_data_engine_prefix().join(version);
    if !dest.join("bin").join("winws.exe").is_file() {
        return Err(EngineFault::new(
            FaultCode::Unknown,
            format!(
                "helper reported install ok but pack missing at {}",
                dest.display()
            ),
        ));
    }
    Ok(dest)
}

/// Ask the elevated helper to exit (Quit / update). Best-effort; ignores missing helper.
pub fn helper_shutdown(app_data: &Path) {
    if !token_path().exists() {
        return;
    }
    let Ok(token) = ensure_token(app_data) else {
        return;
    };
    if !helper_auth_ok(&token).unwrap_or(false) {
        return;
    }
    request_shutdown(&token);
}
