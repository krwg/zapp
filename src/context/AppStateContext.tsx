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
  checkAllServices: () => Promise<void>
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
        const message = formatInvokeError(err)
        push(message, 'error')
        try {
          await refresh()
        } catch {
          
        }
      } finally {
        setBusy((prev) => ({ ...prev, [key]: false }))
      }
    },
    [push, refresh],
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
          push('Обновление приложения запущено — zapp закроется', 'warning')
        })
      } catch {
        /* browser mock */
      }
    })()
    return () => {
      unlisten?.()
    }
  }, [push])

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
      push(status === 'running' ? 'Движок запущен' : 'Движок остановлен', 'success')
    })
  }, [api, push, refresh, snapshot, withBusy])

  const resetEngine = useCallback(async () => {
    await withBusy('engine', async () => {
      await api.resetEngine()
      await refresh()
      push('Сброс winws выполнен', 'warning')
    })
  }, [api, push, refresh, withBusy])

  const setActiveProfile = useCallback(
    async (id: string) => {
      await withBusy(`profile:${id}`, async () => {
        await api.setActiveProfile(id)
        await refresh()
        push('Профиль активирован', 'success')
      })
    },
    [api, push, refresh, withBusy],
  )

  const startDiagnostics = useCallback(async () => {
    await withBusy('diagnostics', async () => {
      const run = await api.startDiagnostics()
      setDiagnosticsLive(run)
      push('Диагностика запущена', 'neutral')
    })
  }, [api, push, withBusy])

  const cancelDiagnostics = useCallback(async () => {
    await withBusy('diagnostics', async () => {
      const run = await api.cancelDiagnostics()
      setDiagnosticsLive(run)
      push('Диагностика отменена', 'warning')
    })
  }, [api, push, withBusy])

  const selectDiagnosticStrategy = useCallback(
    async (resultId: string) => {
      await withBusy(`diag-select:${resultId}`, async () => {
        await api.selectDiagnosticStrategy(resultId)
        await refresh()
        push('Стратегия применена', 'success')
      })
    },
    [api, push, refresh, withBusy],
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
      })
    },
    [api, withBusy],
  )

  const checkAllServices = useCallback(async () => {
    await withBusy('services:all', async () => {
      const list: MonitoredService[] = await api.checkAllServices()
      setSnapshot((prev) => (prev ? { ...prev, services: list } : prev))
      push('Проверка сервисов завершена', 'success')
    })
  }, [api, push, withBusy])

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
      push('Проверка обновлений завершена', 'neutral')
    })
  }, [api, push, withBusy])

  const updateChannel = useCallback(
    async (id: UpdateChannel['id']) => {
      await withBusy(`update:${id}`, async () => {
        const poll = window.setInterval(() => {
          void api.getUpdates().then(setUpdatesLive).catch(() => undefined)
        }, 200)
        try {
          await api.updateChannel(id)
          await refresh()
          push(`Канал ${id} обновлён`, 'success')
        } finally {
          window.clearInterval(poll)
        }
      })
    },
    [api, push, refresh, withBusy],
  )

  const updateSettings = useCallback(
    async (patch: Partial<AppSettings>) => {
      await withBusy('settings', async () => {
        const next = await api.updateSettings(patch)
        if (patch.appearance) setMode(patch.appearance)
        setSnapshot((prev) => (prev ? { ...prev, settings: next } : prev))
        push('Настройки сохранены', 'success')
      })
    },
    [api, push, setMode, withBusy],
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

function formatInvokeError(err: unknown): string {
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
  return 'Неизвестная ошибка'
}

export type { Profile }
