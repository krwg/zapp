// ╔════════════════════════════════════════════════════╗
// ║                  engine/managed                  ║
// ╚════════════════════════════════════════════════════╝

use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::engine::elevate;
use crate::engine::faults::{EngineFault, FaultCode};
use crate::engine::preflight::{run_preflight_scoped, PreflightScope};
use crate::engine::process::{kill_orphan_winws, process_pid_running};
use crate::engine::strategy::parse_strategy_bat;
use crate::error::{AppError, AppResult};
use crate::model::EngineStatus;
use crate::util::now_iso;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, Clone)]
pub struct EngineLayout {
    pub root: std::path::PathBuf,
    pub version: String,
    pub binary: std::path::PathBuf,
    #[allow(dead_code)]
    pub strategies_dir: std::path::PathBuf,
}

impl EngineLayout {
    pub fn under_app_data(app_data: std::path::PathBuf, version: impl Into<String>) -> Self {
        let version = version.into();
        let root = app_data.join("engine").join(&version);
        Self::from_pack_with_version(root, version)
    }

    pub fn from_pack(root: std::path::PathBuf) -> Self {
        let version = crate::engine::discover::read_local_version(&root);
        Self::from_pack_with_version(root, version)
    }

    fn from_pack_with_version(root: std::path::PathBuf, version: String) -> Self {
        Self {
            binary: root.join("bin").join(if cfg!(windows) {
                "winws.exe"
            } else {
                "nfqws"
            }),
            strategies_dir: root.join("lists"),
            root,
            version,
        }
    }

    pub fn is_installed(&self) -> bool {
        self.binary.exists()
    }

    #[allow(dead_code)]
    pub fn strategy_bat(&self, strategy_name: &str) -> std::path::PathBuf {
        self.root.join(format!("{strategy_name}.bat"))
    }
}

struct EngineInner {
    status: EngineStatus,
    layout: EngineLayout,
    active_strategy: Option<String>,
    session_started_at: Option<String>,
    uptime_anchor_secs: Option<i64>,
    last_error: Option<String>,
    child: Option<Child>,
    child_pid: Option<u32>,
    via_helper: bool,
}

pub struct ManagedEngine {
    app_data: std::path::PathBuf,
    inner: Mutex<EngineInner>,
}

#[derive(Debug, Clone, Copy)]
pub struct StartOpts {
    pub allow_elevate: bool,

    pub env_preflight: bool,
}

impl ManagedEngine {
    pub fn new(app_data: std::path::PathBuf, layout: EngineLayout) -> Self {

        kill_orphan_winws();
        Self {
            app_data,
            inner: Mutex::new(EngineInner {
                status: EngineStatus::Stopped,
                layout,
                active_strategy: Some("general (FAKE TLS AUTO)".into()),
                session_started_at: None,
                uptime_anchor_secs: None,
                last_error: None,
                child: None,
                child_pid: None,
                via_helper: false,
            }),
        }
    }

    pub fn app_data(&self) -> &std::path::Path {
        &self.app_data
    }

    pub fn status(&self) -> EngineStatus {
        self.inner.lock().expect("engine lock").status.clone()
    }


    pub fn winws_alive(&self) -> bool {
        let via_helper = {
            let inner = self.inner.lock().expect("engine lock");
            inner.via_helper
        };
        if via_helper {
            return crate::helper::helper_status(&self.app_data)
                .map(|st| st.running)
                .unwrap_or(false);
        }
        let mut inner = self.inner.lock().expect("engine lock");
        child_alive(&mut inner.child)
    }

    pub fn layout(&self) -> EngineLayout {
        self.inner.lock().expect("engine lock").layout.clone()
    }

    pub fn active_strategy(&self) -> Option<String> {
        self.inner
            .lock()
            .expect("engine lock")
            .active_strategy
            .clone()
    }

