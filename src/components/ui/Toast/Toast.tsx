// ╔════════════════════════════════════════════════════╗
// ║            components/ui/Toast/Toast             ║
// ╚════════════════════════════════════════════════════╝

import { X } from 'lucide-react'
import styles from './Toast.module.css'

export type ToastTone = 'neutral' | 'success' | 'error' | 'warning'

export interface ToastItem {
  id: string
  message: string
  tone: ToastTone
}

interface ToastViewportProps {
  items: ToastItem[]
  onDismiss: (id: string) => void
}

export function ToastViewport({ items, onDismiss }: ToastViewportProps) {
  return (
    <div className={styles.viewport} aria-live="polite">
      {items.map((item) => (
        <div key={item.id} className={[styles.toast, styles[item.tone]].join(' ')} role="status">
          <span className={styles.accent} aria-hidden />
          <p className={styles.message}>{item.message}</p>
          <button
            type="button"
            className={styles.close}
            aria-label="Закрыть"
            onClick={() => onDismiss(item.id)}
          >
            <X size={14} />
          </button>
        </div>
      ))}
    </div>
  )
}
