// ╔════════════════════════════════════════════════════╗
// ║              context/UiPrefsContext              ║
// ╚════════════════════════════════════════════════════╝

import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from 'react'

export interface UiPrefs {
  confirmDisableProtection: boolean
  showSidebarSession: boolean
  bugReportHintSeen: boolean
}

interface UiPrefsContextValue extends UiPrefs {
  setPref: <K extends keyof UiPrefs>(key: K, value: UiPrefs[K]) => void
}

const STORAGE_KEY = 'zapp.uiPrefs'
const DEFAULTS: UiPrefs = {
  confirmDisableProtection: false,
  showSidebarSession: true,
  bugReportHintSeen: false,
}

const UiPrefsContext = createContext<UiPrefsContextValue | null>(null)

function load(): UiPrefs {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return DEFAULTS
    return { ...DEFAULTS, ...(JSON.parse(raw) as Partial<UiPrefs>) }
  } catch {
    return DEFAULTS
  }
}

export function UiPrefsProvider({ children }: { children: ReactNode }) {
  const [prefs, setPrefs] = useState<UiPrefs>(() => load())

  const setPref = useCallback(<K extends keyof UiPrefs>(key: K, value: UiPrefs[K]) => {
    setPrefs((prev) => {
      const next = { ...prev, [key]: value }
      localStorage.setItem(STORAGE_KEY, JSON.stringify(next))
      return next
    })
  }, [])

  const value = useMemo(() => ({ ...prefs, setPref }), [prefs, setPref])

  return <UiPrefsContext.Provider value={value}>{children}</UiPrefsContext.Provider>
}

export function useUiPrefs() {
  const ctx = useContext(UiPrefsContext)
  if (!ctx) throw new Error('useUiPrefs must be used within UiPrefsProvider')
  return ctx
}