    pub fn set_active_strategy(&self, name: impl Into<String>) {
        self.inner.lock().expect("engine lock").active_strategy = Some(name.into());
    }

    pub fn session_started_at(&self) -> Option<String> {
        self.inner
            .lock()
            .expect("engine lock")
            .session_started_at
            .clone()
    }

    pub fn last_error(&self) -> Option<String> {
        self.inner.lock().expect("engine lock").last_error.clone()
    }

    pub fn uptime_seconds(&self) -> u64 {
        let inner = self.inner.lock().expect("engine lock");
        match (inner.status.clone(), inner.uptime_anchor_secs) {
            (EngineStatus::Running, Some(anchor)) => {
                let now = chrono::Utc::now().timestamp();
                (now - anchor).max(0) as u64
            }
            _ => 0,
        }
    }

    pub fn set_running(&self, running: bool) -> AppResult<EngineStatus> {
        if running {
            self.start(None)
        } else {
            self.stop()
        }
    }

    pub fn start(&self, strategy_override: Option<&str>) -> AppResult<EngineStatus> {
        self.start_inner(
            strategy_override,
            StartOpts {
                allow_elevate: true,
                env_preflight: true,
            },
        )
    }

    pub fn start_no_elevate(&self, strategy_override: Option<&str>) -> AppResult<EngineStatus> {
        self.start_inner(
            strategy_override,
            StartOpts {
                allow_elevate: false,
                env_preflight: true,
            },
        )
    }


    pub fn start_diag(&self, strategy_override: Option<&str>) -> AppResult<EngineStatus> {
        self.start_inner(
            strategy_override,
            StartOpts {
                allow_elevate: false,
                env_preflight: false,
            },
        )
    }

