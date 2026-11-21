// ╔════════════════════════════════════════════════════╗
// ║                  helper/server                   ║
// ╚════════════════════════════════════════════════════╝

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use interprocess::local_socket::{prelude::*, ListenerOptions};

use crate::engine::kill_orphan_winws;

use super::paths::{
    apply_program_data_acl, managed_app_data_root, program_data_engine_prefix,
    program_data_zapp_root, validate_winws_paths_any,
};
use super::protocol::{HelperRequest, HelperResponse, HelperStatus};
use super::{helper_pipe_name, helper_state_dir, token_path, HELPER_PIPE};
use std::path::{Path, PathBuf};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

struct Runtime {
    child: Option<Child>,
    strategy: Option<String>,
    token: String,
}

pub fn serve() -> Result<(), String> {
    let state_dir = helper_state_dir();
    std::fs::create_dir_all(&state_dir).ok();
    let token = read_token()?;

    let name = helper_pipe_name()?;
    let listener = ListenerOptions::new()
        .name(name)
        .create_sync()
        .map_err(|e| {
            format!(
                "bind helper pipe '{HELPER_PIPE}': {e} (another helper may still be running)"
            )
        })?;

    eprintln!("zapp helper listening on named pipe '{HELPER_PIPE}'");

    let runtime = Mutex::new(Runtime {
        child: None,
        strategy: None,
        token,
    });

    for conn in listener.incoming() {
        let Ok(mut stream) = conn else { continue };
        let mut reader = BufReader::new(&mut stream);
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() {
            continue;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parsed = serde_json::from_str::<HelperRequest>(line);
        let response = match &parsed {
            Ok(req) => handle(&runtime, req.clone()),
            Err(e) => HelperResponse {
                ok: false,
                message: format!("bad request: {e}"),
                status: None,
            },
        };

        if let Ok(payload) = serde_json::to_string(&response) {
            let _ = writeln!(stream, "{payload}");
        }

        if matches!(parsed, Ok(HelperRequest::Shutdown { .. })) {
            break;
        }
    }

    let mut rt = runtime.lock().map_err(|e| e.to_string())?;
    stop_child(&mut rt);
    Ok(())
}

fn handle(runtime: &Mutex<Runtime>, req: HelperRequest) -> HelperResponse {
    let mut rt = match runtime.lock() {
        Ok(g) => g,
        Err(_) => {
            return HelperResponse {
                ok: false,
                message: "helper lock poisoned".into(),
                status: None,
            }
        }
    };

    let token = request_token(&req);
    if token != rt.token {
        return HelperResponse {
            ok: false,
            message: "unauthorized helper token".into(),
            status: None,
        };
    }

    match req {
        HelperRequest::Ping { .. } => HelperResponse {
            ok: true,
            message: "pong".into(),
            status: Some(status_of(&mut rt)),
        },
        HelperRequest::Status { .. } => HelperResponse {
            ok: true,
            message: "ok".into(),
            status: Some(status_of(&mut rt)),
        },
        HelperRequest::Start {
            winws,
            work_dir,
            args,
            strategy,
            ..
        } => {
            let t0 = Instant::now();
            // Own the Child — stop previous via kill/wait, no taskkill on hot path.
            stop_child(&mut rt);

            match spawn_winws(&winws, &work_dir, &args, &strategy) {
                Ok(mut child) => {
                    let pid = child.id();
                    let deadline = Instant::now() + Duration::from_millis(250);
                    let mut alive = true;
                    while Instant::now() < deadline {
                        if let Ok(Some(_)) = child.try_wait() {
                            alive = false;
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(15));
                    }

                    if !alive {
                        let log_tail = read_log_tail(1200);
                        rt.child = None;
                        rt.strategy = None;
                        return HelperResponse {
                            ok: false,
                            message: format!(
                                "winws exited after spawn (pid {pid}). log: {}",
                                if log_tail.trim().is_empty() {
                                    "(empty — WinDivert/AV silent fail or bad argv)"
                                        .to_string()
                                } else {
                                    log_tail
                                }
                            ),
                            status: Some(status_of(&mut rt)),
                        };
                    }

                    rt.child = Some(child);
                    rt.strategy = Some(strategy.clone());
                    eprintln!(
                        "helper start '{strategy}' pid={pid} in {:.0}ms",
                        t0.elapsed().as_secs_f64() * 1000.0
                    );
                    HelperResponse {
                        ok: true,
                        message: format!("started {strategy} pid={pid}"),
                        status: Some(status_of(&mut rt)),
                    }
                }
                Err(e) => HelperResponse {
                    ok: false,
                    message: e,
                    status: Some(status_of(&mut rt)),
                },
            }
        }
        HelperRequest::Stop { .. } => {
            stop_child(&mut rt);
            HelperResponse {
                ok: true,
                message: "stopped".into(),
                status: Some(status_of(&mut rt)),
            }
        }
        HelperRequest::Reset { .. } => {
            stop_child(&mut rt);
            kill_orphan_winws();
            let _ = hidden("net").args(["stop", "WinDivert"]).status();
            HelperResponse {
                ok: true,
                message: "reset complete".into(),
                status: Some(status_of(&mut rt)),
            }
        }
        HelperRequest::InstallEngine {
            version,
            source_dir,
            ..
        } => match install_engine_pack(&version, &source_dir) {
            Ok(dest) => HelperResponse {
                ok: true,
                message: format!("installed engine {version} -> {}", dest.display()),
                status: Some(status_of(&mut rt)),
            },
            Err(e) => HelperResponse {
                ok: false,
                message: e,
                status: Some(status_of(&mut rt)),
            },
        },
        HelperRequest::Shutdown { .. } => {
            stop_child(&mut rt);
            HelperResponse {
                ok: true,
                message: "shutting down".into(),
                status: Some(status_of(&mut rt)),
            }
        }
    }
}

fn install_engine_pack(version: &str, source_dir: &str) -> Result<PathBuf, String> {
    if version.is_empty()
        || version == "."
        || version == ".."
        || version.contains(['/', '\\', ':'])
    {
        return Err("refused: invalid engine version".into());
    }
    for c in Path::new(source_dir).components() {
        if matches!(c, std::path::Component::ParentDir) {
            return Err("refused: source_dir must not contain '..'".into());
        }
    }

    let source = Path::new(source_dir)
        .canonicalize()
        .map_err(|e| format!("refused: cannot resolve source_dir: {e}"))?;
    let app_root = managed_app_data_root()
        .canonicalize()
        .unwrap_or_else(|_| managed_app_data_root());
    let source_s = source.to_string_lossy();
    let app_s = app_root.to_string_lossy();
    if !source_s.eq_ignore_ascii_case(&app_s)
        && !source_s
            .to_ascii_lowercase()
            .starts_with(&(app_s.to_ascii_lowercase() + "\\"))
        && !source_s
            .to_ascii_lowercase()
            .starts_with(&(app_s.to_ascii_lowercase() + "/"))
    {
        return Err(format!(
            "refused: source_dir must be under {} (got {})",
            app_root.display(),
            source.display()
        ));
    }
    if !source.join("bin").join("winws.exe").is_file() {
        return Err("refused: source_dir is not a Flowseal pack (bin/winws.exe missing)".into());
    }

    let dest_prefix = program_data_engine_prefix();
    let dest = dest_prefix.join(version);
    let zapp_root = program_data_zapp_root();
    std::fs::create_dir_all(&zapp_root)
        .map_err(|e| format!("cannot create ProgramData root: {e}"))?;
    apply_program_data_acl(&zapp_root)?;

    if dest.exists() {
        std::fs::remove_dir_all(&dest)
            .map_err(|e| format!("cannot clear previous pack: {e}"))?;
    }
    copy_dir_recursive(&source, &dest)?;
    apply_program_data_acl(&zapp_root)?;

    let current = dest_prefix.join("current");
    let _ = std::fs::remove_dir_all(&current);
    let _ = std::fs::remove_file(&current);
    #[cfg(windows)]
    {
        let status = hidden("cmd")
            .args([
                "/C",
                "mklink",
                "/J",
                current.to_str().unwrap_or_default(),
                dest.to_str().unwrap_or_default(),
            ])
            .status()
            .map_err(|e| format!("mklink failed: {e}"))?;
        if !status.success() {
            copy_dir_recursive(&dest, &current)?;
        }
    }
    #[cfg(not(windows))]
    {
        std::os::unix::fs::symlink(&dest, &current)
            .map_err(|e| format!("symlink current failed: {e}"))?;
    }

    Ok(dest)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| format!("mkdir {}: {e}", dst.display()))?;
    for entry in std::fs::read_dir(src).map_err(|e| format!("read_dir {}: {e}", src.display()))? {
        let entry = entry.map_err(|e| format!("dir entry: {e}"))?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            if let Some(parent) = to.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
            }
            std::fs::copy(&from, &to)
                .map_err(|e| format!("copy {} -> {}: {e}", from.display(), to.display()))?;
        }
    }
    Ok(())
}

