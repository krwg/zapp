// ╔════════════════════════════════════════════════════╗
// ║              components/ui/Hint/Hint             ║
// ╚════════════════════════════════════════════════════╝

import { CircleHelp } from 'lucide-react'
import { useEffect, useId, useRef, useState, type ReactNode } from 'react'
import styles from './Hint.module.css'

interface HintProps {
  title?: string
  body: ReactNode
  label?: string
  wide?: boolean
}

export function Hint({ title, body, label = 'Подсказка', wide }: HintProps) {
  const [open, setOpen] = useState(false)
  const id = useId()
  const rootRef = useRef<HTMLSpanElement>(null)

  useEffect(() => {
    if (!open) return
    const onDoc = (e: MouseEvent) => {
      if (!rootRef.current?.contains(e.target as Node)) setOpen(false)
    }
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setOpen(false)
    }
    document.addEventListener('mousedown', onDoc)
    document.addEventListener('keydown', onKey)
    return () => {
      document.removeEventListener('mousedown', onDoc)
      document.removeEventListener('keydown', onKey)
    }
  }, [open])

  return (
    <span
      ref={rootRef}
      className={[styles.wrap, open ? styles.open : ''].join(' ')}
      onMouseEnter={() => setOpen(true)}
      onMouseLeave={() => setOpen(false)}
    >
      <button
        type="button"
        className={styles.trigger}
        aria-label={label}
        aria-expanded={open}
        aria-controls={id}
        onClick={() => setOpen((v) => !v)}
      >
        <CircleHelp size={16} strokeWidth={1.75} />
      </button>
      <span
        id={id}
        role="tooltip"
        className={[styles.tip, wide ? styles.wide : ''].join(' ')}
      >
        {title ? <span className={styles.title}>{title}</span> : null}
        <span className={styles.body}>{body}</span>
      </span>
    </span>
  )
}
