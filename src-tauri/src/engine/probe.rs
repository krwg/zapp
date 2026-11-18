// ╔════════════════════════════════════════════════════╗
// ║                   engine/probe                   ║
// ╚════════════════════════════════════════════════════╝

use std::net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
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
    /// When true, run HTTPS (skip redundant TCP:443).
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

#[derive(Debug, Clone)]
pub struct ResolvedTarget {
    pub target: DiagTarget,
    pub addrs: Vec<SocketAddr>,
}

/// Prefer IPv4 — bare AAAA with no v6 route causes false connect timeouts.
fn prefer_ipv4(addrs: impl IntoIterator<Item = SocketAddr>) -> Vec<SocketAddr> {
    let all: Vec<SocketAddr> = addrs.into_iter().collect();
    let v4: Vec<SocketAddr> = all
        .iter()
        .copied()
        .filter(|a| matches!(a.ip(), IpAddr::V4(_)))
        .collect();
    if !v4.is_empty() {
        v4
    } else {
        all
    }
}

fn resolve_host_timed(host: &str, port: u16, timeout: Duration) -> Vec<SocketAddr> {
    let host_owned = host.to_string();
    let label = format!("{host}:{port}");
    let (tx, rx) = std::sync::mpsc::channel();
    thread::spawn(move || {
        let addrs = format!("{host_owned}:{port}")
            .to_socket_addrs()
            .map(|i| prefer_ipv4(i))
            .unwrap_or_default();
        let _ = tx.send(addrs);
    });
    match rx.recv_timeout(timeout) {
        Ok(addrs) => addrs,
        Err(_) => {
            log::warn!("DNS resolve timed out for {label} after {timeout:?}");
            Vec::new()
        }
    }
}

/// Resolve DNS once before the strategy loop — DNS is strategy-independent.
pub fn resolve_diag_targets(targets: &[DiagTarget]) -> Vec<ResolvedTarget> {
    let t0 = Instant::now();
    let dns_budget = Duration::from_millis(1500);
    let resolved: Vec<ResolvedTarget> = targets
        .iter()
        .map(|target| {
            let mut addrs = resolve_host_timed(target.host, target.port, dns_budget);
            // Also resolve :443 for HTTPS hosts so resolve() has usable addrs.
            if target.https && target.port != 443 {
                for a in resolve_host_timed(target.host, 443, dns_budget) {
                    if !addrs.contains(&a) {
                        addrs.push(a);
                    }
                }
            }
            ResolvedTarget {
                target: *target,
                addrs,
            }
        })
        .collect();
    log::info!(
        "timing dns_resolve({} hosts): {:.0}ms",
        targets.len(),
        t0.elapsed().as_secs_f64() * 1000.0
    );
    resolved
}

pub fn probe_tcp(host: &str, port: u16, timeout: Duration) -> ProbeResult {
    probe_tcp_addrs(host, port, &[], timeout)
}

fn probe_tcp_addrs(host: &str, port: u16, pre: &[SocketAddr], timeout: Duration) -> ProbeResult {
    probe_tcp_addrs_abortable(host, port, pre, timeout, None)
}

