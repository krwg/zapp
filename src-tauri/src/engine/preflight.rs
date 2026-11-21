// ╔════════════════════════════════════════════════════╗
// ║                 engine/preflight                 ║
// ╚════════════════════════════════════════════════════╝

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use serde::{Deserialize, Serialize};

use super::elevate::is_elevated;
use super::faults::{EngineFault, FaultCode};
use super::process::{kill_image, process_running};
use super::strategy::parse_strategy_bat;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreflightCheck {
    pub id: String,
    pub label: String,
    pub ok: bool,
    pub detail: String,
    pub fault_code: Option<FaultCode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreflightReport {
    pub ok: bool,
    pub elevated: bool,
    pub checks: Vec<PreflightCheck>,
    pub blocking_fault: Option<EngineFault>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreflightScope {

    Environment,

    Strategy,

    Full,
}

pub fn run_preflight(root: &Path, strategy: &str) -> PreflightReport {
    run_preflight_scoped(root, strategy, PreflightScope::Full)
}

pub fn run_preflight_scoped(root: &Path, strategy: &str, scope: PreflightScope) -> PreflightReport {
    let t0 = Instant::now();
    let mut checks = Vec::new();

    if matches!(scope, PreflightScope::Environment | PreflightScope::Full) {

        let _ = super::pack::ensure_flowseal_sidecars(root);
        push_pack_checks(root, &mut checks);


        let foreign_winws = matches!(scope, PreflightScope::Environment);
        push_conflict_checks(&mut checks, foreign_winws);
        push_windivert_service(&mut checks);
        push_elevation(&mut checks);
    }

    if matches!(scope, PreflightScope::Strategy | PreflightScope::Full) {
        if matches!(scope, PreflightScope::Strategy) {

            let _ = super::pack::ensure_flowseal_sidecars(root);
        }
        push_strategy_checks(root, strategy, &mut checks);
    }

    let elevated = is_elevated();
    let blocking = checks
        .iter()
        .find(|c| !c.ok && c.fault_code.is_some())
        .map(|c| {
            EngineFault::new(
                c.fault_code.clone().unwrap_or(FaultCode::Unknown),
                c.detail.clone(),
            )
        });

    let report = PreflightReport {
        ok: blocking.is_none(),
        elevated,
        checks,
        blocking_fault: blocking,
    };
    log::info!(
        "timing preflight({scope:?}, '{strategy}'): {:.0}ms ok={}",
        t0.elapsed().as_secs_f64() * 1000.0,
        report.ok
    );
    report
}

fn push_pack_checks(root: &Path, checks: &mut Vec<PreflightCheck>) {
    let winws = root.join("bin").join("winws.exe");
    let winws_ok = winws.is_file();
    checks.push(PreflightCheck {
        id: "winws".into(),
        label: "winws.exe".into(),
        ok: winws_ok,
        detail: if winws_ok {
            winws.display().to_string()
        } else {
            format!("Missing: {}", winws.display())
        },
        fault_code: (!winws_ok).then_some(FaultCode::WinwsBinaryMissing),
    });

    let sys = root.join("bin").join("WinDivert64.sys");
    let dll = root.join("bin").join("WinDivert.dll");
    let divert_ok = sys.is_file() && dll.is_file();
    checks.push(PreflightCheck {
        id: "windivert".into(),
        label: "WinDivert driver files".into(),
        ok: divert_ok,
        detail: if divert_ok {
            "WinDivert64.sys + WinDivert.dll present".into()
        } else {
            format!("sys={} dll={}", sys.is_file(), dll.is_file())
        },
        fault_code: (!divert_ok).then_some(FaultCode::WinDivertDriverMissing),
    });
}

fn push_strategy_checks(root: &Path, strategy: &str, checks: &mut Vec<PreflightCheck>) {
    let bat = root.join(format!("{strategy}.bat"));
    let bat_ok = bat.is_file();
    checks.push(PreflightCheck {
        id: "strategy-bat".into(),
        label: "Strategy launcher".into(),
        ok: bat_ok,
        detail: if bat_ok {
            bat.display().to_string()
        } else {
            format!("Missing: {}", bat.display())
        },
        fault_code: (!bat_ok).then_some(FaultCode::StrategyBatMissing),
    });

    let mut parse_ok = false;
    let mut parse_detail = String::new();
    let mut payload_ok = true;
    let mut payload_detail = String::from("n/a");
    if bat_ok {
        match parse_strategy_bat(root, strategy) {
            Ok(launch) => {
                parse_ok = true;
                parse_detail = format!("{} args", launch.args.len());
                let missing: Vec<String> = launch
                    .args
                    .iter()
                    .filter_map(|a| extract_path_arg(a))
                    .filter(|p| Path::new(p).is_absolute() && !Path::new(p).exists())
                    .take(5)
                    .collect();
                if !missing.is_empty() {
                    payload_ok = false;
                    payload_detail = format!("Missing: {}", missing.join(", "));
                } else {
                    payload_detail = "Referenced files exist".into();
                }
            }
            Err(e) => {
                parse_detail = e.to_string();
            }
        }
    }
    checks.push(PreflightCheck {
        id: "strategy-parse".into(),
        label: "Parse winws arguments".into(),
        ok: parse_ok,
        detail: parse_detail,
        fault_code: (!parse_ok && bat_ok).then_some(FaultCode::StrategyParseFailed),
    });
    checks.push(PreflightCheck {
        id: "strategy-payloads".into(),
        label: "Strategy payloads / lists".into(),
        ok: payload_ok,
        detail: payload_detail,
        fault_code: (!payload_ok).then_some(FaultCode::StrategyPayloadMissing),
    });
}

fn push_conflict_checks(checks: &mut Vec<PreflightCheck>, foreign_winws: bool) {
    let conflict = detect_conflicting_bypass(foreign_winws);
    checks.push(PreflightCheck {
        id: "conflict".into(),
        label: "No conflicting bypass tools".into(),
        ok: conflict.is_none(),
        detail: conflict.clone().unwrap_or_else(|| {
            if foreign_winws {
                "No conflicting DPI tools / foreign winws detected".into()
            } else {
                "No conflicting DPI tools detected".into()
            }
        }),
        fault_code: conflict.is_some().then_some(FaultCode::ConflictingBypass),
    });
}

fn push_windivert_service(checks: &mut Vec<PreflightCheck>) {
    let windivert_svc = windivert_service_state();
    checks.push(PreflightCheck {
        id: "windivert-svc".into(),
        label: "WinDivert service state".into(),
        ok: true,
        detail: windivert_svc,
        fault_code: None,
    });
}

fn push_elevation(checks: &mut Vec<PreflightCheck>) {
    let elevated = is_elevated();
    checks.push(PreflightCheck {
        id: "elevation".into(),
        label: "Process elevation".into(),
        ok: true,
        detail: if elevated {
            "zapp is elevated — winws inherits rights".into()
        } else {
            "zapp is not elevated — one UAC starts the helper, then Start is silent".into()
        },
        fault_code: None,
    });
}

fn extract_path_arg(arg: &str) -> Option<String> {
    for prefix in [
        "--hostlist=",
        "--hostlist-exclude=",
        "--ipset=",
        "--ipset-exclude=",
        "--dpi-desync-fake-quic=",
        "--dpi-desync-fake-tls=",
        "--dpi-desync-fake-http=",
        "--dpi-desync-fake-discord=",
        "--dpi-desync-fake-stun=",
        "--dpi-desync-fake-unknown=",
        "--dpi-desync-fake-unknown-udp=",
    ] {
        if let Some(rest) = arg.strip_prefix(prefix) {
            let p = rest.trim_matches('"');
            if p.len() > 2 && (p.contains('\\') || p.contains('/')) {
                return Some(p.to_string());
            }
        }
    }
    None
}

const CONFLICT_IMAGES: &[&str] = &[
    "goodbyedpi.exe",
    "zapret_gui.exe",
    "zapretgui.exe",
    "zapret.exe",
    "ezunblock.exe",
    "blockcheck.exe",
    "soupfork.exe",
    "bye-dpi.exe",
    "byedpi.exe",
    "ciadpi.exe",
    "discord_voice_proxy.exe",
    "youtubeunblok.exe",
    "youtubeunblock.exe",
    "nfqws.exe",
];

#[derive(Debug, Clone)]
pub struct ConflictSnapshot {
    pub images: Vec<String>,
    pub detail: String,
}

pub fn list_conflicting_images(foreign_winws: bool) -> Vec<String> {
    let mut out = Vec::new();
    for name in CONFLICT_IMAGES {
        if process_running(name) {
            out.push((*name).to_string());
        }
    }
    if foreign_winws && process_running("winws.exe") {
        out.push("winws.exe".into());
    }
    out
}

pub fn detect_conflicting_bypass(foreign_winws: bool) -> Option<String> {
    let images = list_conflicting_images(foreign_winws);
    if images.is_empty() {
        return None;
    }
    if images.iter().any(|i| i == "winws.exe") && images.len() == 1 {
        return Some(
            "Another winws.exe is still running (Flowseal .bat or Windows service). Stop it, then retry Smart check."
                .into(),
        );
    }
    Some(format!(
        "{} is running — stop it before using zapp",
        images.join(", ")
    ))
}


pub fn resolve_conflicting_bypass(kill_foreign_winws: bool) -> ConflictSnapshot {
    let before = list_conflicting_images(kill_foreign_winws);
    for name in CONFLICT_IMAGES {
        if process_running(name) {
            kill_image(name);
        }
    }
    if kill_foreign_winws && process_running("winws.exe") {
        kill_image("winws.exe");
    }

    std::thread::sleep(std::time::Duration::from_millis(400));
    let after = list_conflicting_images(kill_foreign_winws);
    let detail = if after.is_empty() {
        if before.is_empty() {
            "No conflicting DPI tools detected".into()
        } else {
            format!("Stopped: {}", before.join(", "))
        }
    } else {
        format!(
            "Still running: {} — close them manually if needed",
            after.join(", ")
        )
    };
    ConflictSnapshot {
        images: after,
        detail,
    }
}

fn windivert_service_state() -> String {
    let out = hidden("sc")
        .args(["query", "WinDivert"])
        .output()
        .ok();
    match out {
        Some(o) => {
            let text = String::from_utf8_lossy(&o.stdout);
            if text.contains("RUNNING") {
                "WinDivert service RUNNING".into()
            } else if text.contains("STOPPED") {
                "WinDivert service STOPPED".into()
            } else if text.contains("1060") || text.to_ascii_lowercase().contains("does not exist") {
                "WinDivert service not installed (normal before first start)".into()
            } else {
                "WinDivert service state unknown".into()
            }
        }
        None => "Could not query WinDivert service".into(),
    }
}

fn hidden(program: &str) -> Command {
    let mut cmd = Command::new(program);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}
