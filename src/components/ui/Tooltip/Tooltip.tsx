// ╔════════════════════════════════════════════════════╗
// ║          components/ui/Tooltip/Tooltip           ║
// ╚════════════════════════════════════════════════════╝

import type { ReactNode } from 'react'
import styles from './Tooltip.module.css'

interface TooltipProps {
  content: string
  children: ReactNode
}

export function Tooltip({ content, children }: TooltipProps) {
  return (
    <span className={styles.wrap}>
      {children}
      <span className={styles.tip} role="tooltip">
        {content}
      </span>
    </span>
  )
}
