// ╔════════════════════════════════════════════════════╗
// ║    components/domain/ServiceCard/ServiceCard     ║
// ╚════════════════════════════════════════════════════╝

import { RefreshCw } from 'lucide-react'
import { Button } from '@/components/ui/Button/Button'
import { StatusBadge, type StatusTone } from '@/components/ui/StatusBadge/StatusBadge'
import { useLocale } from '@/context/LocaleContext'
import type { MonitoredService } from '@/types'
import { formatLatency, formatRelative } from '@/utils/format'
import styles from './ServiceCard.module.css'

interface ServiceCardProps {
  service: MonitoredService
  busy?: boolean
  onCheck: () => void
  compact?: boolean
}

export function ServiceCard({ service, busy, onCheck, compact }: ServiceCardProps) {
  const { t, locale } = useLocale()

  const status = ((): { label: string; tone: StatusTone; pulse?: boolean } => {
    switch (service.status) {
      case 'available':
        return { label: t('status.available'), tone: 'success' }
      case 'unavailable':
        return { label: t('status.unavailable'), tone: 'error' }
      case 'checking':
        return { label: t('status.checking'), tone: 'warning', pulse: true }
      case 'unknown':
        return { label: t('status.unknown'), tone: 'neutral' }
    }
  })()

  return (
    <article className={styles.card}>
      <div className={styles.header}>
        <div>
          <h3 className={styles.name}>{service.name}</h3>
          <p className={styles.host}>{service.host}</p>
        </div>
        <StatusBadge label={status.label} tone={status.tone} pulse={status.pulse} />
      </div>

      {!compact && <p className={styles.summary}>{service.diagnosticSummary}</p>}

      <div className={styles.footer}>
        <div className={styles.meta}>
          {formatLatency(service.latencyMs)} · {formatRelative(service.lastCheckedAt, locale)}
        </div>
        <Button size="sm" variant="ghost" onClick={onCheck} disabled={busy || service.status === 'checking'}>
          <RefreshCw size={14} />
          {t('common.check')}
        </Button>
      </div>
    </article>
  )
}