fn spawn_winws(
    winws: &str,
    work_dir: &str,
    args: &[String],
    strategy: &str,
) -> Result<Child, String> {
    let (winws_path, work_path) =
        validate_winws_paths_any(Path::new(winws), Path::new(work_dir))?;
    let winws = winws_path.to_string_lossy();
    let work_dir = work_path.to_string_lossy();

    let state_dir = helper_state_dir();
    std::fs::create_dir_all(&state_dir).ok();
    let log_path = state_dir.join("winws.last.log");
    let cmd_path = state_dir.join("winws.launch.cmd");

    let mut script = String::new();
    script.push_str("@echo off\r\n");
    script.push_str("setlocal DisableDelayedExpansion\r\n");
    script.push_str(&format!("cd /d \"{}\"\r\n", work_dir));
    script.push('"');
    script.push_str(&winws);
    script.push('"');
    for a in args {
        script.push(' ');
        script.push_str(&quote_cmd_arg(a));
    }
    script.push_str("\r\n");
    let _ = std::fs::write(&cmd_path, &script);

    let mut header = format!(
        "strategy={strategy}\r\nwinws={winws}\r\nwork_dir={work_dir}\r\nargs={}\r\n",
        args.len()
    );
    for (i, a) in args.iter().enumerate() {
        header.push_str(&format!("  [{i}] {a}\r\n"));
    }
    header.push_str("--- stderr/stdout ---\r\n");
    let _ = std::fs::write(&log_path, header.as_bytes());

    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| format!("cannot open winws log: {e}"))?;
    let log_err = log_file
        .try_clone()
        .map_err(|e| format!("cannot clone log handle: {e}"))?;

    let mut cmd = Command::new(winws.as_ref());
    cmd.args(args)
        .current_dir(work_dir.as_ref())
        .stdin(Stdio::null())
        .stdout(Stdio::from(log_file))
        .stderr(Stdio::from(log_err));
    #[cfg(windows)]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    cmd.spawn()
        .map_err(|e| format!("spawn winws failed: {e}"))
}

