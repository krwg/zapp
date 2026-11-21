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
import type { BugReportBundle, BugReportPrefill } from '@/types/bugReport'
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
  importEnginePack: () => Promise<void>
  updatesLive: UpdateChannel[]
  updateSettings: (patch: Partial<AppSettings>, opts?: { quiet?: boolean }) => Promise<void>
  applyWorkingMode: (strategyName: string) => Promise<void>
  resolveConflicts: (killForeignWinws?: boolean) => Promise<void>
  dismissConflicts: () => void
  getRecentErrors: () => Promise<string[]>
  openBugReport: (message: string, areas?: BugReportPrefill['areas']) => void
  getBugReportBundle: () => Promise<BugReportBundle>
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
    
    connection = { ...next.connection, status: prev.connection.status }
  }

  const services = next.services.map((s) => {
    const old = prev.services.find((p) => p.id === s.id)
    if (old?.status === 'checking' && s.status !== 'checking') {
      return { ...s, status: 'checking' as const }
    }
    return s
  })

  return { ...next, connection, services }
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
  const checkingIdsRef = useRef<Set<string>>(new Set())

  const refresh = useCallback(async () => {
    const data = await api.getSnapshot()
    setSnapshot((prev) => mergeSnapshot(prev, data))
    setDiagnosticsLive(data.diagnostics)
    setUpdatesLive(data.updates)
    setMode(data.settings.appearance)
  }, [api, setMode])

  const openBugReport = useCallback(
    (message: string, areas?: BugReportPrefill['areas']) => {
      navigate('/about', {
        state: {
          openBugReport: true,
          bugPrefill: {
            summary: message.split('—')[0]?.trim() || message.slice(0, 80),
            expected: t('aboutPage.reportExpectedPh'),
            actual: message,
            areas: areas ?? { bypass: true },
            
          },
        },
      })
    },
    [navigate, t],
  )

  const withBusy = useCallback(
    async (key: string, fn: () => Promise<void>) => {
      setBusy((prev) => ({ ...prev, [key]: true }))
      try {
        await fn()
      } catch (err) {
        const message = formatInvokeError(err, t('common.unknownError'))
        const areas = {
          bypass: key === 'engine' || key.startsWith('profile:'),
          smart: key.startsWith('diagnostics'),
          updates: key.startsWith('update') || key === 'updates:check',
        }
        push({
          title: t('toast.errorTitle'),
          description: message,
          tone: 'error',
          actionLabel: t('aboutPage.reportBug'),
          onAction: () => openBugReport(message, areas),
        })
        try {
          await refresh()
        } catch {
          
        }
      } finally {
        setBusy((prev) => ({ ...prev, [key]: false }))
      }
    },
    [openBugReport, push, refresh, t],
  )

  useEffect(() => {
    let cancelled = false
    ;(async () => {
      try {
        await refresh()
        
        try {
          const stored = localStorage.getItem('zapp.locale')
          const locale = stored === 'ru' || stored === 'en' ? stored : null
          if (locale) {
            await api.updateSettings({ uiLocale: locale })
          }
        } catch {
          
        }
      } finally {
        if (!cancelled) setLoading(false)
      }
    })()
    return () => {
      cancelled = true
    }
  }, [api, refresh])

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
            if (
              prev &&
              TRANSITIONAL.includes(prev.connection.status) &&
              engineActionRef.current === 0 &&
              !TRANSITIONAL.includes(snap.connection.status)
            ) {
              merged = { ...merged, connection: snap.connection }
            }
            if (checkingIdsRef.current.size === 0) {
              return { ...merged, services: snap.services }
            }
            return merged
          })
        } catch {
          /* keep last UI state */
        }
      })()
    }, 4000)
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

  const importEnginePack = useCallback(async () => {
    await withBusy('update:engine', async () => {
      await api.importEnginePack()
      await refresh()
      push({
        title: t('toast.channelUpdatedTitle', { id: 'engine' }),
        description: t('updates.importDone'),
        tone: 'success',
      })
    })
  }, [api, push, refresh, t, withBusy])

  const applyWorkingMode = useCallback(
    async (strategyName: string) => {
      await withBusy(`working:${strategyName}`, async () => {
        await api.applyWorkingMode(strategyName)
        await refresh()
        push({
          title: t('toast.modeReappliedTitle'),
          description: t('toast.modeReappliedDesc', { name: strategyName }),
          tone: 'success',
        })
      })
    },
    [api, push, refresh, t, withBusy],
  )

  const resolveConflicts = useCallback(
    async (killForeignWinws = true) => {
      await withBusy('conflicts', async () => {
        const info = await api.resolveConflicts(killForeignWinws)
        setSnapshot((prev) =>
          prev
            ? {
                ...prev,
                conflicts: info.images.length ? info : null,
              }
            : prev,
        )
        await refresh()
        push({
          title: t('toast.conflictsClearedTitle'),
          description: t('toast.conflictsClearedDesc', { detail: info.detail }),
          tone: 'success',
        })
      })
    },
    [api, push, refresh, t, withBusy],
  )

  const dismissConflicts = useCallback(() => {
    setSnapshot((prev) => (prev ? { ...prev, conflicts: null } : prev))
  }, [])

  useEffect(() => {
    const unlistens: Array<() => void> = []
    void (async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event')
        unlistens.push(
          await listen('app:quit-for-update', () => {
            push({
              title: t('toast.quitForUpdateTitle'),
              description: t('toast.quitForUpdateDesc'),
              tone: 'warning',
            })
          }),
        )
        unlistens.push(
          await listen<{ detail?: string; autoRecheck?: boolean }>(
            'watchdog:bypass_stale',
            (ev) => {
              const detail = ev.payload?.detail || ''
              const auto = !!ev.payload?.autoRecheck
              push({
                title: t('toast.watchdogStaleTitle'),
                description: t('toast.watchdogStaleDesc', { detail }),
                tone: 'warning',
                actionLabel: auto ? undefined : t('toast.watchdogRecheck'),
                onAction: auto
                  ? undefined
                  : () => {
                      void startDiagnostics()
                    },
              })
              void refresh()
            },
          ),
        )
        unlistens.push(
          await listen('network:changed', () => {
            void refresh()
          }),
        )
        unlistens.push(
          await listen('engine:restored', () => {
            void refresh()
          }),
        )
        unlistens.push(
          await listen('diagnostics:completed', () => {
            void refresh()
          }),
        )
        unlistens.push(
          await listen('diagnostics:progress', () => {
            void api.getDiagnostics().then((run) => {
              setDiagnosticsLive(run)
            })
          }),
        )
      } catch {
        /* browser mock */
      }
    })()
    return () => {
      for (const u of unlistens) u()
    }
  }, [api, push, refresh, startDiagnostics, t])

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

  const getRecentErrors = useCallback(async () => {
    return api.getRecentErrors()
  }, [api])

  const getBugReportBundle = useCallback(async () => {
    return api.getBugReportBundle()
  }, [api])

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
      importEnginePack,
      updatesLive,
      updateSettings,
      applyWorkingMode,
      resolveConflicts,
      dismissConflicts,
      getRecentErrors,
      getBugReportBundle,
      openBugReport,
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
      importEnginePack,
      updatesLive,
      updateSettings,
      applyWorkingMode,
      resolveConflicts,
      dismissConflicts,
      getRecentErrors,
      getBugReportBundle,
      openBugReport,
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

