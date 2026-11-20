// ╔════════════════════════════════════════════════════╗
// ║                  api/mockClient                  ║
// ╚════════════════════════════════════════════════════╝

import { MOCK_SNAPSHOT } from '@/data/mock/snapshot'
import type {
  AppSettings,
  AppSnapshot,
  DiagnosticRun,
  DiagnosticStrategyResult,
  EngineStatus,
  MonitoredService,
  Profile,
  UpdateChannel,
} from '@/types'
import type { ZappApi } from './types'

function delay(ms = 420): Promise<void> {
  return new Promise((resolve) => {
    window.setTimeout(resolve, ms)
  })
}

function clone<T>(value: T): T {
  return structuredClone(value)
}

let state: AppSnapshot = clone(MOCK_SNAPSHOT)
let diagTimer: number | null = null

function stopDiagTimer() {
  if (diagTimer !== null) {
    window.clearInterval(diagTimer)
    diagTimer = null
  }
}

function bumpUptime() {
  if (state.connection.status === 'running') {
    state.connection.uptimeSeconds += 1
  }
}

window.setInterval(bumpUptime, 1000)

export const mockApi: ZappApi = {
  async getSnapshot() {
    await delay(180)
    return clone(state)
  },

  async getDiagnostics() {
    return clone(state.diagnostics)
  },

  async getUpdates() {
    return clone(state.updates)
  },

  async resetEngine() {
    await delay(400)
    state.connection.status = 'stopped'
    state.connection.uptimeSeconds = 0
    state.connection.sessionStartedAt = null
    state.connection.lastError = null
    return state.connection.status
  },

  async setEngineRunning(running) {
    await delay(500)
    state.connection.status = running ? 'starting' : 'stopping'
    await delay(650)
    const next: EngineStatus = running ? 'running' : 'stopped'
    state.connection.status = next
    if (running) {
      state.connection.sessionStartedAt = new Date().toISOString()
      state.connection.uptimeSeconds = 0
      state.connection.lastError = null
    } else {
      state.connection.sessionStartedAt = null
      state.connection.uptimeSeconds = 0
    }
    return next
  },

  async setActiveProfile(profileId) {
    await delay(350)
    state.profiles = state.profiles.map((p) => ({
      ...p,
      isActive: p.id === profileId,
    }))
    const active = state.profiles.find((p) => p.id === profileId)
    if (!active) throw new Error('Profile not found')
    state.activeProfile = active
    state.connection.strategyId = active.id
    state.connection.strategyName = active.name
    return clone(active)
  },

  async startDiagnostics() {
    stopDiagTimer()
    await delay(200)

    const demoResults = clone(MOCK_SNAPSHOT.diagnostics.results)
    const total = demoResults.length + 4

    state.diagnostics = {
      id: `diag-demo-${Date.now()}`,
      phase: 'preparing',
      progress: 0,
      currentStrategy: 'Preparing…',
      testedCount: 0,
      totalCount: total,
      workingCount: 0,
      startedAt: new Date().toISOString(),
      finishedAt: null,
      isDemo: true,
      results: [],
    }

    let index = 0
    const queue: DiagnosticStrategyResult[] = [
      ...demoResults,
      {
        id: `extra-${Date.now()}-1`,
        name: 'General ALT3',
        strategyType: 'nfqws',
        success: false,
        latencyMs: null,
        score: 18,
        notes: 'Demo fail',
        testedAt: new Date().toISOString(),
      },
      {
        id: `extra-${Date.now()}-2`,
        name: 'General ALT4',
        strategyType: 'disorder',
        success: true,
        latencyMs: 141,
        score: 70,
        notes: 'Demo pass',
        testedAt: new Date().toISOString(),
      },
      {
        id: `extra-${Date.now()}-3`,
        name: 'General ALT5',
        strategyType: 'split',
        success: false,
        latencyMs: null,
        score: 10,
        notes: 'Demo fail',
        testedAt: new Date().toISOString(),
      },
      {
        id: `extra-${Date.now()}-4`,
        name: 'FAKE TLS AUTO ALT2',
        strategyType: 'fake + multisplit',
        success: true,
        latencyMs: 89,
        score: 91,
        notes: 'Demo pass',
        testedAt: new Date().toISOString(),
      },
    ]

    state.diagnostics.phase = 'testing'
    state.diagnostics.totalCount = queue.length

    diagTimer = window.setInterval(() => {
      if (index >= queue.length) {
        stopDiagTimer()
        state.diagnostics.phase = 'analyzing'
        state.diagnostics.currentStrategy = 'Analyzing results…'
        state.diagnostics.progress = 96
        window.setTimeout(() => {
          state.diagnostics.phase = 'completed'
          state.diagnostics.progress = 100
          state.diagnostics.currentStrategy = null
          state.diagnostics.finishedAt = new Date().toISOString()
        }, 700)
        return
      }

      const item = {
        ...queue[index],
        testedAt: new Date().toISOString(),
      }
      state.diagnostics.results = [...state.diagnostics.results, item]
      state.diagnostics.testedCount = index + 1
      state.diagnostics.workingCount = state.diagnostics.results.filter((r) => r.success).length
      state.diagnostics.currentStrategy = item.name
      state.diagnostics.progress = Math.round(((index + 1) / queue.length) * 92)
      index += 1
    }, 480)

    return clone(state.diagnostics)
  },

  async cancelDiagnostics() {
    stopDiagTimer()
    await delay(150)
    state.diagnostics.phase = 'cancelled'
    state.diagnostics.currentStrategy = null
    state.diagnostics.finishedAt = new Date().toISOString()
    return clone(state.diagnostics)
  },

  async selectDiagnosticStrategy(resultId) {
    await delay(300)
    const result = state.diagnostics.results.find((r) => r.id === resultId)
    if (!result || !result.success) throw new Error('Strategy unavailable')

    const profile: Profile = {
      id: `profile-from-${result.id}`,
      name: result.name,
      kind: 'auto',
      strategyType: result.strategyType,
      source: 'auto',
      description: 'Выбрано из демо-результатов диагностики.',
      isActive: true,
      isAutoSelected: true,
      lastCheckedAt: new Date().toISOString(),
      lastUpdatedAt: new Date().toISOString(),
      tags: ['auto', 'diagnostics'],
    }

    state.profiles = state.profiles.map((p) => ({ ...p, isActive: false }))
    const existing = state.profiles.find((p) => p.name === result.name)
    if (existing) {
      existing.isActive = true
      existing.isAutoSelected = true
      existing.lastCheckedAt = profile.lastCheckedAt
      state.activeProfile = existing
      state.connection.strategyId = existing.id
      state.connection.strategyName = existing.name
      return clone(existing)
    }

    state.profiles = [profile, ...state.profiles]
    state.activeProfile = profile
    state.connection.strategyId = profile.id
    state.connection.strategyName = profile.name
    return clone(profile)
  },

  async checkService(serviceId) {
    await delay(700)
    const service = state.services.find((s) => s.id === serviceId)
    if (!service) throw new Error('Service not found')

    service.status = 'checking'
    await delay(500)

    const ok = Math.random() > 0.25
    service.status = ok ? 'available' : 'unavailable'
    service.latencyMs = ok ? Math.round(70 + Math.random() * 90) : null
    service.lastCheckedAt = new Date().toISOString()
    service.diagnosticSummary = ok
      ? 'Доступен. Демо-проверка прошла успешно.'
      : 'Недоступен. Демо-проверка завершилась ошибкой.'
    service.history = [
      {
        id: `chk-${Date.now()}`,
        checkedAt: service.lastCheckedAt,
        status: service.status,
        latencyMs: service.latencyMs,
        note: ok ? 'OK' : 'Demo failure',
      },
      ...service.history,
    ].slice(0, 8)

    return clone(service)
  },

  async checkAllServices() {
    const results: MonitoredService[] = []
    for (const service of state.services) {
      results.push(await this.checkService(service.id))
    }
    return clone(results)
  },

  async checkUpdates() {
    await delay(600)
    const now = new Date().toISOString()
    state.updates = state.updates.map((u) => ({
      ...u,
      lastCheckedAt: now,
    }))
    state.system.lastUpdateCheckAt = now
    return clone(state.updates)
  },

  async updateChannel(channelId) {
    await delay(200)
    const channel = state.updates.find((u) => u.id === channelId)
    if (!channel) throw new Error('Channel not found')
    if (channel.status === 'up_to_date') return clone(channel)

    channel.status = 'downloading'
    channel.downloadProgress = 0

    await new Promise<void>((resolve) => {
      const timer = window.setInterval(() => {
        channel.downloadProgress = Math.min(100, (channel.downloadProgress ?? 0) + 12)
        if ((channel.downloadProgress ?? 0) >= 100) {
          window.clearInterval(timer)
          channel.status = 'installing'
          window.setTimeout(() => {
            channel.status = 'up_to_date'
            channel.currentVersion = channel.latestVersion
            channel.downloadProgress = null
            if (channel.id === 'engine') state.system.engineVersion = channel.currentVersion
            if (channel.id === 'strategies') state.system.strategiesVersion = channel.currentVersion
            if (channel.id === 'application') {
              state.system.appVersion = channel.currentVersion
              state.about.version = channel.currentVersion
            }
            resolve()
          }, 700)
        }
      }, 180)
    })

    return clone(channel)
  },

  async updateSettings(patch) {
    await delay(220)
    state.settings = { ...state.settings, ...patch }
    return clone(state.settings)
  },

  async setTheme(mode) {
    await delay(80)
    state.settings.appearance = mode
    return mode
  },
}

export function getMockDiagnosticsLive(): DiagnosticRun {
  return clone(state.diagnostics)
}

export function getMockUpdatesLive(): UpdateChannel[] {
  return clone(state.updates)
}

export function getMockSettingsLive(): AppSettings {
  return clone(state.settings)
}

export function resetMockState() {
  stopDiagTimer()
  state = clone(MOCK_SNAPSHOT)
}
