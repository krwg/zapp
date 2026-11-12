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
    pub weight: u32,
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
                weight: 1,
            };
        }
    };

    if resolved.is_empty() {
        return ProbeResult {
            ok: false,
            latency_ms: None,
            note: "DNS returned no addresses".into(),
            protocol: "tcp",
            weight: 1,
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
                    weight: 1,
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
        weight: 1,
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
                weight: 3,
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
                weight: 3,
            }
        }
        Err(e) => ProbeResult {
            ok: false,
            latency_ms: None,
            note: format!("HTTPS failed: {e}"),
            protocol: "https",
            weight: 3,
        },
    }
}

/// Fast adaptive matrix: TCP gate, then HTTPS if TCP lives.
pub fn probe_matrix_host(host: &str, timeout: Duration) -> Vec<ProbeResult> {
    let tcp_budget = timeout.min(Duration::from_millis(900));
    let tcp = probe_tcp(host, 443, tcp_budget);
    if !tcp.ok {
        return vec![tcp];
    }
    let https_budget = timeout.min(Duration::from_millis(1200));
    let https = probe_https(host, https_budget);
    vec![tcp, https]
}

fn latency_points(ms: u32) -> u32 {
    match ms {
        0..=60 => 24,
        61..=100 => 18,
        101..=160 => 12,
        161..=250 => 7,
        251..=400 => 3,
        _ => 1,
    }
}

/// Multi-factor score 0–100: reachability × latency × protocol weight.
pub fn score_probes(results: &[ProbeResult]) -> u32 {
    if results.is_empty() {
        return 0;
    }

    let weight_total: u32 = results.iter().map(|r| r.weight.max(1)).sum();
    let reach: u32 = results
        .iter()
        .filter(|r| r.ok)
        .map(|r| r.weight.max(1) * 70)
        .sum::<u32>()
        / weight_total.max(1);

    let lat_samples: Vec<u32> = results.iter().filter_map(|r| r.latency_ms).collect();
    let latency = if lat_samples.is_empty() {
        0
    } else {
        let sum: u32 = lat_samples.iter().copied().map(latency_points).sum();
        (sum / lat_samples.len() as u32).min(24)
    };

    let https_ok = results
        .iter()
        .any(|r| r.ok && r.protocol == "https");
    let https_bonus = if https_ok { 6 } else { 0 };

    (reach + latency + https_bonus).min(100)
}

/// Soft prior for strategy names — tie-break only, never overrides bad probes.
pub fn strategy_prior_bonus(name: &str) -> i32 {
    let l = name.to_ascii_lowercase();
    if l.contains("fake tls auto") && !l.contains("alt") && !l.contains("exp") {
        3
    } else if l.contains("fake tls auto") {
        1
    } else if l.contains("exp") {
        -2
    } else {
        0
    }
}

pub fn rank_score(probe_score: u32, strategy_name: &str) -> u32 {
    let adjusted = probe_score as i32 + strategy_prior_bonus(strategy_name);
    adjusted.clamp(0, 100) as u32
}

#[allow(dead_code)]
pub fn default_targets() -> Vec<(&'static str, u16)> {
    vec![
        ("youtube.com", 443),
        ("discord.com", 443),
        ("github.com", 443),
    ]
}
