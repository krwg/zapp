// ╔════════════════════════════════════════════════════╗
// ║                   engine/probe                   ║
// ╚════════════════════════════════════════════════════╝

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

use crate::model::{AvailabilityStatus, PathVerdict};

const DNS_CACHE_TTL: Duration = Duration::from_secs(90);

struct DnsCacheEntry {
    addrs: Vec<SocketAddr>,
    at: Instant,
}

fn dns_cache() -> &'static Mutex<HashMap<(String, u16), DnsCacheEntry>> {
    static CACHE: OnceLock<Mutex<HashMap<(String, u16), DnsCacheEntry>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Drop cached DNS after network change so probes re-resolve.
pub fn invalidate_dns_cache() {
    if let Ok(mut g) = dns_cache().lock() {
        g.clear();
    }
}

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

    pub https: bool,

    pub quic: bool,

    pub critical: bool,
}




pub fn diag_targets() -> Vec<DiagTarget> {
    vec![
        DiagTarget {
            service: "youtube",
            host: "youtube.com",
            port: 443,
            https: true,
            quic: false,
            critical: true,
        },
        DiagTarget {
            service: "youtube",
            host: "www.youtube.com",
            port: 443,
            https: true,
            quic: false,
            critical: true,
        },

        DiagTarget {
            service: "youtube",
            host: "i.ytimg.com",
            port: 443,
            https: true,
            quic: false,
            critical: false,
        },

        DiagTarget {
            service: "youtube",
            host: "youtubei.googleapis.com",
            port: 443,
            https: true,
            quic: false,
            critical: false,
        },

        DiagTarget {
            service: "youtube",
            host: "googlevideo.com",
            port: 443,
            https: true,
            quic: false,
            critical: false,
        },

        DiagTarget {
            service: "youtube",
            host: "www.youtube.com",
            port: 443,
            https: false,
            quic: true,
            critical: false,
        },
        DiagTarget {
            service: "discord",
            host: "discord.com",
            port: 443,
            https: true,
            quic: false,
            critical: true,
        },
        DiagTarget {
            service: "discord",
            host: "gateway.discord.gg",
            port: 443,
            https: true,
            quic: false,
            critical: true,
        },

        DiagTarget {
            service: "discord",
            host: "discord.media",
            port: 2053,
            https: false,
            quic: false,
            critical: true,
        },
        DiagTarget {
            service: "discord",
            host: "cdn.discordapp.com",
            port: 443,
            https: true,
            quic: false,
            critical: false,
        },

        DiagTarget {
            service: "discord",
            host: "stun.cloudflare.com",
            port: 3478,
            https: false,
            quic: false,
            critical: false,
        },

        DiagTarget {
            service: "discord",
            host: "discord.com",
            port: 443,
            https: false,
            quic: true,
            critical: false,
        },
    ]
}

#[derive(Debug, Clone)]
pub struct ResolvedTarget {
    pub target: DiagTarget,
    pub addrs: Vec<SocketAddr>,
}


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
    let key = (host.to_ascii_lowercase(), port);
    if let Ok(g) = dns_cache().lock() {
        if let Some(hit) = g.get(&key) {
            if hit.at.elapsed() < DNS_CACHE_TTL {
                return hit.addrs.clone();
            }
        }
    }

    let label = format!("{host}:{port}");
    let (tx, rx) = std::sync::mpsc::channel();
    thread::spawn(move || {
        let addrs = format!("{host_owned}:{port}")
            .to_socket_addrs()
            .map(|i| prefer_ipv4(i))
            .unwrap_or_default();
        let _ = tx.send(addrs);
    });
    let addrs = match rx.recv_timeout(timeout) {
        Ok(addrs) => addrs,
        Err(_) => {
            log::warn!("DNS resolve timed out for {label} after {timeout:?}");
            Vec::new()
        }
    };
    if !addrs.is_empty() {
        if let Ok(mut g) = dns_cache().lock() {
            g.insert(
                key,
                DnsCacheEntry {
                    addrs: addrs.clone(),
                    at: Instant::now(),
                },
            );
        }
    }
    addrs
}


