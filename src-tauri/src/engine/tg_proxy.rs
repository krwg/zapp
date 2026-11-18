// ╔════════════════════════════════════════════════════╗
// ║                 engine/tg_proxy                  ║
// ╚════════════════════════════════════════════════════╝
//! Managed Flowseal tg-ws-proxy sidecar (user-level, not via elevated helper).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use uuid::Uuid;

use crate::engine::process::process_pid_running;
use crate::error::{AppError, AppResult};
use crate::model::EngineStatus;
use crate::util::now_iso;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

const DEFAULT_PORT: u16 = 1443;
const DEFAULT_HOST: &str = "127.0.0.1";
const EXE_NAME: &str = "TgWsProxy_windows.exe";

struct Inner {
    status: EngineStatus,
    child: Option<Child>,
    child_pid: Option<u32>,
    session_started_at: Option<String>,
    uptime_anchor_secs: Option<i64>,
    last_error: Option<String>,
    version: String,
    port: u16,
    host: String,
    secret: String,
}

pub struct ManagedTgProxy {
    app_data: PathBuf,
    inner: Mutex<Inner>,
}

impl ManagedTgProxy {
    pub fn new(app_data: PathBuf) -> Self {
        let (secret, port, host) = load_or_create_secret_config();
        let version = read_installed_version(&app_data).unwrap_or_else(|| "0.0.0".into());
        Self {
            app_data,
            inner: Mutex::new(Inner {
                status: EngineStatus::Stopped,
                child: None,
                child_pid: None,
                session_started_at: None,
                uptime_anchor_secs: None,
                last_error: None,
                version,
                port,
                host,
                secret,
            }),
        }
    }

    pub fn pack_root(&self) -> PathBuf {
        current_pack_dir(&self.app_data)
    }

    pub fn is_installed(&self) -> bool {
        exe_path(&self.app_data).is_file()
    }

    pub fn version(&self) -> String {
        self.inner.lock().expect("tg lock").version.clone()
    }

    pub fn set_version(&self, version: impl Into<String>) {
        self.inner.lock().expect("tg lock").version = version.into();
    }

    pub fn status(&self) -> EngineStatus {
        self.inner.lock().expect("tg lock").status.clone()
    }

    pub fn port(&self) -> u16 {
        self.inner.lock().expect("tg lock").port
    }

    pub fn host(&self) -> String {
        self.inner.lock().expect("tg lock").host.clone()
    }

    pub fn secret(&self) -> String {
        self.inner.lock().expect("tg lock").secret.clone()
    }

    pub fn session_started_at(&self) -> Option<String> {
        self.inner
            .lock()
            .expect("tg lock")
            .session_started_at
            .clone()
    }

    pub fn last_error(&self) -> Option<String> {
        self.inner.lock().expect("tg lock").last_error.clone()
    }

    pub fn uptime_seconds(&self) -> u64 {
        let inner = self.inner.lock().expect("tg lock");
        match (inner.status.clone(), inner.uptime_anchor_secs) {
            (EngineStatus::Running, Some(anchor)) => {
                (chrono::Utc::now().timestamp() - anchor).max(0) as u64
            }
            _ => 0,
        }
    }

    /// `tg://proxy?server=…&port=…&secret=dd{secret}` — Flowseal uses `dd` prefix.
    pub fn proxy_link(&self) -> String {
        let inner = self.inner.lock().expect("tg lock");
        format!(
            "tg://proxy?server={}&port={}&secret=dd{}",
            inner.host, inner.port, inner.secret
        )
    }

    pub fn set_running(&self, running: bool) -> AppResult<EngineStatus> {
        if running {
            self.start()
        } else {
            self.stop()
        }
    }