fn probe_tcp_addrs_abortable(
    host: &str,
    port: u16,
    pre: &[SocketAddr],
    timeout: Duration,
    abort: Option<&AtomicBool>,
) -> ProbeResult {
    let start = Instant::now();
    let per_addr = timeout.min(Duration::from_millis(350));
    let resolved: Vec<SocketAddr> = if !pre.is_empty() {
        let mut addrs: Vec<SocketAddr> = prefer_ipv4(pre.iter().map(|a| SocketAddr::new(a.ip(), port)));
        addrs.sort_by_key(|a| a.ip().to_string());
        addrs.dedup();
        addrs
    } else {
        let addrs = resolve_host_timed(host, port, timeout.min(Duration::from_millis(1500)));
        if addrs.is_empty() {
            return ProbeResult {
                ok: false,
                latency_ms: None,
                note: "DNS failed or timed out".into(),
                protocol: "tcp",
                weight: 1,
            };
        }
        addrs
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

    for socket in &resolved {
        if abort.is_some_and(|a| a.load(Ordering::Relaxed)) {
            return ProbeResult {
                ok: false,
                latency_ms: None,
                note: "Aborted".into(),
                protocol: "tcp",
                weight: 1,
            };
        }
        let slice = per_addr.min(timeout.saturating_sub(start.elapsed()));
        if slice.is_zero() {
            break;
        }
        match TcpStream::connect_timeout(socket, slice) {
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

/// Parallel alt-port TCP with a shared deadline (discord.media 2053/2083/8443).
/// Any open port counts as success; notes record which port answered.
fn probe_tcp_ports_parallel(
    host: &str,
    ports: &[u16],
    pre: &[SocketAddr],
    timeout: Duration,
) -> ProbeResult {
    let start = Instant::now();
    let results: Vec<ProbeResult> = thread::scope(|scope| {
        let mut handles = Vec::with_capacity(ports.len());
        for &port in ports {
            handles.push(scope.spawn(move || {
                let mut r = probe_tcp_addrs(host, port, pre, timeout);
                if r.ok {
                    r.note = format!("TCP:{port} OK");
                } else {
                    r.note = format!("TCP:{port} fail");
                }
                r
            }));
        }
        handles
            .into_iter()
            .filter_map(|h| h.join().ok())
            .collect()
    });

    if let Some(ok) = results.iter().find(|r| r.ok).cloned() {
        return ok;
    }

    let tried = ports
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join("/");
    ProbeResult {
        ok: false,
        latency_ms: None,
        note: format!("TCP fail on {tried} ({:.0}ms)", start.elapsed().as_secs_f64() * 1000.0),
        protocol: "tcp",
        weight: 1,
    }
}

fn shared_https_client() -> &'static reqwest::blocking::Client {
    static CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .timeout(Duration::from_millis(1100))
            .connect_timeout(Duration::from_millis(700))
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .pool_max_idle_per_host(4)
            .user_agent("zapp/0.2")
            .build()
            .expect("https client")
    })
}

fn build_https_client(host: &str, addrs: &[SocketAddr], timeout: Duration) -> Result<reqwest::blocking::Client, String> {
    if addrs.is_empty() {
        // Shared client is fine when we let the system resolver pick IPs.
        let _ = timeout;
        return Ok(shared_https_client().clone());
    }

    let mut builder = reqwest::blocking::Client::builder()
        .timeout(timeout)
        .connect_timeout(timeout.min(Duration::from_millis(700)))
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .pool_max_idle_per_host(0)
        .user_agent("zapp/0.2");

    // Map hostname → IPs so the URL keeps the real host (SNI/Host intact).
    let https_addrs: Vec<SocketAddr> = prefer_ipv4(
        addrs
            .iter()
            .filter(|a| a.port() == 443)
            .copied()
            .chain(addrs.iter().map(|a| SocketAddr::new(a.ip(), 443))),
    );
    let mut https_addrs = https_addrs;
    https_addrs.sort_by_key(|a| a.ip().to_string());
    https_addrs.dedup();
    if !https_addrs.is_empty() {
        builder = builder.resolve_to_addrs(host, &https_addrs);
    }

    builder.build().map_err(|e| e.to_string())
}

pub fn probe_https(host: &str, timeout: Duration) -> ProbeResult {
    probe_https_addrs(host, &[], timeout)
}

fn probe_https_addrs(host: &str, addrs: &[SocketAddr], timeout: Duration) -> ProbeResult {
    // Hostname in the URL (not a raw IP) so TLS SNI matches the real site.
    let url = format!("https://{host}/");
    let start = Instant::now();
    let client = match build_https_client(host, addrs, timeout) {
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

    // HEAD→GET only on Method Not Allowed / Not Implemented — never on timeout.
    let result = match client.head(&url).send() {
        Ok(resp) => {
            let code = resp.status().as_u16();
            if code == 405 || code == 501 {
                client.get(&url).send()
            } else {
                Ok(resp)
            }
        }
        Err(e) => Err(e),
    };

    match result {
        Ok(resp) => {
            let ms = start.elapsed().as_millis().min(u32::MAX as u128) as u32;
            let code = resp.status().as_u16();
            // 2xx/3xx/4xx (incl. 301 with redirect::none) mean the host is reachable.
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
    // Keep for Sites page: still TCP gate then HTTPS.
    let tcp_budget = timeout.min(Duration::from_millis(700));
    let tcp = probe_tcp(host, 443, tcp_budget);
    if !tcp.ok {
        return vec![tcp];
    }
    let https_budget = timeout.min(Duration::from_millis(900));
    let https = probe_https(host, https_budget);
    vec![tcp, https]
}

fn probe_matrix_resolved(
    resolved: &ResolvedTarget,
    timeout: Duration,
    abort: Option<&AtomicBool>,
) -> Vec<ProbeResult> {
    if abort.is_some_and(|a| a.load(Ordering::Relaxed)) {
        return vec![ProbeResult {
            ok: false,
            latency_ms: None,
            note: "Aborted".into(),
            protocol: "skip",
            weight: 1,
        }];
    }

    let target = &resolved.target;
    let addrs = &resolved.addrs;

    if !target.https {
        // TCP-only (discord.media): try Flowseal alt ports in parallel.
        let ports: Vec<u16> = if target.port == 2053 {
            vec![2053, 2083, 8443]
        } else {
            vec![target.port]
        };
        let tcp_budget = timeout.min(Duration::from_millis(700));
        let tcp = probe_tcp_ports_parallel(target.host, &ports, addrs, tcp_budget);
        return vec![tcp];
    }

    // HTTPS targets: skip redundant TCP — one TLS is enough.
    let https_budget = if abort.is_some_and(|a| a.load(Ordering::Relaxed)) {
        timeout.min(Duration::from_millis(250))
    } else {
        timeout.min(Duration::from_millis(900))
    };
    vec![probe_https_addrs(target.host, addrs, https_budget)]
}

/// Probe hosts in parallel. Critical hosts first; soft hosts skipped if a critical fails.
pub fn probe_targets_parallel(
    targets: &[DiagTarget],
    timeout: Duration,
) -> Vec<(DiagTarget, Vec<ProbeResult>)> {
    let resolved = resolve_diag_targets(targets);
    probe_resolved_parallel(&resolved, timeout, None)
}

pub fn probe_resolved_parallel(
    resolved: &[ResolvedTarget],
    timeout: Duration,
    external_cancel: Option<Arc<Mutex<bool>>>,
) -> Vec<(DiagTarget, Vec<ProbeResult>)> {
    let t0 = Instant::now();
    let critical: Vec<&ResolvedTarget> = resolved.iter().filter(|r| r.target.critical).collect();
    let soft: Vec<&ResolvedTarget> = resolved.iter().filter(|r| !r.target.critical).collect();

    let abort = Arc::new(AtomicBool::new(false));
    let cancelled = |flag: &Option<Arc<Mutex<bool>>>| {
        flag.as_ref()
            .map(|c| *c.lock().expect("flag"))
            .unwrap_or(false)
    };

    let critical_results: Vec<(DiagTarget, Vec<ProbeResult>)> = thread::scope(|scope| {
        let mut handles = Vec::with_capacity(critical.len());
        for item in &critical {
            let abort = Arc::clone(&abort);
            let ext = external_cancel.clone();
            handles.push(scope.spawn(move || {
                if abort.load(Ordering::Relaxed) || cancelled(&ext) {
                    return (
                        item.target,
                        vec![ProbeResult {
                            ok: false,
                            latency_ms: None,
                            note: "Skipped".into(),
                            protocol: "skip",
                            weight: 1,
                        }],
                    );
                }
                if cancelled(&ext) {
                    abort.store(true, Ordering::Relaxed);
                }
                let probes = probe_matrix_resolved(item, timeout, Some(&abort));
                let host_ok = score_probes(&probes) >= 55 || probes.iter().any(|p| p.ok);
                // Early-cancel siblings when a critical host is clearly down.
                if !host_ok && probes.iter().all(|p| !p.ok) {
                    abort.store(true, Ordering::Relaxed);
                }
                if cancelled(&ext) {
                    abort.store(true, Ordering::Relaxed);
                }
                (item.target, probes)
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
    });

    if cancelled(&external_cancel) {
        log::info!(
            "timing probe_batch: {:.0}ms cancelled",
            t0.elapsed().as_secs_f64() * 1000.0
        );
        return critical_results;
    }

    let critical_failed = critical_results.iter().any(|(_, probes)| {
        probes.is_empty() || probes.iter().all(|p| !p.ok)
    });

    let mut out = critical_results;

    if !critical_failed && !soft.is_empty() && !cancelled(&external_cancel) {
        let soft_results: Vec<(DiagTarget, Vec<ProbeResult>)> = thread::scope(|scope| {
            let mut handles = Vec::with_capacity(soft.len());
            for item in &soft {
                handles.push(scope.spawn(move || {
                    let probes = probe_matrix_resolved(item, timeout, None);
                    (item.target, probes)
                }));
            }
            handles
                .into_iter()
                .filter_map(|h| h.join().ok())
                .collect()
        });
        out.extend(soft_results);
    } else if critical_failed {
        for item in soft {
            out.push((
                item.target,
                vec![ProbeResult {
                    ok: false,
                    latency_ms: None,
                    note: "Skipped after critical failure".into(),
                    protocol: "skip",
                    weight: 1,
                }],
            ));
        }
    }

    // Preserve original order.
    let mut ordered = Vec::with_capacity(resolved.len());
    for item in resolved {
        if let Some(found) = out.iter().find(|(t, _)| t.host == item.target.host && t.port == item.target.port) {
            ordered.push(found.clone());
        }
    }

    log::info!(
        "timing probe_batch: {:.0}ms critical_failed={critical_failed}",
        t0.elapsed().as_secs_f64() * 1000.0
    );
    ordered
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