    fn start_inner(
        &self,
        strategy_override: Option<&str>,
        opts: StartOpts,
    ) -> AppResult<EngineStatus> {
        let t_all = Instant::now();



        {
            let mut inner = self.inner.lock().expect("engine lock");
            inner.status = EngineStatus::Starting;
            if let Some(mut child) = inner.child.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
            let via_helper = inner.via_helper;
            inner.child_pid = None;
            inner.via_helper = false;
            drop(inner);
            if via_helper {
                let _ = crate::helper::helper_stop(&self.app_data);
            }
        }

        let mut inner = self.inner.lock().expect("engine lock");

        if !inner.layout.is_installed() {
            let fault = EngineFault::new(
                FaultCode::EngineNotInstalled,
                format!("winws missing at {}", inner.layout.binary.display()),
            );
            inner.status = EngineStatus::Error;
            inner.last_error = Some(fault.message());
            return Err(fault.into());
        }

        let strategy = strategy_override
            .map(str::to_string)
            .or_else(|| inner.active_strategy.clone())
            .unwrap_or_else(|| "general (FAKE TLS AUTO)".into());
        let layout_root = inner.layout.root.clone();
        let already_elevated = elevate::is_elevated();
        drop(inner);

        // Show UAC as early as possible — before preflight / strategy parse.
        if !already_elevated && opts.allow_elevate {
            let t_uac = Instant::now();
            if let Err(fault) = crate::helper::ensure_helper(&self.app_data) {
                let mut inner = self.inner.lock().expect("engine lock");
                inner.status = EngineStatus::Error;
                inner.last_error = Some(fault.message());
                return Err(fault.into());
            }
            log::info!(
                "timing start_inner.ensure_helper: {:.0}ms",
                t_uac.elapsed().as_secs_f64() * 1000.0
            );
        }

        let scope = if opts.env_preflight {
            PreflightScope::Full
        } else {
            PreflightScope::Strategy
        };
        let t_pf = Instant::now();
        let report = run_preflight_scoped(&layout_root, &strategy, scope);
        log::info!(
            "timing start_inner.preflight: {:.0}ms",
            t_pf.elapsed().as_secs_f64() * 1000.0
        );
        if let Some(fault) = report.blocking_fault {
            log::warn!("preflight blocked start: {}", fault.log_line());
            let mut inner = self.inner.lock().expect("engine lock");
            inner.status = EngineStatus::Error;
            inner.last_error = Some(fault.message());
            return Err(fault.into());
        }

        let launch = match parse_strategy_bat(&layout_root, &strategy) {
            Ok(l) => l,
            Err(e) => {
                let fault = EngineFault::new(FaultCode::StrategyParseFailed, e.to_string());
                let mut inner = self.inner.lock().expect("engine lock");
                inner.status = EngineStatus::Error;
                inner.last_error = Some(fault.message());
                return Err(fault.into());
            }
        };

        log::info!(
            "parsed strategy='{strategy}' args={} winws={}",
            launch.args.len(),
            launch.winws.display()
        );

        if !already_elevated && !opts.allow_elevate {
            if crate::helper::helper_ping(&self.app_data).is_err() {
                let fault = EngineFault::new(
                    FaultCode::DiagnosticsNeedsAdmin,
                    "Helper not running — press Start once to approve UAC",
                );
                let mut inner = self.inner.lock().expect("engine lock");
                inner.status = EngineStatus::Error;
                inner.last_error = Some(fault.message());
                return Err(fault.into());
            }
        }

        let app_data = self.app_data.clone();
        let winws = launch.winws.clone();
        let work_dir = launch.work_dir.clone();
        let args = launch.args.clone();

        let t_spawn = Instant::now();
        let (start_res, via_helper) = if already_elevated {
            let mut cmd = Command::new(&winws);
            cmd.args(&args)
                .current_dir(&work_dir)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            #[cfg(windows)]
            {
                cmd.creation_flags(CREATE_NO_WINDOW);
            }
            match cmd.spawn() {
                Ok(mut child) => {

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
                        (
                            Err(EngineFault::new(
                                FaultCode::WinwsExitedImmediately,
                                "winws exited immediately after spawn",
                            )),
                            false,
                        )
                    } else {
                        let pid = child.id();
                        let mut inner = self.inner.lock().expect("engine lock");
                        inner.child = Some(child);
                        inner.child_pid = Some(pid);
                        (Ok(Some(pid)), false)
                    }
                }
                Err(e) => (
                    Err(EngineFault::new(
                        FaultCode::AccessDenied,
                        format!("Failed to start winws: {e}"),
                    )),
                    false,
                ),
            }
        } else {
            (
                crate::helper::helper_start(&app_data, &winws, &work_dir, args, &strategy)
                    .map(|st| st.pid),
                true,
            )
        };
        log::info!(
            "timing start_inner.spawn/helper: {:.0}ms via_helper={via_helper}",
            t_spawn.elapsed().as_secs_f64() * 1000.0
        );

        let mut inner = self.inner.lock().expect("engine lock");
        inner.active_strategy = Some(strategy.clone());
        inner.via_helper = via_helper;

