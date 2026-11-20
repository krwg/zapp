// ╔════════════════════════════════════════════════════╗
// ║               context/ToastContext               ║
// ╚════════════════════════════════════════════════════╝

import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from 'react'
import { ToastViewport, type ToastItem, type ToastTone } from '@/components/ui/Toast/Toast'

export interface ToastPayload {
  title: string
  description?: string
  tone?: ToastTone
  actionLabel?: string
  onAction?: () => void
}

interface ToastContextValue {
  push: (payload: ToastPayload) => void
}

const ToastContext = createContext<ToastContextValue | null>(null)

export function ToastProvider({ children }: { children: ReactNode }) {
  const [items, setItems] = useState<ToastItem[]>([])

  const dismiss = useCallback((id: string) => {
    setItems((prev) => prev.filter((t) => t.id !== id))
  }, [])

  const push = useCallback(
    (payload: ToastPayload) => {
      const tone = payload.tone ?? 'neutral'
      const id = `toast-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`
      setItems((prev) => [
        ...prev.slice(-4),
        {
          id,
          title: payload.title,
          description: payload.description,
          tone,
          actionLabel: payload.actionLabel,
          onAction: payload.onAction,
        },
      ])
      const ms = tone === 'error' ? 8000 : 3800
      window.setTimeout(() => dismiss(id), ms)
    },
    [dismiss],
  )

  const value = useMemo(() => ({ push }), [push])

  return (
    <ToastContext.Provider value={value}>
      {children}
      <ToastViewport items={items} onDismiss={dismiss} />
    </ToastContext.Provider>
  )
}

export function useToast() {
  const ctx = useContext(ToastContext)
  if (!ctx) throw new Error('useToast must be used within ToastProvider')
  return ctx
}
