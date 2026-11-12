// ╔════════════════════════════════════════════════════╗
// ║     components/domain/UpdateCard/UpdateCard      ║
// ╚════════════════════════════════════════════════════╝

import { Download } from 'lucide-react'
import { Button } from '@/components/ui/Button/Button'
import { ProgressBar } from '@/components/ui/ProgressBar/ProgressBar'
import { StatusBadge, type StatusTone } from '@/components/ui/StatusBadge/StatusBadge'
import type { UpdateChannel, UpdateChannelStatus } from '@/types'
import { formatRelative } from '@/utils/format'
import styles from './UpdateCard.module.css'

interface UpdateCardProps {
  channel: UpdateChannel
  busy?: boolean
  onUpdate: () => void
}

function statusMeta(status: UpdateChannelStatus): { label: string; tone: StatusTone; pulse?: boolean } {
  switch (status) {
    case 'up_to_date':
      return { label: 'Up to date', tone: 'success' }
    case 'available':
      return { label: 'Update available', tone: 'warning' }
    case 'downloading':
      return { label: 'Downloading', tone: 'neutral', pulse: true }
    case 'installing':
      return { label: 'Installing', tone: 'neutral', pulse: true }
    case 'error':
      return { label: 'Error', tone: 'error' }
  }
}

export function UpdateCard({ channel, busy, onUpdate }: UpdateCardProps) {
  const meta = statusMeta(channel.status)
  const inProgress = channel.status === 'downloading' || channel.status === 'installing'

  return (
    <section className={styles.card}>
      <div className={styles.header}>
        <div>
          <h3 className={styles.title}>{channel.title}</h3>
          <p className={styles.description}>{channel.description}</p>
        </div>
        <StatusBadge label={meta.label} tone={meta.tone} pulse={meta.pulse} />
      </div>

      <div className={styles.versions}>
        <div className={styles.box}>
          <div className={styles.boxLabel}>Current</div>
          <div className={styles.boxValue}>{channel.currentVersion}</div>
        </div>
        <div className={styles.box}>
          <div className={styles.boxLabel}>Latest</div>
          <div className={styles.boxValue}>{channel.latestVersion}</div>
        </div>
      </div>

      <p className={styles.notes}>{channel.releaseNotes}</p>

      {inProgress && (
        <ProgressBar
          value={channel.downloadProgress ?? 0}
          label={channel.status === 'installing' ? 'Installing…' : 'Downloading…'}
          indeterminate={channel.status === 'installing'}
        />
      )}

      <div className={styles.footer}>
        <span className={styles.checked}>Checked {formatRelative(channel.lastCheckedAt)}</span>
        <Button
          size="sm"
          variant={channel.status === 'available' ? 'primary' : 'secondary'}
          onClick={onUpdate}
          disabled={busy || channel.status === 'up_to_date' || inProgress}
        >
          <Download size={14} />
          Update
        </Button>
      </div>
    </section>
  )
}
