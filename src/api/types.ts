// ╔════════════════════════════════════════════════════╗
// ║                    api/types                     ║
// ╚════════════════════════════════════════════════════╝

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

export type HappZapretApi = ZappApi

export interface ZappApi {
  getSnapshot(): Promise<AppSnapshot>
  getDiagnostics(): Promise<DiagnosticRun>
  getUpdates(): Promise<UpdateChannel[]>
  setEngineRunning(running: boolean): Promise<EngineStatus>
  resetEngine(): Promise<EngineStatus>
  setActiveProfile(profileId: string): Promise<Profile>
  startDiagnostics(): Promise<DiagnosticRun>
  cancelDiagnostics(): Promise<DiagnosticRun>
  selectDiagnosticStrategy(resultId: string): Promise<Profile>
  checkService(serviceId: string): Promise<MonitoredService>
  checkAllServices(): Promise<MonitoredService[]>
  checkUpdates(): Promise<UpdateChannel[]>
  updateChannel(channelId: UpdateChannel['id']): Promise<UpdateChannel>
  updateSettings(patch: Partial<AppSettings>): Promise<AppSettings>
  setTheme(mode: ThemeMode): Promise<ThemeMode>
}
