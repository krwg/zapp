// ╔════════════════════════════════════════════════════╗
// ║   components/domain/PathVerdictStrip             ║
// ╚════════════════════════════════════════════════════╝

import { StatusBadge, type StatusTone } from '@/components/ui/StatusBadge/StatusBadge'
import { useLocale } from '@/context/LocaleContext'
import type { MessageKey } from '@/i18n'
import type { AvailabilityStatus, PathVerdict } from '@/types'
import styles from './PathVerdictStrip.module.css'

const STATUS_KEY: Record<AvailabilityStatus, MessageKey> = {
  available: 'status.available',
  unavailable: 'status.unavailable',
  checking: 'status.checking',
  unknown: 'status.unknown',
}

interface PathVerdictStripProps {
  verdict: PathVerdict
  compact?: boolean
}

function toneFor(status: AvailabilityStatus): StatusTone {
  switch (status) {
    case 'available':
      return 'success'
    case 'unavailable':
      return 'error'
    case 'checking':
      return 'warning'
    default:
      return 'neutral'
  }
}

export function PathVerdictStrip({ verdict, compact = false }: PathVerdictStripProps) {
  const { t } = useLocale()

  const rows: { key: keyof PathVerdict; label: string; status: AvailabilityStatus }[] = [
    { key: 'youtube', label: t('home.pathYoutube'), status: verdict.youtube },
    { key: 'discordWeb', label: t('home.pathDiscordWeb'), status: verdict.discordWeb },
    { key: 'discordVoice', label: t('home.pathDiscordVoice'), status: verdict.discordVoice },
  ]

  const voiceDead =
    verdict.discordVoice === 'unavailable' &&
    (verdict.youtube === 'available' || verdict.discordWeb === 'available')

  return (
    <section className={[styles.strip, compact ? styles.compact : ''].join(' ')} aria-label={t('home.pathVerdictTitle')}>
      {!compact && <div className={styles.title}>{t('home.pathVerdictTitle')}</div>}
      <div className={styles.row}>
        {rows.map((r) => (
          <div key={r.key} className={styles.cell}>
            <span className={styles.label}>{r.label}</span>
            <StatusBadge
              label={t(STATUS_KEY[r.status])}
              tone={toneFor(r.status)}
              pulse={r.status === 'checking'}
            />
          </div>
        ))}
      </div>
      {voiceDead && !compact && (
        <p className={styles.hint}>{t('home.voiceHonesty')}</p>
      )}
    </section>
  )
}
