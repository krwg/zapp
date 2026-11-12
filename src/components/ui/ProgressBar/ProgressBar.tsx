// ╔════════════════════════════════════════════════════╗
// ║      components/ui/ProgressBar/ProgressBar       ║
// ╚════════════════════════════════════════════════════╝

import styles from './ProgressBar.module.css'

interface ProgressBarProps {
  value: number
  label?: string
  indeterminate?: boolean
  showValue?: boolean
}

export function ProgressBar({
  value,
  label,
  indeterminate = false,
  showValue = true,
}: ProgressBarProps) {
  const clamped = Math.max(0, Math.min(100, value))

  return (
    <div>
      {(label || showValue) && (
        <div className={styles.meta}>
          <span>{label}</span>
          {showValue && !indeterminate && <span className={styles.value}>{clamped}%</span>}
        </div>
      )}
      <div
        className={[styles.track, indeterminate ? styles.indeterminate : ''].join(' ')}
        role="progressbar"
        aria-valuenow={indeterminate ? undefined : clamped}
        aria-valuemin={0}
        aria-valuemax={100}
      >
        <div className={styles.fill} style={indeterminate ? undefined : { width: `${clamped}%` }} />
      </div>
    </div>
  )
}