fn quote_cmd_arg(arg: &str) -> String {
    if arg.is_empty() {
        return "\"\"".into();
    }
    let needs = arg.chars().any(|c| {
        matches!(c, ' ' | '\t' | '"' | '&' | '|' | '<' | '>' | '^' | '%')
    });
    if !needs {
        return arg.to_string();
    }
    let escaped = arg.replace('"', "\\\"");
    format!("\"{escaped}\"")
}

fn read_log_tail(max_chars: usize) -> String {
    let path = helper_state_dir().join("winws.last.log");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return String::new();
    };
    if text.chars().count() <= max_chars {
        return text;
    }
    text.chars()
        .rev()
        .take(max_chars)
        .collect::<String>()
        .chars()
        .rev()
        .collect()
}

fn request_token(req: &HelperRequest) -> &str {
    match req {
        HelperRequest::Ping { token }
        | HelperRequest::Status { token }
        | HelperRequest::Start { token, .. }
        | HelperRequest::Stop { token }
        | HelperRequest::Reset { token }
        | HelperRequest::InstallEngine { token, .. }
        | HelperRequest::Shutdown { token } => token,
    }
}

fn status_of(rt: &mut Runtime) -> HelperStatus {
    let alive = child_alive(&mut rt.child);
    if !alive {
        rt.child = None;
        rt.strategy = None;
    }
    HelperStatus {
        running: alive,
        strategy: rt.strategy.clone(),
        pid: rt.child.as_ref().map(|c| c.id()),
        elevated: true,
        exe_path: std::env::current_exe()
            .ok()
            .map(|p| p.to_string_lossy().into_owned()),
        exe_mtime: exe_mtime_secs(),
    }
}

fn exe_mtime_secs() -> Option<u64> {
    let path = std::env::current_exe().ok()?;
    let meta = std::fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?;
    Some(
        modified
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_secs(),
    )
}

fn stop_child(rt: &mut Runtime) {
    if let Some(mut child) = rt.child.take() {
        let _ = child.kill();
        let _ = child.wait();
    }
    rt.strategy = None;
}

fn child_alive(child: &mut Option<Child>) -> bool {
    match child {
        Some(c) => matches!(c.try_wait(), Ok(None)),
        None => false,
    }
}

fn hidden(program: &str) -> Command {
    let mut cmd = Command::new(program);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

fn read_token() -> Result<String, String> {
    let path = token_path();
    std::fs::read_to_string(&path)
        .map(|s| s.trim().to_string())
        .map_err(|e| format!("helper token missing at {}: {e}", path.display()))
}
