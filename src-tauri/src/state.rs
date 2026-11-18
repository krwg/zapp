// ╔════════════════════════════════════════════════════╗
// ║                      state                       ║
// ╚════════════════════════════════════════════════════╝

use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use tauri::{AppHandle, Manager};

use crate::diagnostics::run_diagnostics_worker;
use crate::engine::{
    is_elevated, probe_tcp, resolve_engine_layout, scan_strategy_profiles, ManagedEngine,
    ManagedTgProxy, UpdateManager,
};
use crate::error::{AppError, AppResult};
use crate::history::HistoryLog;
use crate::model::*;
use crate::util::{new_id, now_iso};

pub struct AppState {
    pub engine: ManagedEngine,
    pub tg_proxy: ManagedTgProxy,
    pub updates: UpdateManager,
    pub history: HistoryLog,
    pub(crate) inner: Mutex<Store>,
    diag_cancel: Mutex<Option<Arc<Mutex<bool>>>>,
    diag_worker: Mutex<Option<JoinHandle<()>>>,
    /// Latest prefer_previous requested while a run was still winding down.
    diag_pending: Mutex<Option<bool>>,
    /// Serializes start / queue / drain so runs are not silently dropped.
    diag_gate: Mutex<()>,
}

pub(crate) struct Store {
    pub profiles: Vec<Profile>,
    pub services: Vec<MonitoredService>,
    pub diagnostics: DiagnosticRun,
    pub settings: AppSettings,
    pub about: AppAbout,
    pub system: SystemInfo,
}

impl AppState {
    pub fn new(app_data: std::path::PathBuf) -> Self {
        let layout = resolve_engine_layout(&app_data);
        let engine_version = layout.version.clone();
        let mut profiles = scan_strategy_profiles(&layout.root);
        if profiles.is_empty() {
            profiles = seed_profiles();
        }
        let engine = ManagedEngine::new(app_data.clone(), layout);
        if let Some(active) = profiles.iter().find(|p| p.is_active) {
            engine.set_active_strategy(active.name.clone());
        }
        let tg_proxy = ManagedTgProxy::new(app_data.clone());
        Self::assemble(app_data, engine, tg_proxy, engine_version, profiles)
    }

    fn assemble(
        app_data: std::path::PathBuf,
        engine: ManagedEngine,
        tg_proxy: ManagedTgProxy,
        engine_version: String,
        profiles: Vec<Profile>,
    ) -> Self {
        let tg_version = tg_proxy.version();
        let updates = UpdateManager::new(
            seed_updates(&engine_version, &tg_version),
            app_data.clone(),
        );
        Self {
            engine,
            tg_proxy,
            updates,
            history: HistoryLog::open(app_data),
            diag_cancel: Mutex::new(None),
            diag_worker: Mutex::new(None),
            diag_pending: Mutex::new(None),
            diag_gate: Mutex::new(()),
            inner: Mutex::new(Store {
                profiles,
                services: seed_services(),
                diagnostics: seed_diagnostics(),
                settings: seed_settings(),
                about: seed_about(),
                system: SystemInfo {
                    app_version: env!("CARGO_PKG_VERSION").into(),
                    engine_version: engine_version.clone(),
                    strategies_version: engine_version,
                    last_update_check_at: None,
                    auto_update_enabled: true,
                    elevated: is_elevated(),
                },
            }),
        }
    }

    pub fn snapshot(&self) -> AppSnapshot {
        let status = self.engine.status();
        if !matches!(
            status,
            EngineStatus::Starting | EngineStatus::Stopping
        ) {
            self.engine.refresh_status_from_os();
        }

        let mut store = self.inner.lock().expect("state lock");

        if !store.system.elevated {
            store.system.elevated = is_elevated();
        }
        let active = store.profiles.iter().find(|p| p.is_active).cloned();
        AppSnapshot {
            connection: ConnectionState {
                status: self.engine.status(),
                strategy_name: active
                    .as_ref()
                    .map(|p| p.name.clone())
                    .or_else(|| self.engine.active_strategy()),
                strategy_id: active.as_ref().map(|p| p.id.clone()),
                session_started_at: self.engine.session_started_at(),
                uptime_seconds: self.engine.uptime_seconds(),
                last_error: self.engine.last_error(),
            },
            telegram: self.tg_proxy.snapshot_state(),
            active_profile: active,
            profiles: store.profiles.clone(),
            services: store.services.clone(),
            system: store.system.clone(),
            diagnostics: store.diagnostics.clone(),
            updates: self.updates.list(),
            settings: store.settings.clone(),
            about: store.about.clone(),
        }
    }

