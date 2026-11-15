// ╔════════════════════════════════════════════════════╗
// ║       components/ui/BootLoader/BootLoader        ║
// ╚════════════════════════════════════════════════════╝

import styles from './BootLoader.module.css'

export function BootLoader() {
  return (
    <div className={styles.root} role="status" aria-live="polite" aria-label="zapp">
      <div className={styles.mark} aria-hidden>
        <span className={styles.ring} />
        <span className={styles.ringDelay} />
        <span className={styles.core} />
      </div>
      <div className={styles.copy}>
        <div className={styles.brand}>zapp</div>
        <div className={styles.bar}>
          <span className={styles.barFill} />
        </div>
      </div>
    </div>
  )
}
