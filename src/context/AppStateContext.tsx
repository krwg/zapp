// ╔════════════════════════════════════════════════════╗
// ║             context/AppStateContext              ║
// ╚════════════════════════════════════════════════════╝

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from 'react'
import { useNavigate } from 'react-router-dom'
import { getApi } from '@/api/client'
import type {
  AppSettings,
  AppSnapshot,
  DiagnosticRun,
  EngineStatus,
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
  toggleTelegramProxy: () => Promise<void>
  openTelegramProxyLink: () => Promise<void>
  copyTelegramProxyLink: () => Promise<void>
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
  updateSettings: (patch: Partial<AppSettings>, opts?: { quiet?: boolean }) => Promise<void>
  getRecentErrors: () => Promise<string[]>
  busy: Record<string, boolean>
}

const AppStateContext = createContext<AppStateContextValue | null>(null)

const TRANSITIONAL: EngineStatus[] = ['starting', 'stopping']

function mergeSnapshot(prev: AppSnapshot | null, next: AppSnapshot): AppSnapshot {
  if (!prev) return next

  let connection = next.connection
  if (
    TRANSITIONAL.includes(prev.connection.status) &&
    (next.connection.status === 'stopped' || next.connection.status === 'running')
  ) {
    // Keep optimistic starting/stopping until the action settles.
    connection = { ...next.connection, status: prev.connection.status }
  }

  let telegram = next.telegram
  if (
    prev.telegram &&
    TRANSITIONAL.includes(prev.telegram.status) &&
    (next.telegram.status === 'stopped' || next.telegram.status === 'running')
  ) {
    telegram = { ...next.telegram, status: prev.telegram.status }
  }

  const services = next.services.map((s) => {
    const old = prev.services.find((p) => p.id === s.id)
    if (old?.status === 'checking' && s.status !== 'checking') {
      return { ...s, status: 'checking' as const }
    }
    return s
  })

  return { ...next, connection, telegram, services }
}