    pub fn diagnostics(&self) -> DiagnosticRun {
        self.inner.lock().expect("state lock").diagnostics.clone()
    }

    pub fn set_active_profile(&self, profile_id: &str) -> AppResult<Profile> {
        let mut store = self.inner.lock().expect("state lock");
        if !store.profiles.iter().any(|p| p.id == profile_id) {
            return Err(AppError::from("Profile not found"));
        }
        for profile in store.profiles.iter_mut() {
            profile.is_active = profile.id == profile_id;
        }
        let active = store
            .profiles
            .iter()
            .find(|p| p.id == profile_id)
            .cloned()
            .ok_or_else(|| AppError::from("Profile not found"))?;
        self.engine.set_active_strategy(active.name.clone());
        self.history
            .push("profile", format!("Activated profile '{}'", active.name));
        
        if matches!(self.engine.status(), EngineStatus::Running) {
            let _ = self.engine.start(Some(&active.name));
        }
        Ok(active)
    }

    pub fn start_diagnostics(&self, app: AppHandle) -> AppResult<DiagnosticRun> {
        // Manual Smart check: do not prefer the previous strategy (UDP issues are invisible to probes).
        self.start_diagnostics_with(app, false)
    }

    /// Background re-check may pass `prefer_previous = true` to verify the active strategy first.
    pub fn start_diagnostics_with(
        &self,
        app: AppHandle,
        prefer_previous: bool,
    ) -> AppResult<DiagnosticRun> {
        let _gate = self.diag_gate.lock().expect("diag gate");
        self.cancel_diagnostics_flag();

        // If a worker is still alive (running or cancelling), queue the latest request
        // instead of spawning a second joiner chain that can silently drop a start.
        {
            let worker = self.diag_worker.lock().expect("worker lock");
            if worker.as_ref().is_some_and(|h| !h.is_finished()) {
                *self.diag_pending.lock().expect("pending lock") = Some(prefer_previous);
                let mut store = self.inner.lock().expect("state lock");
                if matches!(
                    store.diagnostics.phase,
                    DiagnosticPhase::Preparing
                        | DiagnosticPhase::Testing
                        | DiagnosticPhase::Analyzing
                ) {
                    store.diagnostics.current_strategy =
                        Some("Busy — will restart when ready…".into());
                }
                self.history.push(
                    "diagnostics",
                    "Smart check busy — queued a restart after the current run ends",
                );
                return Ok(store.diagnostics.clone());
            }
        }

        // Reap a finished handle (join is instant once is_finished).
        if let Some(done) = self.diag_worker.lock().expect("worker lock").take() {
            let _ = done.join();
        }
        *self.diag_pending.lock().expect("pending lock") = None;

        self.spawn_diagnostics_locked(app, prefer_previous)
    }

    fn spawn_diagnostics_locked(
        &self,
        app: AppHandle,
        prefer_previous: bool,
    ) -> AppResult<DiagnosticRun> {
        let cancel = Arc::new(Mutex::new(false));
        *self.diag_cancel.lock().expect("cancel lock") = Some(cancel.clone());

        let run_id = new_id("diag");
        let total = {
            let store = self.inner.lock().expect("state lock");
            store.profiles.len() as u32
        };

        {
            let mut store = self.inner.lock().expect("state lock");
            store.diagnostics = DiagnosticRun {
                id: run_id.clone(),
                phase: DiagnosticPhase::Preparing,
                progress: 0,
                current_strategy: Some("Preparing…".into()),
                tested_count: 0,
                total_count: total,
                working_count: 0,
                started_at: Some(now_iso()),
                finished_at: None,
                results: vec![],
                is_demo: false,
            };
        }

        let handle = thread::spawn(move || {
            run_diagnostics_worker(app.clone(), run_id, cancel, prefer_previous);
            // Drain queued restart without holding the gate across the worker body.
            let app_for_pending = app.clone();
            if let Some(state) = app.try_state::<AppState>() {
                let pending = {
                    let _gate = state.diag_gate.lock().expect("diag gate");
                    let _ = state.diag_worker.lock().expect("worker lock").take();
                    state.diag_pending.lock().expect("pending lock").take()
                };
                if let Some(prefer) = pending {
                    let _ = state.start_diagnostics_with(app_for_pending, prefer);
                }
            }
        });
        *self.diag_worker.lock().expect("worker lock") = Some(handle);

        Ok(self.diagnostics())
    }