pub fn resolve_diag_targets(targets: &[DiagTarget]) -> Vec<ResolvedTarget> {
    let t0 = Instant::now();
    let dns_budget = Duration::from_millis(1500);
    let mut unique: Vec<(String, u16)> = Vec::new();
    for target in targets {
        let host = target.host.to_ascii_lowercase();
        let pair = (host.clone(), target.port);
        if !unique.iter().any(|u| u == &pair) {
            unique.push(pair);
        }
        if target.https && target.port != 443 {
            let https = (host, 443u16);
            if !unique.iter().any(|u| u == &https) {
                unique.push(https);
            }
        }
    }

    thread::scope(|scope| {
        let handles: Vec<_> = unique
            .iter()
            .map(|(host, port)| {
                let host = host.clone();
                let port = *port;
                scope.spawn(move || {
                    let addrs = resolve_host_timed(&host, port, dns_budget);
                    ((host, port), addrs)
                })
            })
            .collect();
        for h in handles {
            let _ = h.join();
        }
    });

    let resolved: Vec<ResolvedTarget> = targets
        .iter()
        .map(|target| {
            let mut addrs = resolve_host_timed(target.host, target.port, dns_budget);
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
        "timing dns_resolve({} hosts, {} unique): {:.0}ms",
        targets.len(),
        unique.len(),
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



fn probe_udp_quic(host: &str, port: u16, pre: &[SocketAddr], timeout: Duration) -> ProbeResult {
    use std::net::UdpSocket;

    let start = Instant::now();
    let resolved: Vec<SocketAddr> = if !pre.is_empty() {
        prefer_ipv4(pre.iter().map(|a| SocketAddr::new(a.ip(), port)))
    } else {
        resolve_host_timed(host, port, timeout.min(Duration::from_millis(1200)))
    };
    if resolved.is_empty() {
        return ProbeResult {
            ok: false,
            latency_ms: None,
            note: "QUIC DNS failed".into(),
            protocol: "quic",
            weight: 2,
        };
    }

    let Ok(sock) = UdpSocket::bind("0.0.0.0:0") else {
        return ProbeResult {
            ok: false,
            latency_ms: None,
            note: "QUIC bind failed".into(),
            protocol: "quic",
            weight: 2,
        };
    };
    let slice = timeout.min(Duration::from_millis(600));
    let _ = sock.set_read_timeout(Some(slice));
    let _ = sock.set_write_timeout(Some(slice));


    let mut req = [0u8; 64];
    req[0] = 0xc0;
    for (i, b) in req[1..].iter_mut().enumerate() {
        *b = (0x5a ^ (i as u8)).wrapping_add(start.elapsed().as_nanos() as u8);
    }

    for addr in &resolved {
        if sock.send_to(&req, addr).is_err() {
            continue;
        }
        let mut buf = [0u8; 256];
        match sock.recv_from(&mut buf) {
            Ok((n, _)) if n > 0 => {
                let ms = start.elapsed().as_millis().min(u32::MAX as u128) as u32;
                return ProbeResult {
                    ok: true,
                    latency_ms: Some(ms),
                    note: "QUIC/UDP reply".into(),
                    protocol: "quic",
                    weight: 2,
                };
            }
            _ => continue,
        }
    }

    ProbeResult {
        ok: false,
        latency_ms: None,
        note: "QUIC/UDP no reply (HTTP3 may fail)".into(),
        protocol: "quic",
        weight: 2,
    }
}


fn probe_udp_stun(host: &str, port: u16, pre: &[SocketAddr], timeout: Duration) -> ProbeResult {
    use std::net::UdpSocket;

    let start = Instant::now();
    let resolved: Vec<SocketAddr> = if !pre.is_empty() {
        prefer_ipv4(pre.iter().map(|a| SocketAddr::new(a.ip(), port)))
    } else {
        resolve_host_timed(host, port, timeout.min(Duration::from_millis(1200)))
    };
    if resolved.is_empty() {
        return ProbeResult {
            ok: false,
            latency_ms: None,
            note: "UDP DNS failed".into(),
            protocol: "udp",
            weight: 2,
        };
    }

    let Ok(sock) = UdpSocket::bind("0.0.0.0:0") else {
        return ProbeResult {
            ok: false,
            latency_ms: None,
            note: "UDP bind failed".into(),
            protocol: "udp",
            weight: 2,
        };
    };
    let slice = timeout.min(Duration::from_millis(600));
    let _ = sock.set_read_timeout(Some(slice));
    let _ = sock.set_write_timeout(Some(slice));


    let mut req = [0u8; 20];
    req[0] = 0x00;
    req[1] = 0x01;
    req[4] = 0x21;
    req[5] = 0x12;
    req[6] = 0xa4;
    req[7] = 0x42;
    for b in &mut req[8..] {
        *b = (start.elapsed().as_nanos() as u8).wrapping_add(*b);
    }

    for addr in &resolved {
        if sock.send_to(&req, addr).is_err() {
            continue;
        }
        let mut buf = [0u8; 128];
        match sock.recv_from(&mut buf) {
            Ok((n, _)) if n >= 20 && buf[0] == 0x01 && buf[1] == 0x01 => {
                let ms = start.elapsed().as_millis().min(u32::MAX as u128) as u32;
                return ProbeResult {
                    ok: true,
                    latency_ms: Some(ms),
                    note: "UDP/STUN OK".into(),
                    protocol: "udp",
                    weight: 2,
                };
            }
            _ => continue,
        }
    }

    ProbeResult {
        ok: false,
        latency_ms: None,
        note: "UDP/STUN no reply (voice may fail)".into(),
        protocol: "udp",
        weight: 2,
    }
}



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

    if target.quic {
        let udp_budget = timeout.min(Duration::from_millis(700));
        return vec![probe_udp_quic(target.host, target.port, addrs, udp_budget)];
    }

    if !target.https {

        if target.port == 3478 {
            let udp_budget = timeout.min(Duration::from_millis(700));
            return vec![probe_udp_stun(target.host, target.port, addrs, udp_budget)];
        }

        let ports: Vec<u16> = if target.port == 2053 {
            vec![2053, 2083, 8443]
        } else {
            vec![target.port]
        };
        let tcp_budget = timeout.min(Duration::from_millis(700));
        let tcp = probe_tcp_ports_parallel(target.host, &ports, addrs, tcp_budget);
        return vec![tcp];
    }


    let https_budget = if abort.is_some_and(|a| a.load(Ordering::Relaxed)) {
        timeout.min(Duration::from_millis(250))
    } else {
        timeout.min(Duration::from_millis(900))
    };
    vec![probe_https_addrs(target.host, addrs, https_budget)]
}


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
                            quic: false,
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

            let score = ((critical_floor * 75) + (soft_avg * 25)) / 100;
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
    rank_score_with_latency(probe_score, strategy_name, None)
}


pub fn rank_score_with_latency(
    probe_score: u32,
    strategy_name: &str,
    best_latency_ms: Option<u32>,
) -> u32 {
    rank_score_full(probe_score, strategy_name, best_latency_ms, 0)
}

pub fn rank_score_full(
    probe_score: u32,
    strategy_name: &str,
    best_latency_ms: Option<u32>,
    soft_score: u32,
) -> u32 {
    let mut adjusted = probe_score as i32 + strategy_prior_bonus(strategy_name);
    if let Some(ms) = best_latency_ms {
        adjusted += match ms {
            0..=80 => 4,
            81..=150 => 2,
            151..=300 => 0,
            301..=600 => -2,
            _ => -4,
        };
    }

    adjusted += (soft_score as i32 / 20).min(5);
    adjusted.clamp(0, 100) as u32
}


pub fn path_verdict_from_results(host_results: &[(DiagTarget, Vec<ProbeResult>)]) -> PathVerdict {
    let youtube_ok = host_results
        .iter()
        .filter(|(t, _)| t.critical && t.service == "youtube")
        .all(|(_, probes)| probes.iter().any(|p| p.ok));
    let discord_web_ok = host_results
        .iter()
        .filter(|(t, _)| {
            t.critical && t.service == "discord" && !t.quic && t.port != 3478
        })
        .all(|(_, probes)| probes.iter().any(|p| p.ok));
    let voice_probes: Vec<&ProbeResult> = host_results
        .iter()
        .filter(|(t, _)| {
            !t.critical
                && (t.port == 3478
                    || t.quic
                    || t.host.contains("stun")
                    || probes_are_udp(t, host_results))
        })
        .flat_map(|(_, probes)| probes.iter())
        .collect();

    let discord_voice = if voice_probes.is_empty() {
        AvailabilityStatus::Unknown
    } else if voice_probes.iter().any(|p| p.ok) {
        AvailabilityStatus::Available
    } else {
        AvailabilityStatus::Unavailable
    };

    PathVerdict {
        youtube: if youtube_ok {
            AvailabilityStatus::Available
        } else {
            AvailabilityStatus::Unavailable
        },
        discord_web: if discord_web_ok {
            AvailabilityStatus::Available
        } else {
            AvailabilityStatus::Unavailable
        },
        discord_voice,
    }
}

fn probes_are_udp(t: &DiagTarget, _all: &[(DiagTarget, Vec<ProbeResult>)]) -> bool {
    t.port == 3478 || t.quic
}


pub fn watchdog_targets() -> Vec<DiagTarget> {
    diag_targets()
        .into_iter()
        .filter(|t| {
            t.critical
                || t.port == 3478
                || (t.quic && (t.host.contains("discord") || t.host.contains("youtube")))
        })
        .collect()
}


pub fn soft_contribution(host_results: &[(DiagTarget, Vec<ProbeResult>)]) -> u32 {
    let soft: Vec<u32> = host_results
        .iter()
        .filter(|(t, _)| !t.critical)
        .map(|(_, probes)| score_probes(probes))
        .collect();
    if soft.is_empty() {
        0
    } else {
        soft.iter().sum::<u32>() / soft.len() as u32
    }
}

pub fn latency_stats(probes: &[ProbeResult]) -> (Option<u32>, Option<u32>, Option<u32>) {
    let mut vals: Vec<u32> = probes.iter().filter_map(|p| p.latency_ms).collect();
    if vals.is_empty() {
        return (None, None, None);
    }
    vals.sort_unstable();
    let min = vals.first().copied();
    let max = vals.last().copied();
    let mid = vals[vals.len() / 2];
    (min, Some(mid), max)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_probe(protocol: &'static str, weight: u32, ms: u32) -> ProbeResult {
        ProbeResult {
            ok: true,
            latency_ms: Some(ms),
            note: "ok".into(),
            protocol,
            weight,
        }
    }

    fn fail_probe(protocol: &'static str) -> ProbeResult {
        ProbeResult {
            ok: false,
            latency_ms: None,
            note: "fail".into(),
            protocol,
            weight: 1,
        }
    }

    #[test]
    fn score_by_service_requires_all_critical() {
        let yt = DiagTarget {
            service: "youtube",
            host: "youtube.com",
            port: 443,
            https: true,
            quic: false,
            critical: true,
        };
        let yt_cdn = DiagTarget {
            service: "youtube",
            host: "i.ytimg.com",
            port: 443,
            https: true,
            quic: false,
            critical: false,
        };
        let disc = DiagTarget {
            service: "discord",
            host: "discord.com",
            port: 443,
            https: true,
            quic: false,
            critical: true,
        };

        let pass = score_by_service(
            &[
                (yt, vec![ok_probe("https", 3, 80)]),
                (yt_cdn, vec![fail_probe("https")]),
                (disc, vec![ok_probe("https", 3, 90)]),
            ],
            55,
        );
        assert!(pass.success);
        assert!(pass.score < 100);

        let fail = score_by_service(
            &[
                (yt, vec![fail_probe("https")]),
                (yt_cdn, vec![ok_probe("https", 3, 50)]),
                (disc, vec![ok_probe("https", 3, 90)]),
            ],
            55,
        );
        assert!(!fail.success);
    }

    #[test]
    fn rank_score_full_soft_breaks_ties() {
        let a = rank_score_full(70, "general (FAKE TLS AUTO)", Some(100), 20);
        let b = rank_score_full(70, "general (FAKE TLS AUTO)", Some(100), 80);
        assert!(b > a);
    }
}
