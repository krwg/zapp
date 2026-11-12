// ╔════════════════════════════════════════════════════╗
// ║    components/domain/ServiceCard/ServiceCard     ║
// ╚════════════════════════════════════════════════════╝

import { RefreshCw } from 'lucide-react'
import { Button } from '@/components/ui/Button/Button'
import { StatusBadge, type StatusTone } from '@/components/ui/StatusBadge/StatusBadge'
import type { AvailabilityStatus, MonitoredService } from '@/types'
import { formatLatency, formatRelative } from '@/utils/format'
import styles from './ServiceCard.module.css'

interface ServiceCardProps {
  service: MonitoredService
  busy?: boolean
  onCheck: () => void
  compact?: boolean
}

function toneFor(status: AvailabilityStatus): { label: string; tone: StatusTone; pulse?: boolean } {
  switch (status) {
    case 'available':
      return { label: 'Available', tone: 'success' }
    case 'unavailable':
      return { label: 'Unavailable', tone: 'error' }
    case 'checking':
      return { label: 'Checking', tone: 'warning', pulse: true }
    case 'unknown':
      return { label: 'Unknown', tone: 'neutral' }
  }
}

export function ServiceCard({ service, busy, onCheck, compact }: ServiceCardProps) {
  const status = toneFor(service.status)

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
          {formatLatency(service.latencyMs)} · {formatRelative(service.lastCheckedAt)}
        </div>
        <Button size="sm" variant="ghost" onClick={onCheck} disabled={busy || service.status === 'checking'}>
          <RefreshCw size={14} />
          Check
        </Button>
      </div>
    </article>
  )
}
