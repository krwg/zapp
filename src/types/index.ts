// ╔════════════════════════════════════════════════════╗
// ║                   types/index                    ║
// ╚════════════════════════════════════════════════════╝

export type ThemeMode = 'light' | 'dark' | 'system'

export type EngineStatus = 'running' | 'stopped' | 'starting' | 'stopping' | 'error'

export type AvailabilityStatus = 'available' | 'unavailable' | 'checking' | 'unknown'

export type UpdateChannelStatus =
  | 'up_to_date'
  | 'available'
  | 'downloading'
  | 'installing'
  | 'error'

export type DiagnosticPhase =
  | 'idle'
  | 'preparing'
  | 'testing'
  | 'analyzing'
  | 'completed'
  | 'failed'
  | 'cancelled'

export type StrategySource = 'bundled' | 'remote' | 'auto' | 'user' | 'imported'

export type ProfileKind = 'general' | 'youtube' | 'discord' | 'custom' | 'auto'

export interface ConnectionState {
  status: EngineStatus
  strategyName: string | null
  strategyId: string | null
  sessionStartedAt: string | null
  uptimeSeconds: number
  lastError: string | null
}

export interface Profile {
  id: string
  name: string
  kind: ProfileKind
  strategyType: string
  source: StrategySource
  description: string
  isActive: boolean
  isAutoSelected: boolean
  lastCheckedAt: string | null
  lastUpdatedAt: string
  tags: string[]
}

export interface ServiceCheckHistoryItem {
  id: string
  checkedAt: string
  status: AvailabilityStatus
  latencyMs: number | null
  note: string | null
}

export interface MonitoredService {
  id: string
  name: string
  host: string
  status: AvailabilityStatus
  lastCheckedAt: string | null
  latencyMs: number | null
  diagnosticSummary: string
  history: ServiceCheckHistoryItem[]
  isBuiltIn: boolean
}

export interface SystemInfo {
  appVersion: string
  engineVersion: string
  strategiesVersion: string
  lastUpdateCheckAt: string | null
  autoUpdateEnabled: boolean
  elevated: boolean
}

export interface DiagnosticHostDetail {
  host: string
  service: string
  ok: boolean
  latencyMs: number | null
  note: string
  critical: boolean
}

export interface DiagnosticServiceScore {
  service: string
  score: number
  ok: boolean
  latencyMs: number | null
}

export interface DiagnosticStrategyResult {
  id: string
  name: string
  strategyType: string
  success: boolean
  latencyMs: number | null
  latencyMedianMs: number | null
  latencyMaxMs: number | null
  score: number
  softScore: number
  services: DiagnosticServiceScore[]
  hosts: DiagnosticHostDetail[]
  notes: string
  testedAt: string
}

export interface DiagnosticRun {
  id: string
  phase: DiagnosticPhase
  progress: number
  currentStrategy: string | null
  testedCount: number
  totalCount: number
  workingCount: number
  startedAt: string | null
  finishedAt: string | null
  results: DiagnosticStrategyResult[]
  isDemo: boolean
}

export interface UpdateChannel {
  id: 'application' | 'engine' | 'strategies' | 'telegramProxy'
  title: string
  description: string
  currentVersion: string
  latestVersion: string
  status: UpdateChannelStatus
  lastCheckedAt: string | null
  downloadProgress: number | null
  releaseNotes: string
  independent: true
}

export type DiagnosticsMode = 'fast' | 'thorough'

export interface TelegramProxyState {
  status: EngineStatus
  installed: boolean
  version: string
  host: string
  port: number
  sessionStartedAt: string | null
  uptimeSeconds: number
  lastError: string | null
  proxyLink: string
}

export interface AppSettings {
  appearance: ThemeMode
  startWithSystem: boolean
  minimizeToTray: boolean
  autoCheckUpdates: boolean
  autoInstallUpdates: boolean
  autoDiagnostics: boolean
  diagnosticsIntervalHours: number
  diagnosticsMode: DiagnosticsMode
  networkTimeoutMs: number
  networkRetries: number
  advancedLogging: boolean
  enginePath: string | null
  strategiesPath: string | null
  telegramProxyEnabled: boolean
  telegramOfferLinkOnStart: boolean
}

export interface AppAbout {
  name: string
  version: string
  license: string
  authors: string[]
  githubUrl: string
  homepageUrl: string
}

export interface AppSnapshot {
  connection: ConnectionState
  telegram: TelegramProxyState
  activeProfile: Profile | null
  profiles: Profile[]
  services: MonitoredService[]
  system: SystemInfo
  diagnostics: DiagnosticRun
  updates: UpdateChannel[]
  settings: AppSettings
  about: AppAbout
}
