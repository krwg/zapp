// ╔════════════════════════════════════════════════════╗
// ║           components/ui/Toggle/Toggle            ║
// ╚════════════════════════════════════════════════════╝

import { useId } from 'react'
import styles from './Toggle.module.css'

interface ToggleProps {
  checked: boolean
  onChange: (checked: boolean) => void
  label?: string
  hint?: string
  disabled?: boolean
  id?: string
}

export function Toggle({ checked, onChange, label, hint, disabled, id }: ToggleProps) {
  const autoId = useId()
  const inputId = id ?? autoId

  return (
    <label className={styles.root} htmlFor={inputId}>
      <input
        id={inputId}
        className={styles.input}
        type="checkbox"
        role="switch"
        checked={checked}
        disabled={disabled}
        onChange={(e) => onChange(e.target.checked)}
      />
      <span className={styles.track} aria-hidden>
        <span className={styles.thumb} />
      </span>
      {(label || hint) && (
        <span>
          {label && <span className={styles.label}>{label}</span>}
          {hint && <span className={styles.hint}>{hint}</span>}
        </span>
      )}
    </label>
  )
}