    pub fn start(&self) -> AppResult<EngineStatus> {
        let exe = exe_path(&self.app_data);
        if !exe.is_file() {
            let mut inner = self.inner.lock().expect("tg lock");
            inner.status = EngineStatus::Error;
            let msg = "Telegram proxy not installed — update the Telegram proxy channel".to_string();
            inner.last_error = Some(msg.clone());
            return Err(AppError::from(msg));
        }

        // Stop previous owned child if any.
        let _ = self.stop();

        let (host, port, secret) = {
            let mut inner = self.inner.lock().expect("tg lock");
            inner.status = EngineStatus::Starting;
            inner.last_error = None;
            (inner.host.clone(), inner.port, inner.secret.clone())
        };

        write_flowseal_config(&host, port, &secret)?;

        let mut cmd = Command::new(&exe);
        cmd.current_dir(exe.parent().unwrap_or_else(|| Path::new(".")))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(windows)]
        {
            // Flowseal tray UI still appears; CREATE_NO_WINDOW only hides a console.
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let mut child = cmd.spawn().map_err(|e| {
            let msg = format!("Failed to start Telegram proxy: {e}");
            let mut inner = self.inner.lock().expect("tg lock");
            inner.status = EngineStatus::Error;
            inner.last_error = Some(msg.clone());
            AppError::from(msg)
        })?;

        let pid = child.id();
        let deadline = Instant::now() + Duration::from_millis(800);
        let mut alive = true;
        while Instant::now() < deadline {
            if let Ok(Some(_)) = child.try_wait() {
                alive = false;
                break;
            }
            // Prefer listening port as readiness.
            if port_open(&host, port) {
                alive = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(40));
        }

        let mut inner = self.inner.lock().expect("tg lock");
        if !alive && !port_open(&host, port) {
            let msg = "Telegram proxy exited immediately after start".to_string();
            inner.status = EngineStatus::Error;
            inner.last_error = Some(msg.clone());
            inner.child = None;
            inner.child_pid = None;
            return Err(AppError::from(msg));
        }

        inner.child = Some(child);
        inner.child_pid = Some(pid);
        inner.status = EngineStatus::Running;
        inner.session_started_at = Some(now_iso());
        inner.uptime_anchor_secs = Some(chrono::Utc::now().timestamp());
        inner.last_error = None;
        Ok(EngineStatus::Running)
    }

    pub fn stop(&self) -> AppResult<EngineStatus> {
        let mut inner = self.inner.lock().expect("tg lock");
        inner.status = EngineStatus::Stopping;
        if let Some(mut child) = inner.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        } else if let Some(pid) = inner.child_pid {
            kill_pid(pid);
        }
        // Also stop a foreign/stale TgWsProxy we didn't own (best-effort).
        kill_image_named("TgWsProxy_windows.exe");
        inner.child_pid = None;
        inner.status = EngineStatus::Stopped;
        inner.session_started_at = None;
        inner.uptime_anchor_secs = None;
        inner.last_error = None;
        Ok(EngineStatus::Stopped)
    }

    pub fn refresh_status_from_os(&self) {
        let mut inner = self.inner.lock().expect("tg lock");
        if matches!(
            inner.status,
            EngineStatus::Starting | EngineStatus::Stopping
        ) {
            return;
        }
        let local = match &mut inner.child {
            Some(c) => matches!(c.try_wait(), Ok(None)),
            None => false,
        };
        let pid_alive = inner.child_pid.is_some_and(process_pid_running);
        let listening = port_open(&inner.host, inner.port);
        let alive = local || pid_alive || (inner.status == EngineStatus::Running && listening);
        if !alive && matches!(inner.status, EngineStatus::Running) {
            inner.status = EngineStatus::Stopped;
            inner.session_started_at = None;
            inner.uptime_anchor_secs = None;
            inner.child = None;
            inner.child_pid = None;
        }
    }

    pub fn adopt_install(&self, version: &str) {
        let mut inner = self.inner.lock().expect("tg lock");
        inner.version = version.to_string();
    }

    pub fn shutdown_for_exit(&self) {
        let _ = self.stop();
    }

    pub fn snapshot_state(&self) -> crate::model::TelegramProxyState {
        self.refresh_status_from_os();
        let inner = self.inner.lock().expect("tg lock");
        crate::model::TelegramProxyState {
            status: inner.status.clone(),
            installed: exe_path(&self.app_data).is_file(),
            version: inner.version.clone(),
            host: inner.host.clone(),
            port: inner.port,
            session_started_at: inner.session_started_at.clone(),
            uptime_seconds: match (inner.status.clone(), inner.uptime_anchor_secs) {
                (EngineStatus::Running, Some(anchor)) => {
                    (chrono::Utc::now().timestamp() - anchor).max(0) as u64
                }
                _ => 0,
            },
            last_error: inner.last_error.clone(),
            proxy_link: format!(
                "tg://proxy?server={}&port={}&secret=dd{}",
                inner.host, inner.port, inner.secret
            ),
        }
    }
}

pub fn current_pack_dir(app_data: &Path) -> PathBuf {
    app_data.join("tg-ws-proxy").join("current")
}

pub fn version_pack_dir(app_data: &Path, version: &str) -> PathBuf {
    app_data.join("tg-ws-proxy").join(version)
}

