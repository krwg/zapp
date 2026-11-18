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
import { displaySiteStatus, isSiteCheckFresh, siteStatusLabelKey } from '@/utils/siteStatus'
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

  const labelKey = siteStatusLabelKey(service, engineStatus)
  const status = ((): { label: string; tone: StatusTone; pulse?: boolean } => {
    switch (labelKey) {
      case 'status.available':
        return { label: t(labelKey), tone: 'success' }
      case 'status.unavailable':
        return { label: t(labelKey), tone: 'error' }
      case 'status.checking':
        return { label: t(labelKey), tone: 'warning', pulse: true }
      default:
        return { label: t(labelKey), tone: 'neutral' }
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
            : t(labelKey)}
        </div>
        <Button size="sm" variant="ghost" onClick={onCheck} disabled={busy || service.status === 'checking'}>
          <RefreshCw size={14} />
          {t('common.check')}
        </Button>
      </div>
    </article>
  )
}
