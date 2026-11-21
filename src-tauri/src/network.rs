// ╔════════════════════════════════════════════════════╗
// ║                     network                      ║
// ╚════════════════════════════════════════════════════╝

use std::process::{Command, Stdio};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkFingerprint {
    pub gateway: Option<String>,
    pub ssid: Option<String>,
}

impl NetworkFingerprint {
    pub fn key(&self) -> String {
        format!(
            "{}|{}",
            self.gateway.as_deref().unwrap_or("-"),
            self.ssid.as_deref().unwrap_or("-")
        )
    }
}

pub fn current_fingerprint() -> NetworkFingerprint {
    NetworkFingerprint {
        gateway: default_gateway_v4(),
        ssid: wlan_ssid(),
    }
}

fn default_gateway_v4() -> Option<String> {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("route");
        cmd.args(["print", "-4"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        cmd.creation_flags(CREATE_NO_WINDOW);
        let out = cmd.output().ok()?;
        let text = String::from_utf8_lossy(&out.stdout);

        for line in text.lines() {
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() >= 3 && cols[0] == "0.0.0.0" {
                let gw = cols[2];
                if gw != "0.0.0.0" && gw.contains('.') {
                    return Some(gw.to_string());
                }
            }
        }
        None
    }
    #[cfg(not(windows))]
    {
        None
    }
}

fn wlan_ssid() -> Option<String> {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("netsh");
        cmd.args(["wlan", "show", "interfaces"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        cmd.creation_flags(CREATE_NO_WINDOW);
        let out = cmd.output().ok()?;
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("SSID") && !trimmed.starts_with("BSSID") {
                if let Some(rest) = trimmed.split_once(':') {
                    let ssid = rest.1.trim();
                    if !ssid.is_empty() {
                        return Some(ssid.to_string());
                    }
                }
            }
        }
        None
    }
    #[cfg(not(windows))]
    {
        None
    }
}
