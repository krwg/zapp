// ╔════════════════════════════════════════════════════╗
// ║            components/ui/Toast/Toast             ║
// ╚════════════════════════════════════════════════════╝

import { X } from 'lucide-react'
import styles from './Toast.module.css'

export type ToastTone = 'neutral' | 'success' | 'error' | 'warning'

export interface ToastItem {
  id: string
  title: string
  description?: string
  tone: ToastTone
  actionLabel?: string
  onAction?: () => void
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
          <div className={styles.body}>
            <p className={styles.title}>{item.title}</p>
            {item.description ? <p className={styles.description}>{item.description}</p> : null}
            {item.actionLabel && item.onAction ? (
              <button
                type="button"
                className={styles.action}
                onClick={() => {
                  item.onAction?.()
                  onDismiss(item.id)
                }}
              >
                {item.actionLabel}
              </button>
            ) : null}
          </div>
          <button
            type="button"
            className={styles.close}
            aria-label="Close"
            onClick={() => onDismiss(item.id)}
          >
            <X size={14} />
          </button>
        </div>
      ))}
    </div>
  )
}
