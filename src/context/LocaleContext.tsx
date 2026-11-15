// ╔════════════════════════════════════════════════════╗
// ║              context/LocaleContext               ║
// ╚════════════════════════════════════════════════════╝

import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from 'react'
import {
  getCatalog,
  translate,
  type Locale,
  type MessageKey,
  type MessageTree,
} from '@/i18n'

interface LocaleContextValue {
  locale: Locale
  setLocale: (locale: Locale) => void
  t: (key: MessageKey, vars?: Record<string, string | number>) => string
  messages: MessageTree
}

const LocaleContext = createContext<LocaleContextValue | null>(null)

const STORAGE_KEY = 'zapp.locale'

function detectLocale(): Locale {
  const stored = localStorage.getItem(STORAGE_KEY)
  if (stored === 'ru' || stored === 'en') return stored
  const nav = navigator.language.toLowerCase()
  return nav.startsWith('ru') ? 'ru' : 'en'
}

export function LocaleProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(() => detectLocale())

  const setLocale = useCallback((next: Locale) => {
    setLocaleState(next)
    localStorage.setItem(STORAGE_KEY, next)
  }, [])

  const t = useCallback(
    (key: MessageKey, vars?: Record<string, string | number>) =>
      translate(locale, key, vars),
    [locale],
  )

  const value = useMemo(
    () => ({
      locale,
      setLocale,
      t,
      messages: getCatalog(locale),
    }),
    [locale, setLocale, t],
  )

  return <LocaleContext.Provider value={value}>{children}</LocaleContext.Provider>
}

export function useLocale() {
  const ctx = useContext(LocaleContext)
  if (!ctx) throw new Error('useLocale must be used within LocaleProvider')
  return ctx
}
