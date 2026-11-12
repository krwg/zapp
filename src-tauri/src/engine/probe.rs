// ╔════════════════════════════════════════════════════╗
// ║                   engine/probe                   ║
// ╚════════════════════════════════════════════════════╝

use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

use crate::model::AvailabilityStatus;

#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub ok: bool,
    pub latency_ms: Option<u32>,
    pub note: String,
    #[allow(dead_code)]
    pub protocol: &'static str,
}

impl ProbeResult {
    pub fn status(&self) -> AvailabilityStatus {
        if self.ok {
            AvailabilityStatus::Available
        } else {
            AvailabilityStatus::Unavailable
        }
    }
}

pub fn probe_tcp(host: &str, port: u16, timeout: Duration) -> ProbeResult {
    let addr = format!("{host}:{port}");
    let start = Instant::now();

    let resolved = match addr.to_socket_addrs() {
        Ok(iter) => iter.collect::<Vec<_>>(),
        Err(e) => {
            return ProbeResult {
                ok: false,
                latency_ms: None,
                note: format!("DNS failed: {e}"),
                protocol: "tcp",
            };
        }
    };

    if resolved.is_empty() {
        return ProbeResult {
            ok: false,
            latency_ms: None,
            note: "DNS returned no addresses".into(),
            protocol: "tcp",
        };
    }

    for socket in resolved {
        match TcpStream::connect_timeout(&socket, timeout) {
            Ok(_) => {
                let ms = start.elapsed().as_millis().min(u32::MAX as u128) as u32;
                return ProbeResult {
                    ok: true,
                    latency_ms: Some(ms),
                    note: "TCP OK".into(),
                    protocol: "tcp",
                };
            }
            Err(_) => continue,
        }
    }

    ProbeResult {
        ok: false,
        latency_ms: None,
        note: "TCP connect failed".into(),
        protocol: "tcp",
    }
}

pub fn probe_https(host: &str, timeout: Duration) -> ProbeResult {
    let url = format!("https://{host}/");
    let start = Instant::now();
    let client = match reqwest::blocking::Client::builder()
        .timeout(timeout)
        .redirect(reqwest::redirect::Policy::limited(3))
        .user_agent("zapp/0.2")
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return ProbeResult {
                ok: false,
                latency_ms: None,
                note: format!("HTTP client error: {e}"),
                protocol: "https",
            };
        }
    };

    match client.head(&url).send().or_else(|_| client.get(&url).send()) {
        Ok(resp) => {
            let ms = start.elapsed().as_millis().min(u32::MAX as u128) as u32;
            let code = resp.status().as_u16();
            let ok = code < 500;
            ProbeResult {
                ok,
                latency_ms: Some(ms),
                note: format!("HTTPS {code}"),
                protocol: "https",
            }
        }
        Err(e) => ProbeResult {
            ok: false,
            latency_ms: None,
            note: format!("HTTPS failed: {e}"),
            protocol: "https",
        },
    }
}

pub fn probe_matrix_host(host: &str, timeout: Duration) -> Vec<ProbeResult> {
    let mut out = Vec::with_capacity(2);
    out.push(probe_tcp(host, 443, timeout));
    out.push(probe_https(host, timeout));
    out
}

pub fn score_probes(results: &[ProbeResult]) -> u32 {
    if results.is_empty() {
        return 0;
    }
    let ok = results.iter().filter(|r| r.ok).count() as u32;
    let total = results.len() as u32;
    let base = (ok * 80) / total;
    let latency_bonus: u32 = results
        .iter()
        .filter_map(|r| r.latency_ms)
        .map(|ms| {
            if ms < 80 {
                20
            } else if ms < 150 {
                12
            } else if ms < 300 {
                6
            } else {
                2
            }
        })
        .sum::<u32>()
        / total.max(1);
    (base + latency_bonus).min(100)
}

#[allow(dead_code)]
pub fn default_targets() -> Vec<(&'static str, u16)> {
    vec![
        ("youtube.com", 443),
        ("discord.com", 443),
        ("github.com", 443),
    ]
}