pub fn exe_path(app_data: &Path) -> PathBuf {
    current_pack_dir(app_data).join(EXE_NAME)
}

fn read_installed_version(app_data: &Path) -> Option<String> {
    let marker = current_pack_dir(app_data).join("VERSION");
    fs::read_to_string(marker)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn install_exe_bytes(app_data: &Path, version: &str, bytes: &[u8]) -> AppResult<PathBuf> {
    let dir = version_pack_dir(app_data, version);
    fs::create_dir_all(&dir).map_err(|e| AppError::from(e.to_string()))?;
    let dest = dir.join(EXE_NAME);
    fs::write(&dest, bytes).map_err(|e| AppError::from(e.to_string()))?;
    fs::write(dir.join("VERSION"), version).ok();

    let current = current_pack_dir(app_data);
    let _ = fs::remove_dir_all(&current);
    #[cfg(windows)]
    {
        // Junction via cmd — same pattern as engine current link.
        let _ = Command::new("cmd")
            .args([
                "/C",
                "mklink",
                "/J",
                &current.to_string_lossy(),
                &dir.to_string_lossy(),
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .status();
        if !current.exists() {
            // Fallback: copy
            fs::create_dir_all(&current).ok();
            let _ = fs::copy(&dest, current.join(EXE_NAME));
            let _ = fs::write(current.join("VERSION"), version);
        }
    }
    #[cfg(not(windows))]
    {
        fs::create_dir_all(&current).ok();
        let _ = fs::copy(&dest, current.join(EXE_NAME));
        let _ = fs::write(current.join("VERSION"), version);
    }
    Ok(dest)
}

fn flowseal_config_dir() -> PathBuf {
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    base.join("TgWsProxy")
}

fn our_secret_path() -> PathBuf {
    let base = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    base.join("zapp").join("tg-proxy.json")
}

fn load_or_create_secret_config() -> (String, u16, String) {
    let path = our_secret_path();
    if let Ok(raw) = fs::read_to_string(&path) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
            let secret = v
                .get("secret")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let port = v.get("port").and_then(|x| x.as_u64()).unwrap_or(DEFAULT_PORT as u64)
                as u16;
            let host = v
                .get("host")
                .and_then(|x| x.as_str())
                .unwrap_or(DEFAULT_HOST)
                .to_string();
            if secret.len() >= 32 {
                return (secret, port, host);
            }
        }
    }
    let secret = Uuid::new_v4().simple().to_string() + &Uuid::new_v4().simple().to_string();
    let secret = secret.chars().take(32).collect::<String>();
    let _ = fs::create_dir_all(path.parent().unwrap_or_else(|| Path::new(".")));
    let payload = serde_json::json!({
        "secret": secret,
        "port": DEFAULT_PORT,
        "host": DEFAULT_HOST,
    });
    let _ = fs::write(&path, payload.to_string());
    harden_secret_acl(&path);
    (secret, DEFAULT_PORT, DEFAULT_HOST.into())
}

fn harden_secret_acl(path: &Path) {
    #[cfg(windows)]
    {
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

fn write_flowseal_config(host: &str, port: u16, secret: &str) -> AppResult<()> {
    let dir = flowseal_config_dir();
    fs::create_dir_all(&dir).map_err(|e| AppError::from(e.to_string()))?;
    let path = dir.join("config.json");
    let mut cfg = if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .unwrap_or_else(|| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    if let Some(obj) = cfg.as_object_mut() {
        obj.insert("host".into(), serde_json::json!(host));
        obj.insert("port".into(), serde_json::json!(port));
        obj.insert("secret".into(), serde_json::json!(secret));
    }
    fs::write(&path, serde_json::to_string_pretty(&cfg).unwrap_or_default())
        .map_err(|e| AppError::from(e.to_string()))?;
    Ok(())
}

fn port_open(host: &str, port: u16) -> bool {
    use std::net::{SocketAddr, TcpStream};
    let Ok(addr) = format!("{host}:{port}").parse::<SocketAddr>() else {
        return false;
    };
    TcpStream::connect_timeout(&addr, Duration::from_millis(120)).is_ok()
}

fn kill_pid(pid: u32) {
    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
        use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
        unsafe {
            let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
            if !handle.is_null() && handle != INVALID_HANDLE_VALUE {
                let _ = TerminateProcess(handle, 1);
                CloseHandle(handle);
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = pid;
    }
}

fn kill_image_named(image: &str) {
    crate::engine::process::kill_image(image);
}