    pub fn cancel_diagnostics(&self) -> AppResult<DiagnosticRun> {
        // Flip cancel and drop any queued restart — Cancel must not auto-start again.
        self.cancel_diagnostics_flag();
        *self
            .diag_pending
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = None;
        let mut store = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        if matches!(
            store.diagnostics.phase,
            DiagnosticPhase::Preparing
                | DiagnosticPhase::Testing
                | DiagnosticPhase::Analyzing
        ) {
            store.diagnostics.current_strategy = Some("Cancelling…".into());
        }
        Ok(store.diagnostics.clone())
    }

    fn cancel_diagnostics_flag(&self) {
        if let Some(flag) = self
            .diag_cancel
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
        {
            *flag.lock().unwrap_or_else(|e| e.into_inner()) = true;
        }
    }

    pub fn select_diagnostic_strategy(&self, result_id: &str) -> AppResult<Profile> {
        let mut store = self.inner.lock().expect("state lock");
        let result = store
            .diagnostics
            .results
            .iter()
            .find(|r| r.id == result_id)
            .cloned()
            .ok_or_else(|| AppError::from("Strategy unavailable"))?;
        if !result.success {
            return Err(AppError::from("Strategy unavailable"));
        }

        for profile in store.profiles.iter_mut() {
            profile.is_active = false;
        }

        let selected_name = result.name.clone();
        if let Some(existing) = store.profiles.iter_mut().find(|p| p.name == result.name) {
            existing.is_active = true;
            existing.is_auto_selected = true;
            existing.last_checked_at = Some(now_iso());
            let out = existing.clone();
            drop(store);
            self.engine.set_active_strategy(selected_name.clone());
            let _ = self.engine.start(Some(&selected_name));
            self.history.push(
                "profile",
                format!("Applied diagnostic strategy '{selected_name}'"),
            );
            return Ok(out);
        }

        let profile = Profile {
            id: new_id("profile-from"),
            name: result.name,
            kind: ProfileKind::Auto,
            strategy_type: result.strategy_type,
            source: StrategySource::Auto,
            description: "Selected from diagnostics.".into(),
            is_active: true,
            is_auto_selected: true,
            last_checked_at: Some(now_iso()),
            last_updated_at: now_iso(),
            tags: vec!["auto".into(), "diagnostics".into()],
        };
        store.profiles.insert(0, profile.clone());
        drop(store);
        self.engine.set_active_strategy(selected_name.clone());
        let _ = self.engine.start(Some(&selected_name));
        Ok(profile)
    }

    pub fn check_service(&self, service_id: &str) -> AppResult<MonitoredService> {
        let (host, timeout_ms, is_telegram) = {
            let mut store = self.inner.lock().expect("state lock");
            let service = store
                .services
                .iter_mut()
                .find(|s| s.id == service_id)
                .ok_or_else(|| AppError::from("Service not found"))?;
            service.status = AvailabilityStatus::Checking;
            let is_tg = service.id == "svc-telegram";
            (service.host.clone(), store.settings.network_timeout_ms, is_tg)
        };

        let probe = if is_telegram {
            let port = self.tg_proxy.port();
            let host = self.tg_proxy.host();
            probe_tcp(&host, port, Duration::from_millis(timeout_ms.max(800) as u64))
        } else {
            probe_tcp(&host, 443, Duration::from_millis(timeout_ms.max(1000) as u64))
        };

        let mut store = self.inner.lock().expect("state lock");
        let service = store
            .services
            .iter_mut()
            .find(|s| s.id == service_id)
            .ok_or_else(|| AppError::from("Service not found"))?;

        service.status = probe.status();
        service.latency_ms = probe.latency_ms;
        service.last_checked_at = Some(now_iso());
        service.diagnostic_summary = if is_telegram {
            if probe.ok {
                format!("Local Telegram proxy listening. {}", probe.note)
            } else {
                format!("Local Telegram proxy not listening. {}", probe.note)
            }
        } else if probe.ok {
            format!("Available. {}", probe.note)
        } else {
            format!("Unavailable. {}", probe.note)
        };
        service.history.insert(
            0,
            ServiceCheckHistoryItem {
                id: new_id("chk"),
                checked_at: service.last_checked_at.clone().unwrap_or_default(),
                status: service.status.clone(),
                latency_ms: service.latency_ms,
                note: Some(probe.note),
            },
        );
        service.history.truncate(12);
        Ok(service.clone())
    }

