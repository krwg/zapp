// ╔════════════════════════════════════════════════════╗
// ║             context/AppStateContext              ║
// ╚════════════════════════════════════════════════════╝

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from 'react'
import { getApi } from '@/api/client'
import type {
  AppSettings,
  AppSnapshot,
  DiagnosticRun,
  MonitoredService,
  Profile,
  UpdateChannel,
} from '@/types'
import { useLocale } from './LocaleContext'
import { useToast } from './ToastContext'
import { useTheme } from './ThemeContext'

interface AppStateContextValue {
  loading: boolean
  snapshot: AppSnapshot | null
  refresh: () => Promise<void>
  toggleEngine: () => Promise<void>
  resetEngine: () => Promise<void>
  setActiveProfile: (id: string) => Promise<void>
  startDiagnostics: () => Promise<void>
  cancelDiagnostics: () => Promise<void>
  selectDiagnosticStrategy: (resultId: string) => Promise<void>
  diagnosticsLive: DiagnosticRun | null
  checkService: (id: string) => Promise<void>
  checkAllServices: (opts?: { silent?: boolean }) => Promise<void>
  checkUpdates: () => Promise<void>
  updateChannel: (id: UpdateChannel['id']) => Promise<void>
  updatesLive: UpdateChannel[]
  updateSettings: (patch: Partial<AppSettings>) => Promise<void>
  busy: Record<string, boolean>
}

const AppStateContext = createContext<AppStateContextValue | null>(null)

