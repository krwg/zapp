// ╔════════════════════════════════════════════════════╗
// ║ components/domain/ConnectionCard/ConnectionCard  ║
// ╚════════════════════════════════════════════════════╝

import { Power } from 'lucide-react'
import { Button } from '@/components/ui/Button/Button'
import { StatusBadge } from '@/components/ui/StatusBadge/StatusBadge'
import type { ConnectionState } from '@/types'
import { formatUptime } from '@/utils/format'
import styles from './ConnectionCard.module.css'

interface ConnectionCardProps {
  connection: ConnectionState
  busy?: boolean
  onToggle: () => void
  onReset?: () => void
}

function mapStatus(status: ConnectionState['status']) {
  switch (status) {
    case 'running':
      return { label: 'Connected', tone: 'success' as const, title: 'Running', pulse: true }
    case 'stopped':
      return { label: 'Stopped', tone: 'neutral' as const, title: 'Stopped', pulse: false }
    case 'starting':
      return { label: 'Starting', tone: 'warning' as const, title: 'Starting…', pulse: true }
    case 'stopping':
      return { label: 'Stopping', tone: 'warning' as const, title: 'Stopping…', pulse: true }
    case 'error':
      return { label: 'Error', tone: 'error' as const, title: 'Error', pulse: false }
  }
}

export function ConnectionCard({ connection, busy, onToggle, onReset }: ConnectionCardProps) {
  const mapped = mapStatus(connection.status)
  const isOn = connection.status === 'running' || connection.status === 'starting'

  return (
    <section className={styles.card}>
      <div className={styles.top}>
        <div>
          <div className={styles.label}>Connection Status</div>
          <div className={styles.statusRow}>
            <h2 className={styles.statusText}>{mapped.title}</h2>
            <StatusBadge label={mapped.label} tone={mapped.tone} pulse={mapped.pulse} />
          </div>
        </div>
        <div className={styles.actions}>
          {onReset && (
            <Button
              variant="ghost"
              onClick={onReset}
              disabled={busy || connection.status === 'starting' || connection.status === 'stopping'}
            >
              Reset
            </Button>
          )}
          <Button
            variant={isOn ? 'secondary' : 'primary'}
            onClick={onToggle}
            disabled={busy || connection.status === 'starting' || connection.status === 'stopping'}
          >
            <Power size={15} />
            {isOn ? 'Stop' : 'Start'}
          </Button>
        </div>
      </div>

      <div className={styles.meta}>
        <div className={styles.metaItem}>
          <div className={styles.metaLabel}>Active strategy</div>
          <div className={styles.metaValue}>{connection.strategyName ?? '—'}</div>
        </div>
        <div className={styles.metaItem}>
          <div className={styles.metaLabel}>Session uptime</div>
          <div className={styles.metaValue}>
            {connection.status === 'running' ? formatUptime(connection.uptimeSeconds) : '—'}
          </div>
        </div>
      </div>

      {connection.lastError && (
        <div className={styles.errorBox} role="alert">
          <div className={styles.errorTitle}>Последняя ошибка</div>
          <div className={styles.errorText}>{connection.lastError}</div>
        </div>
      )}
    </section>
  )
}
