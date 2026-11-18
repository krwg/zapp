// ╔════════════════════════════════════════════════════╗
// ║     components/domain/UpdateCard/UpdateCard      ║
// ╚════════════════════════════════════════════════════╝

import { useState } from 'react'
import { ChevronDown, Download } from 'lucide-react'
import { Button } from '@/components/ui/Button/Button'
import { Markdown, markdownPreview } from '@/components/ui/Markdown/Markdown'
import { ProgressBar } from '@/components/ui/ProgressBar/ProgressBar'
import { StatusBadge, type StatusTone } from '@/components/ui/StatusBadge/StatusBadge'
import { useLocale } from '@/context/LocaleContext'
import type { MessageKey } from '@/i18n'
import type { UpdateChannel, UpdateChannelStatus } from '@/types'
import { formatRelative } from '@/utils/format'
import { formatVersion } from '@/utils/version'
import styles from './UpdateCard.module.css'

interface UpdateCardProps {
  channel: UpdateChannel
  busy?: boolean
  onUpdate: () => void
}

function channelCopy(
  id: UpdateChannel['id'],
  t: (key: MessageKey) => string,
): { title: string; description: string } {
  switch (id) {
    case 'application':
      return {
        title: t('updates.channels.application'),
        description: t('updates.channels.applicationDesc'),
      }
    case 'engine':
      return {
        title: t('updates.channels.engine'),
        description: t('updates.channels.engineDesc'),
      }
    case 'strategies':
      return {
        title: t('updates.channels.strategies'),
        description: t('updates.channels.strategiesDesc'),
      }
    case 'telegramProxy':
      return {
        title: t('updates.channels.telegramProxy'),
        description: t('updates.channels.telegramProxyDesc'),
      }
  }
}

function statusMeta(
  status: UpdateChannelStatus,
  t: (key: MessageKey) => string,
): { label: string; tone: StatusTone; pulse?: boolean } {
  switch (status) {
    case 'up_to_date':
      return { label: t('status.upToDate'), tone: 'success' }
    case 'available':
      return { label: t('status.updateAvailable'), tone: 'warning' }
    case 'downloading':
      return { label: t('status.downloading'), tone: 'neutral', pulse: true }
    case 'installing':
      return { label: t('status.installing'), tone: 'neutral', pulse: true }
    case 'error':
      return { label: t('status.error'), tone: 'error' }
  }
}

function displayVer(id: UpdateChannel['id'], version: string): string {
  return id === 'application' ? formatVersion(version) : version
}

export function UpdateCard({ channel, busy, onUpdate }: UpdateCardProps) {
  const { t, locale } = useLocale()
  const [openNotes, setOpenNotes] = useState(false)
  const meta = statusMeta(channel.status, t)
  const copy = channelCopy(channel.id, t)
  const inProgress = channel.status === 'downloading' || channel.status === 'installing'
  const hasNotes = Boolean(channel.releaseNotes?.trim())

  return (
    <section className={styles.card}>
      <div className={styles.header}>
        <div>
          <h3 className={styles.title}>{copy.title}</h3>
          <p className={styles.description}>{copy.description}</p>
        </div>
        <StatusBadge label={meta.label} tone={meta.tone} pulse={meta.pulse} />
      </div>

      <div className={styles.versions}>
        <div className={styles.box}>
          <div className={styles.boxLabel}>{t('updates.current')}</div>
          <div className={styles.boxValue}>{displayVer(channel.id, channel.currentVersion)}</div>
        </div>
        <div className={styles.box}>
          <div className={styles.boxLabel}>{t('updates.latest')}</div>
          <div className={styles.boxValue}>{displayVer(channel.id, channel.latestVersion)}</div>
        </div>
      </div>

      {hasNotes && (
        <div className={styles.notesBlock}>
          <button
            type="button"
            className={styles.notesToggle}
            onClick={() => setOpenNotes((v) => !v)}
            aria-expanded={openNotes}
          >
            <span>{t('updates.releaseNotes')}</span>
            <ChevronDown
              size={16}
              className={[styles.chevron, openNotes ? styles.chevronOpen : ''].join(' ')}
            />
          </button>
          {openNotes ? (
            <div className={styles.notesBody}>
              <Markdown source={channel.releaseNotes} />
            </div>
          ) : (
            <p className={styles.notesPreview}>{markdownPreview(channel.releaseNotes)}</p>
          )}
        </div>
      )}

      {!hasNotes && <p className={styles.notesPreview}>{t('updates.noNotes')}</p>}

      {inProgress && (
        <ProgressBar
          value={channel.downloadProgress ?? 0}
          label={
            channel.status === 'installing' ? t('updates.installing') : t('updates.downloading')
          }
          indeterminate={channel.status === 'installing'}
        />
      )}

      <div className={styles.footer}>
        <span className={styles.checked}>
          {t('updates.checked')} {formatRelative(channel.lastCheckedAt, locale)}
        </span>
        <Button
          size="sm"
          variant={channel.status === 'available' ? 'primary' : 'secondary'}
          onClick={onUpdate}
          disabled={busy || channel.status === 'up_to_date' || inProgress}
        >
          <Download size={14} />
          {t('updates.update')}
        </Button>
      </div>
    </section>
  )
}
