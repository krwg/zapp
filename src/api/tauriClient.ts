// ╔════════════════════════════════════════════════════╗
// ║                 api/tauriClient                  ║
// ╚════════════════════════════════════════════════════╝

import { invoke } from '@tauri-apps/api/core'
import type {
  AppSettings,
  AppSnapshot,
  DiagnosticRun,
  EngineStatus,
  MonitoredService,
  Profile,
  ThemeMode,
  UpdateChannel,
} from '@/types'
import type { BugReportBundle } from '@/types/bugReport'
import type { ZappApi } from './types'

export function createTauriApi(): ZappApi {
  return {
    getSnapshot: () => invoke<AppSnapshot>('get_snapshot'),
    getDiagnostics: () => invoke<DiagnosticRun>('get_diagnostics'),
    getUpdates: () => invoke<UpdateChannel[]>('get_updates'),
    setEngineRunning: (running) =>
      invoke<EngineStatus>('set_engine_running', { running }),
    resetEngine: () => invoke<EngineStatus>('reset_engine'),
    setActiveProfile: (profileId) =>
      invoke<Profile>('set_active_profile', { profileId }),
    startDiagnostics: () => invoke<DiagnosticRun>('start_diagnostics'),
    cancelDiagnostics: () => invoke<DiagnosticRun>('cancel_diagnostics'),
    selectDiagnosticStrategy: (resultId) =>
      invoke<Profile>('select_diagnostic_strategy', { resultId }),
    checkService: (serviceId) =>
      invoke<MonitoredService>('check_service', { serviceId }),
    checkAllServices: () => invoke<MonitoredService[]>('check_all_services'),
    checkUpdates: () => invoke<UpdateChannel[]>('check_updates'),
    updateChannel: (channelId) =>
      invoke<UpdateChannel>('update_channel', { channelId }),
    updateSettings: (patch) => invoke<AppSettings>('update_settings', { patch }),
    setTheme: (mode) => invoke<ThemeMode>('set_theme', { mode }),
    openExternalUrl: (url) => invoke<void>('open_external_url', { url }),
    getRecentErrors: () => invoke<string[]>('get_recent_errors'),
    getBugReportBundle: () => invoke<BugReportBundle>('get_bug_report_bundle'),
    importEnginePack: () =>
      invoke<UpdateChannel>('import_engine_pack'),
  }
}

export function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}