        match start_res {
            Ok(pid) => {
                let mut alive = child_alive(&mut inner.child);
                if !alive && via_helper {
                    if let Ok(st) = crate::helper::helper_status(&app_data) {
                        alive = st.running;
                    }
                }

                if alive {
                    inner.status = EngineStatus::Running;
                    inner.session_started_at = Some(now_iso());
                    inner.uptime_anchor_secs = Some(chrono::Utc::now().timestamp());
                    if let Some(pid) = pid {
                        inner.child_pid = Some(pid);
                    }
                    inner.last_error = None;
                    log::info!(
                        "zapp started winws strategy='{strategy}' via_helper={via_helper} pid={:?} pack={} total={:.0}ms",
                        pid,
                        inner.layout.version,
                        t_all.elapsed().as_secs_f64() * 1000.0
                    );
                    Ok(inner.status.clone())
                } else {
                    let fault = EngineFault::new(
                        FaultCode::WinwsExitedImmediately,
                        if via_helper {
                            "helper reports winws is not running after start"
                        } else {
                            "winws did not stay running after start"
                        },
                    );
                    log::error!("{}", fault.log_line());
                    inner.status = EngineStatus::Error;
                    inner.last_error = Some(fault.message());
                    inner.child = None;
                    inner.child_pid = None;
                    inner.via_helper = false;
                    Err(fault.into())
                }
            }
            Err(fault) => {
                log::error!("winws spawn failed: {}", fault.log_line());
                inner.status = EngineStatus::Error;
                inner.last_error = Some(fault.message());
                inner.via_helper = false;
                Err(fault.into())
            }
        }
    }


    pub fn shutdown_for_exit(&self) {
        let _ = self.stop();
        crate::helper::helper_shutdown(&self.app_data);
    }

    pub fn stop(&self) -> AppResult<EngineStatus> {
        let t0 = Instant::now();
        let mut inner = self.inner.lock().expect("engine lock");
        inner.status = EngineStatus::Stopping;

        if let Some(mut child) = inner.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        inner.child_pid = None;
        let via_helper = inner.via_helper;
        inner.via_helper = false;
        drop(inner);

        if via_helper {
            let _ = crate::helper::helper_stop(&self.app_data);
        }

        let mut inner = self.inner.lock().expect("engine lock");
        inner.status = EngineStatus::Stopped;
        inner.session_started_at = None;
        inner.uptime_anchor_secs = None;
        inner.last_error = None;
        log::info!(
            "zapp stopped winws ({:.0}ms)",
            t0.elapsed().as_secs_f64() * 1000.0
        );
        Ok(EngineStatus::Stopped)
    }

    pub fn reset(&self) -> AppResult<EngineStatus> {
        let _ = crate::helper::helper_reset(&self.app_data);
        kill_orphan_winws();
        let mut inner = self.inner.lock().expect("engine lock");
        inner.status = EngineStatus::Stopped;
        inner.session_started_at = None;
        inner.uptime_anchor_secs = None;
        inner.child = None;
        inner.child_pid = None;
        inner.via_helper = false;
        inner.last_error = None;
        Ok(EngineStatus::Stopped)
    }

    pub fn adopt_layout(&self, layout: EngineLayout) -> AppResult<()> {
        let mut inner = self.inner.lock().expect("engine lock");
        if matches!(
            inner.status,
            EngineStatus::Running | EngineStatus::Starting | EngineStatus::Stopping
        ) {
            return Err(AppError::from(
                "Stop the engine before switching to another zapret build",
            ));
        }
        inner.layout = layout;
        Ok(())
    }

    pub fn refresh_status_from_os(&self) {
        let mut inner = self.inner.lock().expect("engine lock");
        if matches!(
            inner.status,
            EngineStatus::Starting | EngineStatus::Stopping
        ) {
            return;
        }

        let via_helper = inner.via_helper;
        let owned_pid = inner.child_pid;
        let app_data = self.app_data.clone();
        let local = child_alive(&mut inner.child);
        drop(inner);


        let alive = if via_helper {
            match crate::helper::helper_status(&app_data) {
                Ok(st) => st.running,
                Err(_) => owned_pid.is_some_and(process_pid_running),
            }
        } else {
            local || owned_pid.is_some_and(process_pid_running)
        };

        let mut inner = self.inner.lock().expect("engine lock");


        let _ = (local, via_helper);
        if !alive && matches!(inner.status, EngineStatus::Running) {
            inner.status = EngineStatus::Stopped;
            inner.session_started_at = None;
            inner.uptime_anchor_secs = None;
            inner.child = None;
            inner.child_pid = None;
            inner.via_helper = false;
        }
    }
}

fn child_alive(child: &mut Option<Child>) -> bool {
    match child {
        Some(c) => matches!(c.try_wait(), Ok(None)),
        None => false,
    }
}
