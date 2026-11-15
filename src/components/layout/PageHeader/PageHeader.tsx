// ╔════════════════════════════════════════════════════╗
// ║     components/layout/PageHeader/PageHeader      ║
// ╚════════════════════════════════════════════════════╝

import type { ReactNode } from 'react'
import styles from './PageHeader.module.css'

interface PageHeaderProps {
  title: string
  description?: string
  eyebrow?: string
  titleAddon?: ReactNode
  actions?: ReactNode
}

export function PageHeader({
  title,
  description,
  eyebrow,
  titleAddon,
  actions,
}: PageHeaderProps) {
  return (
    <header className={styles.header}>
      <div className={styles.copy}>
        {eyebrow && <p className={styles.eyebrow}>{eyebrow}</p>}
        <div className={styles.titleRow}>
          <h1 className={styles.title}>{title}</h1>
          {titleAddon}
        </div>
        {description && <p className={styles.description}>{description}</p>}
      </div>
      {actions && <div className={styles.actions}>{actions}</div>}
    </header>
  )
}
