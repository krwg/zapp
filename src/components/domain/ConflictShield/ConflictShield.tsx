// ╔════════════════════════════════════════════════════╗
// ║   components/domain/ConflictShield               ║
// ╚════════════════════════════════════════════════════╝

import { Button } from '@/components/ui/Button/Button'
import { useLocale } from '@/context/LocaleContext'
import type { ConflictInfo } from '@/types'
import styles from './ConflictShield.module.css'

interface ConflictShieldProps {
  conflict: ConflictInfo
  busy?: boolean
  onResolve: () => void
  onDismiss?: () => void
}

export function ConflictShield({
  conflict,
  busy = false,
  onResolve,
  onDismiss,
}: ConflictShieldProps) {
  const { t } = useLocale()
  const images = conflict.images.filter(Boolean)

  return (
    <section className={styles.card} role="alert">
      <div className={styles.copy}>
        <div className={styles.title}>{t('home.conflictTitle')}</div>
        <p className={styles.body}>
          {conflict.detail || t('home.conflictBody')}
        </p>
        {images.length > 0 && (
          <ul className={styles.images}>
            {images.map((img) => (
              <li key={img}>{img}</li>
            ))}
          </ul>
        )}
      </div>
      <div className={styles.actions}>
        <Button variant="danger" size="sm" disabled={busy} onClick={onResolve}>
          {t('home.conflictKill')}
        </Button>
        {onDismiss && (
          <Button variant="ghost" size="sm" disabled={busy} onClick={onDismiss}>
            {t('home.conflictDismiss')}
          </Button>
        )}
      </div>
    </section>
  )
}
