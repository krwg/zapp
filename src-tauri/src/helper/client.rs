// ╔════════════════════════════════════════════════════╗
// ║                  helper/client                   ║
// ╚════════════════════════════════════════════════════╝

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use interprocess::local_socket::{prelude::*, Stream as LocalStream};
use uuid::Uuid;

use super::protocol::{HelperRequest, HelperResponse, HelperStatus};
use super::{helper_exe_path, helper_pipe_name, helper_state_dir, token_path, HELPER_FLAG};
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

        let path_s = path.to_string_lossy();
        let user = std::env::var("USERNAME").unwrap_or_else(|_| "Users".into());
        let grant_user = format!("{user}:(R)");
        let _ = Command::new("icacls")
            .args([path_s.as_ref(), "/inheritance:r"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .status();
        let _ = Command::new("icacls")
            .args([path_s.as_ref(), "/grant:r", &grant_user])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .status();
        let _ = Command::new("icacls")
            .args([path_s.as_ref(), "/grant:r", "SYSTEM:(F)"])
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

fn connect_helper_within(timeout: Duration) -> Result<LocalStream, EngineFault> {
    let name = helper_pipe_name().map_err(|e| EngineFault::new(FaultCode::Unknown, e))?;
    let deadline = Instant::now() + timeout;
    let mut last_err = None;
    loop {
        match LocalStream::connect(name.clone()) {
            Ok(s) => return Ok(s),
            Err(e) => {
                last_err = Some(e);
                if Instant::now() >= deadline {
                    break;
                }
                std::thread::sleep(Duration::from_millis(40));
            }
        }
    }
    let msg = last_err
        .map(|e| e.to_string())
        .unwrap_or_else(|| "helper not reachable".into());
    let denied = msg.to_ascii_lowercase().contains("access is denied") || msg.contains("os error 5");
    let detail = if denied {
        format!(
            "helper pipe access denied ({msg}) — elevated helper may be using a locked-down pipe ACL; restart zapp after update"
        )
    } else {
        format!("helper not reachable: {msg}")
    };
    Err(EngineFault::new(FaultCode::UacLaunchFailed, detail))
}

fn connect_helper() -> Result<LocalStream, EngineFault> {
    connect_helper_within(Duration::from_secs(3))
}

fn helper_pipe_alive_quick() -> bool {
    connect_helper_within(Duration::from_millis(120)).is_ok()
}

fn request_with_timeout(
    req: &HelperRequest,
    _read_timeout: Duration,
) -> Result<HelperResponse, EngineFault> {
    let mut stream = connect_helper()?;

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
        if !crate::engine::process_running("zapp-helper.exe") {
            return;
        }
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "zapp-helper.exe", "/T"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .status();
        std::thread::sleep(Duration::from_millis(120));
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

fn helper_ping_current(token: &str) -> bool {
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
    have_path.eq_ignore_ascii_case(&helper_bin.to_string_lossy())
}


fn helper_is_current(token: &str) -> bool {
    if !helper_ping_current(token) {
        return false;
    }


    match (local_exe_mtime_secs(), helper_bin_mtime_secs()) {
        (Some(ui), Some(helper)) => ui <= helper + 2,
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
    let t0 = Instant::now();
    let token = ensure_token(app_data)?;

    if helper_pipe_alive_quick() && helper_is_current(&token) {
        return Ok(());
    }

    // Avoid multi-second connect timeouts when no helper is listening.
    if helper_pipe_alive_quick() {
        request_shutdown(&token);
    }
    kill_stale_helper();

    spawn_helper_elevated(app_data)?;
    log::info!(
        "timing ensure_helper.uac_shown: {:.0}ms",
        t0.elapsed().as_secs_f64() * 1000.0
    );

    let deadline = Instant::now() + Duration::from_secs(45);
    while Instant::now() < deadline {
        if helper_ping_current(&token) {
            log::info!(
                "timing ensure_helper.ready: {:.0}ms",
                t0.elapsed().as_secs_f64() * 1000.0
            );
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(200));
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
        stage_helper_binary(&src, &helper_bin)?;

        match crate::win32::shell_execute_runas(&helper_bin, HELPER_FLAG, &dir) {
            Ok(()) => Ok(()),
            Err(crate::win32::ElevateError::Cancelled) => Err(EngineFault::new(
                FaultCode::UacCancelled,
                "User declined the UAC prompt for zapp helper",
            )),
            Err(crate::win32::ElevateError::Failed(msg)) => {
                Err(EngineFault::new(FaultCode::UacLaunchFailed, msg))
            }
        }
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

fn helper_binary_fresh(src: &Path, helper_bin: &Path) -> bool {
    let Ok(src_meta) = fs::metadata(src) else {
        return false;
    };
    let Ok(dst_meta) = fs::metadata(helper_bin) else {
        return false;
    };
    if src_meta.len() != dst_meta.len() {
        return false;
    }
    match (src_meta.modified(), dst_meta.modified()) {
        (Ok(s), Ok(d)) => d >= s,
        _ => false,
    }
}

fn stage_helper_binary(src: &Path, helper_bin: &Path) -> Result<(), EngineFault> {
    if helper_binary_fresh(src, helper_bin) {
        log::info!("helper binary already staged — skip copy");
        return Ok(());
    }

    let t0 = Instant::now();
    let tmp = helper_bin.with_extension("exe.staging");
    let mut last = None;
    for attempt in 0..8 {
        if attempt > 0 {
            kill_stale_helper();
            std::thread::sleep(Duration::from_millis(120 + attempt * 80));
        }
        let _ = fs::remove_file(&tmp);
        match fs::copy(src, &tmp) {
            Ok(_) => match fs::rename(&tmp, helper_bin).or_else(|_| {
                let _ = fs::remove_file(helper_bin);
                fs::rename(&tmp, helper_bin)
            }) {
                Ok(()) => {
                    log::info!(
                        "timing stage_helper_binary: {:.0}ms",
                        t0.elapsed().as_secs_f64() * 1000.0
                    );
                    return Ok(());
                }
                Err(e) => last = Some(e),
            },
            Err(e) => last = Some(e),
        }
    }
    let detail = last
        .map(|e| e.to_string())
        .unwrap_or_else(|| "unknown staging error".into());
    Err(EngineFault::new(
        FaultCode::UacLaunchFailed,
        format!("cannot stage helper binary: {detail}"),
    ))
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