export function AppStateProvider({ children }: { children: ReactNode }) {
  const api = getApi()
  const { push } = useToast()
  const { setMode } = useTheme()
  const { t } = useLocale()
  const [loading, setLoading] = useState(true)
  const [snapshot, setSnapshot] = useState<AppSnapshot | null>(null)
  const [diagnosticsLive, setDiagnosticsLive] = useState<DiagnosticRun | null>(null)
  const [updatesLive, setUpdatesLive] = useState<UpdateChannel[]>([])
  const [busy, setBusy] = useState<Record<string, boolean>>({})

  const refresh = useCallback(async () => {
    const data = await api.getSnapshot()
    setSnapshot(data)
    setDiagnosticsLive(data.diagnostics)
    setUpdatesLive(data.updates)
    setMode(data.settings.appearance)
  }, [api, setMode])

  const withBusy = useCallback(
    async (key: string, fn: () => Promise<void>) => {
      setBusy((prev) => ({ ...prev, [key]: true }))
      try {
        await fn()
      } catch (err) {
        const message = formatInvokeError(err, t('common.unknownError'))
        push({
          title: t('toast.errorTitle'),
          description: message,
          tone: 'error',
        })
        try {
          await refresh()
        } catch {
          
        }
      } finally {
        setBusy((prev) => ({ ...prev, [key]: false }))
      }
    },
    [push, refresh, t],
  )

  useEffect(() => {
    let cancelled = false
    ;(async () => {
      try {
        await refresh()
      } finally {
        if (!cancelled) setLoading(false)
      }
    })()
    return () => {
      cancelled = true
    }
  }, [refresh])

  useEffect(() => {
    let unlisten: (() => void) | undefined
    void (async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event')
        unlisten = await listen('app:quit-for-update', () => {
          push({
            title: t('toast.quitForUpdateTitle'),
            description: t('toast.quitForUpdateDesc'),
            tone: 'warning',
          })
        })
      } catch {
        /* browser mock */
      }
    })()
    return () => {
      unlisten?.()
    }
  }, [push, t])

  useEffect(() => {
    if (loading) return

    const timer = window.setInterval(() => {
      void (async () => {
        try {
          const snap = await api.getSnapshot()
          setDiagnosticsLive(snap.diagnostics)
          setUpdatesLive(snap.updates)
          setSnapshot(snap)
        } catch {
          /* keep last UI state */
        }
      })()
    }, 2500)
    return () => window.clearInterval(timer)
  }, [api, loading])

  const toggleEngine = useCallback(async () => {
    if (!snapshot) return
    const running = snapshot.connection.status === 'running'
    await withBusy('engine', async () => {
      setSnapshot((prev) =>
        prev
          ? {
              ...prev,
              connection: {
                ...prev.connection,
                status: running ? 'stopping' : 'starting',
              },
            }
          : prev,
      )
      const status = await api.setEngineRunning(!running)
      await refresh()
      if (status === 'running') {
        push({
          title: t('toast.protectionOnTitle'),
          description: t('toast.protectionOnDesc'),
          tone: 'success',
        })
      } else {
        push({
          title: t('toast.protectionOffTitle'),
          description: t('toast.protectionOffDesc'),
          tone: 'success',
        })
      }
    })
  }, [api, push, refresh, snapshot, t, withBusy])

  const resetEngine = useCallback(async () => {
    await withBusy('engine', async () => {
      await api.resetEngine()
      await refresh()
      push({
        title: t('toast.repairedTitle'),
        description: t('toast.repairedDesc'),
        tone: 'warning',
      })
    })
  }, [api, push, refresh, t, withBusy])

  const setActiveProfile = useCallback(
    async (id: string) => {
      await withBusy(`profile:${id}`, async () => {
        await api.setActiveProfile(id)
        await refresh()
        push({
          title: t('toast.modeActivatedTitle'),
          description: t('toast.modeActivatedDesc'),
          tone: 'success',
        })
      })
    },
    [api, push, refresh, t, withBusy],
  )

  const startDiagnostics = useCallback(async () => {
    await withBusy('diagnostics', async () => {
      const run = await api.startDiagnostics()
      setDiagnosticsLive(run)
      const queued = (run.currentStrategy ?? '').toLowerCase().includes('busy')
      push({
        title: queued ? t('toast.checkQueuedTitle') : t('toast.checkStartedTitle'),
        description: queued ? t('toast.checkQueuedDesc') : t('toast.checkStartedDesc'),
        tone: queued ? 'warning' : 'neutral',
      })
    })
  }, [api, push, t, withBusy])

  const cancelDiagnostics = useCallback(async () => {
    await withBusy('diagnostics', async () => {
      const run = await api.cancelDiagnostics()
      setDiagnosticsLive(run)
      push({
        title: t('toast.checkCancelledTitle'),
        description: t('toast.checkCancelledDesc'),
        tone: 'warning',
      })
    })
  }, [api, push, t, withBusy])

  const selectDiagnosticStrategy = useCallback(
    async (resultId: string) => {
      await withBusy(`diag-select:${resultId}`, async () => {
        await api.selectDiagnosticStrategy(resultId)
        await refresh()
        push({
          title: t('toast.modeAppliedTitle'),
          description: t('toast.modeAppliedDesc'),
          tone: 'success',
        })
      })
    },
    [api, push, refresh, t, withBusy],
  )

  const checkService = useCallback(
    async (id: string) => {
      await withBusy(`service:${id}`, async () => {
        setSnapshot((prev) => {
          if (!prev) return prev
          return {
            ...prev,
            services: prev.services.map((s) =>
              s.id === id ? { ...s, status: 'checking' as const } : s,
            ),
          }
        })
        const updated = await api.checkService(id)
        setSnapshot((prev) => {
          if (!prev) return prev
          return {
            ...prev,
            services: prev.services.map((s) => (s.id === id ? updated : s)),
          }
        })
        push({
          title: t('toast.siteCheckTitle'),
          description: t('toast.siteCheckDesc', { name: updated.name }),
          tone: 'success',
        })
      })
    },
    [api, push, t, withBusy],
  )

  const checkAllServices = useCallback(
    async (opts?: { silent?: boolean }) => {
      await withBusy('services:all', async () => {
        const list: MonitoredService[] = await api.checkAllServices()
        setSnapshot((prev) => (prev ? { ...prev, services: list } : prev))
        if (!opts?.silent) {
          push({
            title: t('toast.sitesCheckedTitle'),
            description: t('toast.sitesCheckedDesc'),
            tone: 'success',
          })
        }
      })
    },
    [api, push, t, withBusy],
  )

  const checkUpdates = useCallback(async () => {
    await withBusy('updates:check', async () => {
      const list = await api.checkUpdates()
      setUpdatesLive(list)
      setSnapshot((prev) =>
        prev
          ? {
              ...prev,
              updates: list,
              system: { ...prev.system, lastUpdateCheckAt: new Date().toISOString() },
            }
          : prev,
      )
      push({
        title: t('toast.updatesCheckedTitle'),
        description: t('toast.updatesCheckedDesc'),
        tone: 'neutral',
      })
    })
  }, [api, push, t, withBusy])

  const updateChannel = useCallback(
    async (id: UpdateChannel['id']) => {
      await withBusy(`update:${id}`, async () => {
        const poll = window.setInterval(() => {
          void api.getUpdates().then(setUpdatesLive).catch(() => undefined)
        }, 200)
        try {
          await api.updateChannel(id)
          await refresh()
          push({
            title: t('toast.channelUpdatedTitle', { id }),
            description: t('toast.channelUpdatedDesc'),
            tone: 'success',
          })
        } finally {
          window.clearInterval(poll)
        }
      })
    },
    [api, push, refresh, t, withBusy],
  )

  const updateSettings = useCallback(
    async (patch: Partial<AppSettings>) => {
      await withBusy('settings', async () => {
        const next = await api.updateSettings(patch)
        if (patch.appearance) setMode(patch.appearance)
        setSnapshot((prev) => (prev ? { ...prev, settings: next } : prev))
        push({
          title: t('toast.settingsSavedTitle'),
          description: t('toast.settingsSavedDesc'),
          tone: 'success',
        })
      })
    },
    [api, push, setMode, t, withBusy],
  )

  const value = useMemo<AppStateContextValue>(
    () => ({
      loading,
      snapshot,
      refresh,
      toggleEngine,
      resetEngine,
      setActiveProfile,
      startDiagnostics,
      cancelDiagnostics,
      selectDiagnosticStrategy,
      diagnosticsLive,
      checkService,
      checkAllServices,
      checkUpdates,
      updateChannel,
      updatesLive,
      updateSettings,
      busy,
    }),
    [
      loading,
      snapshot,
      refresh,
      toggleEngine,
      resetEngine,
      setActiveProfile,
      startDiagnostics,
      cancelDiagnostics,
      selectDiagnosticStrategy,
      diagnosticsLive,
      checkService,
      checkAllServices,
      checkUpdates,
      updateChannel,
      updatesLive,
      updateSettings,
      busy,
    ],
  )

  return <AppStateContext.Provider value={value}>{children}</AppStateContext.Provider>
}

export function useAppState() {
  const ctx = useContext(AppStateContext)
  if (!ctx) throw new Error('useAppState must be used within AppStateProvider')
  return ctx
}

function formatInvokeError(err: unknown, fallback: string): string {
  if (typeof err === 'string') return err
  if (err instanceof Error && err.message) return err.message
  if (err && typeof err === 'object') {
    const record = err as Record<string, unknown>
    if (typeof record.message === 'string') return record.message
    try {
      return JSON.stringify(err)
    } catch {
      
    }
  }
  return fallback
}

export type { Profile }