    pub fn check_all_services(&self) -> AppResult<Vec<MonitoredService>> {
        let tg_host = self.tg_proxy.host();
        let tg_port = self.tg_proxy.port();
        let jobs: Vec<(String, String, u16, u64, bool)> = {
            let mut store = self.inner.lock().expect("state lock");
            let timeout_ms = store.settings.network_timeout_ms.max(1000) as u64;
            store
                .services
                .iter_mut()
                .map(|s| {
                    s.status = AvailabilityStatus::Checking;
                    let is_tg = s.id == "svc-telegram";
                    let (host, port) = if is_tg {
                        (tg_host.clone(), tg_port)
                    } else {
                        (s.host.clone(), 443)
                    };
                    (s.id.clone(), host, port, timeout_ms, is_tg)
                })
                .collect()
        };

        let probes: Vec<(String, crate::engine::ProbeResult, bool)> = thread::scope(|scope| {
            let mut handles = Vec::with_capacity(jobs.len());
            for (id, host, port, timeout_ms, is_tg) in &jobs {
                handles.push(scope.spawn(move || {
                    let probe = probe_tcp(host, *port, Duration::from_millis(*timeout_ms));
                    (id.clone(), probe, *is_tg)
                }));
            }
            handles
                .into_iter()
                .filter_map(|h| h.join().ok())
                .collect()
        });

        let mut out = Vec::with_capacity(probes.len());
        let mut store = self.inner.lock().expect("state lock");
        for (id, probe, is_tg) in probes {
            let Some(service) = store.services.iter_mut().find(|s| s.id == id) else {
                continue;
            };
            service.status = probe.status();
            service.latency_ms = probe.latency_ms;
            service.last_checked_at = Some(now_iso());
            service.diagnostic_summary = if is_tg {
                if probe.ok {
                    format!("Local Telegram proxy listening. {}", probe.note)
                } else {
                    format!("Local Telegram proxy not listening. {}", probe.note)
                }
            } else if probe.ok {
                format!("Available. {}", probe.note)
            } else {
                format!("Unavailable. {}", probe.note)
            };
            service.history.insert(
                0,
                ServiceCheckHistoryItem {
                    id: new_id("chk"),
                    checked_at: service.last_checked_at.clone().unwrap_or_default(),
                    status: service.status.clone(),
                    latency_ms: service.latency_ms,
                    note: Some(probe.note),
                },
            );
            service.history.truncate(12);
            out.push(service.clone());
        }
        Ok(out)
    }

    pub fn check_updates(&self) -> AppResult<Vec<UpdateChannel>> {
        let list = self.updates.check_all()?;
        let mut store = self.inner.lock().expect("state lock");
        store.system.last_update_check_at = Some(now_iso());
        Ok(list)
    }

    /// Check GitHub releases; when `auto_install` is on, install Available channels.
    /// Returns whether the app must quit (Application installer launched).
    pub fn check_updates_and_auto_install(&self, auto_install: bool) -> AppResult<bool> {
        let _ = self.bootstrap_engine_if_needed()?;
        let _ = self.check_updates()?;
        if !auto_install {
            return Ok(false);
        }
        let outcomes = self.updates.auto_install_available()?;
        let mut restart = false;
        for outcome in outcomes {
            self.apply_update_outcome(&outcome)?;
            if outcome.restart_required {
                restart = true;
            }
        }
        Ok(restart)
    }

