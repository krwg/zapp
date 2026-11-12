// ╔════════════════════════════════════════════════════╗
// ║  components/domain/SettingsGroup/SettingsGroup   ║
// ╚════════════════════════════════════════════════════╝

import type { ReactNode } from 'react'
import styles from './SettingsGroup.module.css'

interface SettingsGroupProps {
  id?: string
  title: string
  description?: string
  children: ReactNode
}

export function SettingsGroup({ id, title, description, children }: SettingsGroupProps) {
  return (
    <section id={id} className={styles.group}>
      <div className={styles.header}>
        <h2 className={styles.title}>{title}</h2>
        {description && <p className={styles.description}>{description}</p>}
      </div>
      <div className={styles.body}>{children}</div>
    </section>
  )
}

interface SettingsRowProps {
  title: string
  hint?: string
  children: ReactNode
}

export function SettingsRow({ title, hint, children }: SettingsRowProps) {
  return (
    <div className={styles.row}>
      <div className={styles.rowCopy}>
        <div className={styles.rowTitle}>{title}</div>
        {hint && <div className={styles.rowHint}>{hint}</div>}
      </div>
      <div className={styles.control}>{children}</div>
    </div>
  )
}

export const settingsStyles = styles
