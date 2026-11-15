// ╔════════════════════════════════════════════════════╗
// ║                   engine/probe                   ║
// ╚════════════════════════════════════════════════════╝

use std::net::{TcpStream, ToSocketAddrs};
use std::thread;
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

#[derive(Debug, Clone, Copy)]
pub struct DiagTarget {
    pub service: &'static str,
    pub host: &'static str,
    pub port: u16,
    /// When true, run HTTPS after a successful TCP connect.
    pub https: bool,
    /// Soft hosts affect the score but cannot alone fail the service.
    pub critical: bool,
}

/// Critical endpoints used by Smart Check.
/// Discord needs gateway + `discord.media` on Flowseal alt ports (2053…).
pub fn diag_targets() -> Vec<DiagTarget> {
    vec![
        DiagTarget {
            service: "youtube",
            host: "youtube.com",
            port: 443,
            https: true,
            critical: true,
        },
        DiagTarget {
            service: "discord",
            host: "discord.com",
            port: 443,
            https: true,
            critical: true,
        },
        DiagTarget {
            service: "discord",
            host: "gateway.discord.gg",
            port: 443,
            https: true,
            critical: true,
        },
        // Matches `--hostlist-domains=discord.media --filter-tcp=2053,...` in Flowseal bats.
        DiagTarget {
            service: "discord",
            host: "discord.media",
            port: 2053,
            https: false,
            critical: true,
        },
        DiagTarget {
            service: "discord",
            host: "cdn.discordapp.com",
            port: 443,
            https: true,
            critical: false,
        },
    ]
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

/// Fast adaptive matrix: TCP:443 gate, then HTTPS if TCP lives.
pub fn probe_matrix_host(host: &str, timeout: Duration) -> Vec<ProbeResult> {
    let tcp_budget = timeout.min(Duration::from_millis(700));
    let tcp = probe_tcp(host, 443, tcp_budget);
    if !tcp.ok {
        return vec![tcp];
    }
    let https_budget = timeout.min(Duration::from_millis(900));
    let https = probe_https(host, https_budget);
    vec![tcp, https]
}

fn probe_matrix_target(target: &DiagTarget, timeout: Duration) -> Vec<ProbeResult> {
    let tcp_budget = timeout.min(Duration::from_millis(700));
    let mut tcp = probe_tcp(target.host, target.port, tcp_budget);
    if target.port != 443 {
        tcp.note = format!("TCP:{} {}", target.port, if tcp.ok { "OK" } else { "fail" });
        // Alt-port media endpoints: try a couple of Flowseal ports if the first fails.
        if !tcp.ok {
            for alt in [2083u16, 8443] {
                let mut retry = probe_tcp(target.host, alt, tcp_budget);
                if retry.ok {
                    retry.note = format!("TCP:{alt} OK");
                    return vec![retry];
                }
            }
            return vec![tcp];
        }
        return vec![tcp];
    }
    if !tcp.ok || !target.https {
        return vec![tcp];
    }
    let https_budget = timeout.min(Duration::from_millis(900));
    let https = probe_https(target.host, https_budget);
    vec![tcp, https]
}

/// Probe every host in parallel (scoped threads). Order matches `targets`.
pub fn probe_targets_parallel(
    targets: &[DiagTarget],
    timeout: Duration,
) -> Vec<(DiagTarget, Vec<ProbeResult>)> {
    thread::scope(|scope| {
        let mut handles = Vec::with_capacity(targets.len());
        for target in targets {
            handles.push(scope.spawn(move || {
                let probes = probe_matrix_target(target, timeout);
                (*target, probes)
            }));
        }
        handles
            .into_iter()
            .map(|h| {
                h.join().unwrap_or_else(|_| {
                    (
                        DiagTarget {
                            service: "unknown",
                            host: "unknown",
                            port: 443,
                            https: true,
                            critical: true,
                        },
                        Vec::new(),
                    )
                })
            })
            .collect()
    })
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

    let https_ok = results.iter().any(|r| r.ok && r.protocol == "https");
    let https_bonus = if https_ok { 6 } else { 0 };

    (reach + latency + https_bonus).min(100)
}

#[derive(Debug, Clone)]
pub struct ServiceScore {
    pub service: String,
    pub score: u32,
    pub ok: bool,
}

#[derive(Debug, Clone)]
pub struct AdaptiveScore {
    pub score: u32,
    pub success: bool,
    pub services: Vec<ServiceScore>,
    pub all_probes: Vec<ProbeResult>,
}

/// Score by service. A strategy only "works" when every critical service passes.
/// Within a service, the worst *critical* host decides pass/fail so gateway
/// failures cannot hide behind a living discord.com homepage. Soft hosts
/// (CDN) only nudge the numeric score.
pub fn score_by_service(
    host_results: &[(DiagTarget, Vec<ProbeResult>)],
    pass_threshold: u32,
) -> AdaptiveScore {
    let mut all_probes = Vec::new();
    let mut by_service: Vec<(&str, Vec<(bool, u32)>)> = Vec::new();

    for (target, probes) in host_results {
        all_probes.extend(probes.iter().cloned());
        let host_score = score_probes(probes);
        if let Some((_, scores)) = by_service.iter_mut().find(|(s, _)| *s == target.service) {
            scores.push((target.critical, host_score));
        } else {
            by_service.push((target.service, vec![(target.critical, host_score)]));
        }
    }

    let services: Vec<ServiceScore> = by_service
        .into_iter()
        .map(|(service, scores)| {
            let critical: Vec<u32> = scores
                .iter()
                .filter(|(critical, _)| *critical)
                .map(|(_, s)| *s)
                .collect();
            let soft: Vec<u32> = scores
                .iter()
                .filter(|(critical, _)| !*critical)
                .map(|(_, s)| *s)
                .collect();

            let critical_floor = critical.iter().copied().min().unwrap_or(0);
            let soft_avg = if soft.is_empty() {
                critical_floor
            } else {
                soft.iter().sum::<u32>() / soft.len() as u32
            };
            // 85% critical floor, 15% soft hosts — soft cannot rescue a dead gateway.
            let score = ((critical_floor * 85) + (soft_avg * 15)) / 100;
            ServiceScore {
                service: service.into(),
                score,
                ok: critical_floor >= pass_threshold,
            }
        })
        .collect();

    let success = !services.is_empty() && services.iter().all(|s| s.ok);
    let score = if services.is_empty() {
        0
    } else {
        services.iter().map(|s| s.score).sum::<u32>() / services.len() as u32
    };

    AdaptiveScore {
        score,
        success,
        services,
        all_probes,
    }
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
        ("gateway.discord.gg", 443),
        ("github.com", 443),
    ]
}