    /// Download Flowseal into app data when the pack is missing/incomplete.
    /// Also creates Flowseal user-list sidecars the BATs expect.
    pub fn bootstrap_engine_if_needed(&self) -> AppResult<bool> {
        let layout = self.engine.layout();
        let _ = crate::engine::ensure_flowseal_sidecars(&layout.root);
        match self.updates.ensure_engine_installed(&layout)? {
            Some(new_layout) => {
                let outcome = crate::engine::UpdateOutcome {
                    channel: self
                        .updates
                        .list()
                        .into_iter()
                        .find(|c| c.id == UpdateChannelId::Engine)
                        .ok_or_else(|| AppError::from("Engine channel missing"))?,
                    layout: Some(new_layout),
                    restart_required: false,
                };
                self.apply_update_outcome(&outcome)?;
                self.history.push("engine", "Flowseal engine pack installed");
                Ok(true)
            }
            None => Ok(false),
        }
    }

    pub fn update_channel(&self, id: UpdateChannelId) -> AppResult<(UpdateChannel, bool)> {
        let outcome = self.updates.update_channel(id)?;
        self.apply_update_outcome(&outcome)?;
        Ok((outcome.channel, outcome.restart_required))
    }

    fn apply_update_outcome(
        &self,
        outcome: &crate::engine::UpdateOutcome,
    ) -> AppResult<()> {
        let mut store = self.inner.lock().expect("state lock");
        match outcome.channel.id {
            UpdateChannelId::Application => {
                // Installer applies the new version; keep displayed target in notes/latest.
                store.system.app_version = outcome.channel.current_version.clone();
                store.about.version = outcome.channel.current_version.clone();
            }
            UpdateChannelId::Engine | UpdateChannelId::Strategies => {
                store.system.engine_version = outcome.channel.current_version.clone();
                store.system.strategies_version = outcome.channel.current_version.clone();
                if let Some(layout) = outcome.layout.clone() {
                    let profiles = scan_strategy_profiles(&layout.root);
                    if !profiles.is_empty() {
                        store.profiles = profiles;
                    }
                    drop(store);
                    self.engine.adopt_layout(layout)?;
                    return Ok(());
                }
            }
            UpdateChannelId::TelegramProxy => {
                drop(store);
                self.tg_proxy.adopt_install(&outcome.channel.current_version);
            }
        }
        Ok(())
    }

    pub fn shutdown_for_exit(&self) {
        self.engine.shutdown_for_exit();
        self.tg_proxy.shutdown_for_exit();
    }

    pub fn update_settings(&self, patch: SettingsPatch) -> AppSettings {
        let mut store = self.inner.lock().expect("state lock");
        if let Some(v) = patch.appearance {
            store.settings.appearance = v;
        }
        if let Some(v) = patch.start_with_system {
            store.settings.start_with_system = v;
        }
        if let Some(v) = patch.minimize_to_tray {
            store.settings.minimize_to_tray = v;
        }
        if let Some(v) = patch.auto_check_updates {
            store.settings.auto_check_updates = v;
        }
        if let Some(v) = patch.auto_install_updates {
            store.settings.auto_install_updates = v;
        }
        if let Some(v) = patch.auto_diagnostics {
            store.settings.auto_diagnostics = v;
        }
        if let Some(v) = patch.diagnostics_interval_hours {
            store.settings.diagnostics_interval_hours = v;
        }
        if let Some(v) = patch.diagnostics_mode {
            store.settings.diagnostics_mode = v;
        }
        if let Some(v) = patch.network_timeout_ms {
            store.settings.network_timeout_ms = v;
        }
        if let Some(v) = patch.network_retries {
            store.settings.network_retries = v;
        }
        if let Some(v) = patch.advanced_logging {
            store.settings.advanced_logging = v;
        }
        if let Some(v) = patch.engine_path {
            store.settings.engine_path = v;
        }
        if let Some(v) = patch.strategies_path {
            store.settings.strategies_path = v;
        }
        if let Some(v) = patch.telegram_proxy_enabled {
            store.settings.telegram_proxy_enabled = v;
        }
        if let Some(v) = patch.telegram_offer_link_on_start {
            store.settings.telegram_offer_link_on_start = v;
        }
        store.system.auto_update_enabled = store.settings.auto_check_updates;
        store.settings.clone()
    }

    pub fn set_theme(&self, mode: ThemeMode) -> ThemeMode {
        let mut store = self.inner.lock().expect("state lock");
        store.settings.appearance = mode.clone();
        mode
    }
}