export function AppStateProvider({ children }: { children: ReactNode }) {
  const api = useMemo(() => getApi(), [])
  const { push } = useToast()
  const { setMode } = useTheme()
  const { t } = useLocale()
  const navigate = useNavigate()
  const [loading, setLoading] = useState(true)
  const [snapshot, setSnapshot] = useState<AppSnapshot | null>(null)
  const [diagnosticsLive, setDiagnosticsLive] = useState<DiagnosticRun | null>(null)
  const [updatesLive, setUpdatesLive] = useState<UpdateChannel[]>([])
  const [busy, setBusy] = useState<Record<string, boolean>>({})
  const engineActionRef = useRef(0)
  const tgActionRef = useRef(0)
  const checkingIdsRef = useRef<Set<string>>(new Set())

  const refresh = useCallback(async () => {
    const data = await api.getSnapshot()
    setSnapshot((prev) => mergeSnapshot(prev, data))
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
          actionLabel: t('aboutPage.reportBug'),
          onAction: () => navigate('/about'),
        })
        try {
          await refresh()
        } catch {
          /* keep UI */
        }
      } finally {
        setBusy((prev) => ({ ...prev, [key]: false }))
      }
    },
    [navigate, push, refresh, t],
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
          setSnapshot((prev) => {
            let merged = mergeSnapshot(prev, snap)
            // Drop transitional once engine action finished and server agrees.
            if (
              prev &&
              TRANSITIONAL.includes(prev.connection.status) &&
              engineActionRef.current === 0 &&
              !TRANSITIONAL.includes(snap.connection.status)
            ) {
              merged = { ...merged, connection: snap.connection }
            }
            if (
              prev &&
              TRANSITIONAL.includes(prev.telegram.status) &&
              tgActionRef.current === 0 &&
              !TRANSITIONAL.includes(snap.telegram.status)
            ) {
              merged = { ...merged, telegram: snap.telegram }
            }
            // Drop checking once individual checks cleared.
            if (checkingIdsRef.current.size === 0) {
              return { ...merged, services: snap.services }
            }
            return merged
          })
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
      engineActionRef.current += 1
      try {
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
        const data = await api.getSnapshot()
        setSnapshot(data)
        setDiagnosticsLive(data.diagnostics)
        setUpdatesLive(data.updates)
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
      } finally {
        engineActionRef.current = Math.max(0, engineActionRef.current - 1)
      }
    })
  }, [api, push, snapshot, t, withBusy])

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

  const toggleTelegramProxy = useCallback(async () => {
    if (!snapshot) return
    const running = snapshot.telegram.status === 'running'
    await withBusy('telegram', async () => {
      tgActionRef.current += 1
      try {
        setSnapshot((prev) =>
          prev
            ? {
                ...prev,
                telegram: {
                  ...prev.telegram,
                  status: running ? 'stopping' : 'starting',
                },
              }
            : prev,
        )
        const tg = await api.setTelegramProxyRunning(!running)
        const data = await api.getSnapshot()
        setSnapshot({ ...data, telegram: tg })
        setDiagnosticsLive(data.diagnostics)
        setUpdatesLive(data.updates)
        if (tg.status === 'running') {
          push({
            title: t('telegram.onTitle'),
            description: t('telegram.onDesc'),
            tone: 'success',
          })
        } else {
          push({
            title: t('telegram.offTitle'),
            description: t('telegram.offDesc'),
            tone: 'success',
          })
        }
      } finally {
        tgActionRef.current = Math.max(0, tgActionRef.current - 1)
      }
    })
  }, [api, push, snapshot, t, withBusy])

  const openTelegramProxyLink = useCallback(async () => {
    const link =
      snapshot?.telegram.proxyLink ?? (await api.getTelegramProxyLink())
    await api.openExternalUrl(link)
  }, [api, snapshot])

  const copyTelegramProxyLink = useCallback(async () => {
    const link =
      snapshot?.telegram.proxyLink ?? (await api.getTelegramProxyLink())
    try {
      await navigator.clipboard.writeText(link)
      push({
        title: t('telegram.linkCopiedTitle'),
        description: t('telegram.linkCopiedDesc'),
        tone: 'success',
      })
    } catch {
      push({
        title: t('telegram.linkCopiedTitle'),
        description: link,
        tone: 'neutral',
      })
    }
  }, [api, push, snapshot, t])

  const getRecentErrors = useCallback(async () => {
    return api.getRecentErrors()
  }, [api])

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
    await withBusy('diagnostics:start', async () => {
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
    await withBusy('diagnostics:cancel', async () => {
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
        checkingIdsRef.current.add(id)
        try {
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
        } finally {
          checkingIdsRef.current.delete(id)
        }
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
        await api.updateChannel(id)
        // Download/install runs in the backend; poll until settled.
        await new Promise<void>((resolve, reject) => {
          const started = Date.now()
          const poll = window.setInterval(() => {
            void api
              .getUpdates()
              .then((list) => {
                setUpdatesLive(list)
                const ch = list.find((c) => c.id === id)
                if (!ch) return
                if (ch.status === 'downloading' || ch.status === 'installing') return
                window.clearInterval(poll)
                if (ch.status === 'error') {
                  reject(new Error(ch.releaseNotes || t('common.unknownError')))
                  return
                }
                resolve()
              })
              .catch(() => undefined)
            if (Date.now() - started > 10 * 60 * 1000) {
              window.clearInterval(poll)
              reject(new Error(t('common.unknownError')))
            }
          }, 200)
        })
        await refresh()
        push({
          title: t('toast.channelUpdatedTitle', { id }),
          description: t('toast.channelUpdatedDesc'),
          tone: 'success',
        })
      })
    },
    [api, push, refresh, t, withBusy],
  )

  const updateSettings = useCallback(
    async (patch: Partial<AppSettings>, opts?: { quiet?: boolean }) => {
      await withBusy('settings', async () => {
        const next = await api.updateSettings(patch)
        if (patch.appearance) setMode(patch.appearance)
        setSnapshot((prev) => (prev ? { ...prev, settings: next } : prev))
        if (!opts?.quiet) {
          push({
            title: t('toast.settingsSavedTitle'),
            description: t('toast.settingsSavedDesc'),
            tone: 'success',
          })
        }
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
      toggleTelegramProxy,
      openTelegramProxyLink,
      copyTelegramProxyLink,
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
      getRecentErrors,
      busy,
    }),
    [
      loading,
      snapshot,
      refresh,
      toggleEngine,
      resetEngine,
      toggleTelegramProxy,
      openTelegramProxyLink,
      copyTelegramProxyLink,
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
      getRecentErrors,
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
      /* fall through */
    }
  }
  return fallback
}

export type { Profile }
