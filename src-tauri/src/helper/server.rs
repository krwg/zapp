// ╔════════════════════════════════════════════════════╗
// ║                  helper/server                   ║
// ╚════════════════════════════════════════════════════╝

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use super::protocol::{HelperRequest, HelperResponse, HelperStatus};
use super::{helper_state_dir, token_path, HELPER_PORT};

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

    let listener = TcpListener::bind(("127.0.0.1", HELPER_PORT)).map_err(|e| {
        format!(
            "bind helper port {HELPER_PORT}: {e} (another helper may still be running)"
        )
    })?;
    listener
        .set_nonblocking(false)
        .map_err(|e| e.to_string())?;

    eprintln!("zapp helper listening on 127.0.0.1:{HELPER_PORT}");

    let runtime = Mutex::new(Runtime {
        child: None,
        strategy: None,
        token,
    });

    for conn in listener.incoming() {
        let Ok(mut stream) = conn else { continue };
        let mut reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);
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
            stop_child(&mut rt);
            kill_winws_os();
            
            
            std::thread::sleep(Duration::from_millis(200));

            match spawn_winws(&winws, &work_dir, &args, &strategy) {
                Ok(child) => {
                    let pid = child.id();
                    rt.child = Some(child);
                    rt.strategy = Some(strategy.clone());

                    
                    let mut alive = false;
                    for _ in 0..30 {
                        std::thread::sleep(Duration::from_millis(200));
                        if winws_running() || child_alive(&mut rt.child) {
                            alive = true;
                            std::thread::sleep(Duration::from_millis(500));
                            alive = winws_running() || child_alive(&mut rt.child);
                            break;
                        }
                    }

                    if !alive {
                        let log_tail = read_log_tail(1200);
                        let _ = stop_child(&mut rt);
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
            kill_winws_os();
            HelperResponse {
                ok: true,
                message: "stopped".into(),
                status: Some(status_of(&mut rt)),
            }
        }
        HelperRequest::Reset { .. } => {
            stop_child(&mut rt);
            kill_winws_os();
            
            let _ = hidden("net").args(["stop", "WinDivert"]).status();
            HelperResponse {
                ok: true,
                message: "reset complete".into(),
                status: Some(status_of(&mut rt)),
            }
        }
        HelperRequest::Shutdown { .. } => {
            stop_child(&mut rt);
            kill_winws_os();
            HelperResponse {
                ok: true,
                message: "shutting down".into(),
                status: Some(status_of(&mut rt)),
            }
        }
    }
}

fn spawn_winws(
    winws: &str,
    work_dir: &str,
    args: &[String],
    strategy: &str,
) -> Result<Child, String> {
    let state_dir = helper_state_dir();
    std::fs::create_dir_all(&state_dir).ok();
    let log_path = state_dir.join("winws.last.log");
    let cmd_path = state_dir.join("winws.launch.cmd");

    let mut script = String::new();
    script.push_str("@echo off\r\n");
    script.push_str("setlocal DisableDelayedExpansion\r\n");
    script.push_str(&format!("cd /d \"{}\"\r\n", work_dir));
    script.push('"');
    script.push_str(winws);
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

    let mut cmd = Command::new(winws);
    cmd.args(args)
        .current_dir(work_dir)
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
        | HelperRequest::Shutdown { token } => token,
    }
}

fn status_of(rt: &mut Runtime) -> HelperStatus {
    let alive = child_alive(&mut rt.child) || winws_running();
    if !alive {
        rt.child = None;
        if !winws_running() {
            rt.strategy = None;
        }
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

fn winws_running() -> bool {
    hidden("tasklist")
        .args(["/FI", "IMAGENAME eq winws.exe", "/NH"])
        .output()
        .map(|out| {
            String::from_utf8_lossy(&out.stdout)
                .to_ascii_lowercase()
                .contains("winws.exe")
        })
        .unwrap_or(false)
}

fn kill_winws_os() {
    let _ = hidden("taskkill")
        .args(["/IM", "winws.exe", "/F", "/T"])
        .status();
    let _ = hidden("taskkill")
        .args(["/FI", "WINDOWTITLE eq zapret:*", "/F"])
        .status();
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