fn seed_profiles() -> Vec<Profile> {
    vec![
        Profile {
            id: "profile-auto-01".into(),
            name: "FAKE TLS AUTO".into(),
            kind: ProfileKind::Auto,
            strategy_type: "fake + multisplit".into(),
            source: StrategySource::Auto,
            description: "Автоматически подобранная стратегия по результатам диагностики.".into(),
            is_active: true,
            is_auto_selected: true,
            last_checked_at: Some("2026-11-12T09:40:00.000Z".into()),
            last_updated_at: "2026-11-11T18:20:00.000Z".into(),
            tags: vec!["auto".into(), "stable".into()],
        },
        Profile {
            id: "profile-general-01".into(),
            name: "General".into(),
            kind: ProfileKind::General,
            strategy_type: "nfqws / winws".into(),
            source: StrategySource::Bundled,
            description: "Базовая конфигурация для широкого набора сервисов.".into(),
            is_active: false,
            is_auto_selected: false,
            last_checked_at: Some("2026-11-10T14:00:00.000Z".into()),
            last_updated_at: "2026-11-08T12:00:00.000Z".into(),
            tags: vec!["bundled".into()],
        },
        Profile {
            id: "profile-yt-01".into(),
            name: "YouTube Optimized".into(),
            kind: ProfileKind::Youtube,
            strategy_type: "fake tls + disorder".into(),
            source: StrategySource::Remote,
            description: "Профиль, ориентированный на стабильность YouTube.".into(),
            is_active: false,
            is_auto_selected: false,
            last_checked_at: Some("2026-11-12T07:15:00.000Z".into()),
            last_updated_at: "2026-11-09T09:30:00.000Z".into(),
            tags: vec!["youtube".into()],
        },
        Profile {
            id: "profile-discord-01".into(),
            name: "Discord Voice".into(),
            kind: ProfileKind::Discord,
            strategy_type: "split + dpi desync".into(),
            source: StrategySource::Bundled,
            description: "Конфигурация для Discord, включая голосовые каналы.".into(),
            is_active: false,
            is_auto_selected: false,
            last_checked_at: Some("2026-11-11T21:00:00.000Z".into()),
            last_updated_at: "2026-11-07T16:45:00.000Z".into(),
            tags: vec!["discord".into()],
        },
        Profile {
            id: "profile-user-01".into(),
            name: "My Custom Profile".into(),
            kind: ProfileKind::Custom,
            strategy_type: "user defined".into(),
            source: StrategySource::User,
            description: "Пользовательский профиль с ручными параметрами.".into(),
            is_active: false,
            is_auto_selected: false,
            last_checked_at: None,
            last_updated_at: "2026-11-05T11:10:00.000Z".into(),
            tags: vec!["custom".into()],
        },
    ]
}

fn seed_services() -> Vec<MonitoredService> {
    vec![
        MonitoredService {
            id: "svc-youtube".into(),
            name: "YouTube".into(),
            host: "youtube.com".into(),
            status: AvailabilityStatus::Available,
            last_checked_at: Some("2026-11-12T09:55:00.000Z".into()),
            latency_ms: Some(86),
            diagnostic_summary: "Доступен. TLS handshake успешен.".into(),
            is_built_in: true,
            history: vec![ServiceCheckHistoryItem {
                id: "h1".into(),
                checked_at: "2026-11-12T09:55:00.000Z".into(),
                status: AvailabilityStatus::Available,
                latency_ms: Some(86),
                note: Some("OK".into()),
            }],
        },
        MonitoredService {
            id: "svc-discord".into(),
            name: "Discord".into(),
            host: "gateway.discord.gg".into(),
            status: AvailabilityStatus::Available,
            last_checked_at: Some("2026-11-12T09:55:00.000Z".into()),
            latency_ms: Some(112),
            diagnostic_summary: "Доступен. Gateway отвечает.".into(),
            is_built_in: true,
            history: vec![ServiceCheckHistoryItem {
                id: "d1".into(),
                checked_at: "2026-11-12T09:55:00.000Z".into(),
                status: AvailabilityStatus::Available,
                latency_ms: Some(112),
                note: Some("OK".into()),
            }],
        },
        MonitoredService {
            id: "svc-github".into(),
            name: "GitHub".into(),
            host: "github.com".into(),
            status: AvailabilityStatus::Unavailable,
            last_checked_at: Some("2026-11-12T09:54:00.000Z".into()),
            latency_ms: None,
            diagnostic_summary: "Недоступен. Требуется повторная диагностика.".into(),
            is_built_in: false,
            history: vec![ServiceCheckHistoryItem {
                id: "g1".into(),
                checked_at: "2026-11-12T09:54:00.000Z".into(),
                status: AvailabilityStatus::Unavailable,
                latency_ms: None,
                note: Some("Connection reset".into()),
            }],
        },
        MonitoredService {
            id: "svc-telegram".into(),
            name: "Telegram".into(),
            host: "127.0.0.1".into(),
            status: AvailabilityStatus::Unknown,
            last_checked_at: None,
            latency_ms: None,
            diagnostic_summary: "Локальный MTProto-прокси (tg-ws-proxy). Не часть DPI Bypass.".into(),
            is_built_in: true,
            history: vec![],
        },
    ]
}

