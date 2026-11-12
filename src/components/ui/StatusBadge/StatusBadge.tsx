// ╔════════════════════════════════════════════════════╗
// ║      components/ui/StatusBadge/StatusBadge       ║
// ╚════════════════════════════════════════════════════╝

import styles from './StatusBadge.module.css'

export type StatusTone = 'success' | 'error' | 'warning' | 'neutral'

interface StatusBadgeProps {
  label: string
  tone?: StatusTone
  pulse?: boolean
}

export function StatusBadge({ label, tone = 'neutral', pulse = false }: StatusBadgeProps) {
  return (
    <span className={[styles.badge, styles[tone], pulse ? styles.pulse : ''].join(' ')}>
      <span className={styles.dot} aria-hidden />
      {label}
    </span>
  )
}
