// ╔════════════════════════════════════════════════════╗
// ║    components/domain/ServiceCard/ServiceCard     ║
// ╚════════════════════════════════════════════════════╝

import { RefreshCw } from 'lucide-react'
import { Button } from '@/components/ui/Button/Button'
import { StatusBadge, type StatusTone } from '@/components/ui/StatusBadge/StatusBadge'
import { useAppState } from '@/context/AppStateContext'
import { useLocale } from '@/context/LocaleContext'
import type { MonitoredService } from '@/types'
import { formatLatency, formatRelative } from '@/utils/format'
import { displaySiteStatus, isSiteCheckFresh } from '@/utils/siteStatus'
import styles from './ServiceCard.module.css'

interface ServiceCardProps {
  service: MonitoredService
  busy?: boolean
  onCheck: () => void
  compact?: boolean
}

export function ServiceCard({ service, busy, onCheck, compact }: ServiceCardProps) {
  const { t, locale } = useLocale()
  const { snapshot } = useAppState()
  const engineStatus = snapshot?.connection.status ?? 'stopped'
  const shown = displaySiteStatus(service, engineStatus)
  const fresh = engineStatus === 'running' && isSiteCheckFresh(service.lastCheckedAt)

  const status = ((): { label: string; tone: StatusTone; pulse?: boolean } => {
    switch (shown) {
      case 'available':
        return { label: t('status.available'), tone: 'success' }
      case 'unavailable':
        return { label: t('status.unavailable'), tone: 'error' }
      case 'checking':
        return { label: t('status.checking'), tone: 'warning', pulse: true }
      default:
        return { label: t('status.notChecked'), tone: 'neutral' }
    }
  })()

  return (
    <article className={[styles.card, shown === 'unknown' ? styles.stale : ''].join(' ')}>
      <div className={styles.header}>
        <div>
          <h3 className={styles.name}>{service.name}</h3>
          <p className={styles.host}>{service.host}</p>
        </div>
        <StatusBadge label={status.label} tone={status.tone} pulse={status.pulse} />
      </div>

      {!compact && shown !== 'unknown' && (
        <p className={styles.summary}>{service.diagnosticSummary}</p>
      )}

      <div className={styles.footer}>
        <div className={styles.meta}>
          {fresh
            ? `${formatLatency(service.latencyMs)} · ${formatRelative(service.lastCheckedAt, locale)}`
            : t('status.notChecked')}
        </div>
        <Button size="sm" variant="ghost" onClick={onCheck} disabled={busy || service.status === 'checking'}>
          <RefreshCw size={14} />
          {t('common.check')}
        </Button>
      </div>
    </article>
  )
}