fn seed_diagnostics() -> DiagnosticRun {
    DiagnosticRun {
        id: "diag-idle".into(),
        phase: DiagnosticPhase::Idle,
        progress: 0,
        current_strategy: None,
        tested_count: 0,
        total_count: 0,
        working_count: 0,
        started_at: None,
        finished_at: None,
        is_demo: false,
        results: vec![],
    }
}

fn seed_updates(engine_version: &str, tg_version: &str) -> Vec<UpdateChannel> {
    vec![
        UpdateChannel {
            id: UpdateChannelId::Application,
            title: "Application".into(),
            description: "Клиент zapp. Независим от версии движка zapret.".into(),
            current_version: env!("CARGO_PKG_VERSION").into(),
            latest_version: env!("CARGO_PKG_VERSION").into(),
            status: UpdateChannelStatus::UpToDate,
            last_checked_at: None,
            download_progress: None,
            release_notes: "Источник: https://github.com/krwg/zapp/releases".into(),
            independent: true,
        },
        UpdateChannel {
            id: UpdateChannelId::Engine,
            title: "Engine".into(),
            description: "Пакет Flowseal/zapret-discord-youtube (winws). Auto-download с GitHub Releases.".into(),
            current_version: engine_version.into(),
            latest_version: engine_version.into(),
            status: UpdateChannelStatus::UpToDate,
            last_checked_at: None,
            download_progress: None,
            release_notes: "Источник: https://github.com/Flowseal/zapret-discord-youtube".into(),
            independent: true,
        },
        UpdateChannel {
            id: UpdateChannelId::Strategies,
            title: "Strategies".into(),
            description: "Стратегии и списки из Flowseal-пакета (общий релиз с Engine).".into(),
            current_version: engine_version.into(),
            latest_version: engine_version.into(),
            status: UpdateChannelStatus::UpToDate,
            last_checked_at: None,
            download_progress: None,
            release_notes: "Списки обновляются вместе с релизом Flowseal.".into(),
            independent: true,
        },
        UpdateChannel {
            id: UpdateChannelId::TelegramProxy,
            title: "Telegram proxy".into(),
            description: "Flowseal tg-ws-proxy (локальный MTProto → WebSocket). Отдельно от DPI Bypass.".into(),
            current_version: tg_version.into(),
            latest_version: tg_version.into(),
            status: UpdateChannelStatus::UpToDate,
            last_checked_at: None,
            download_progress: None,
            release_notes: "Источник: https://github.com/Flowseal/tg-ws-proxy".into(),
            independent: true,
        },
    ]
}

fn seed_settings() -> AppSettings {
    AppSettings {
        appearance: ThemeMode::System,
        start_with_system: true,
        minimize_to_tray: true,
        auto_check_updates: true,
        auto_install_updates: true,
        auto_diagnostics: true,
        diagnostics_interval_hours: 24,
        diagnostics_mode: DiagnosticsMode::Fast,
        network_timeout_ms: 5000,
        network_retries: 2,
        advanced_logging: false,
        engine_path: None,
        strategies_path: None,
        telegram_proxy_enabled: false,
        telegram_offer_link_on_start: true,
    }
}

fn seed_about() -> AppAbout {
    AppAbout {
        name: "zapp".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        license: "MIT".into(),
        authors: vec!["krwg".into()],
        github_url: "https://github.com/krwg/zapp".into(),
        homepage_url: "https://github.com/krwg/zapp".into(),
    }
}
