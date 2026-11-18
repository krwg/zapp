// ╔════════════════════════════════════════════════════╗
// ║ components/domain/TelegramProxyCard/TelegramProxyCard ║
// ╚════════════════════════════════════════════════════╝

import { Copy, ExternalLink, Power } from 'lucide-react'
import { useEffect, useState } from 'react'
import { Button } from '@/components/ui/Button/Button'
import { Dialog } from '@/components/ui/Dialog/Dialog'
import { useLocale } from '@/context/LocaleContext'
import type { TelegramProxyState } from '@/types'
import { formatUptime } from '@/utils/format'
import styles from './TelegramProxyCard.module.css'

interface TelegramProxyCardProps {
  telegram: TelegramProxyState
  busy?: boolean
  offerLink?: boolean
  onToggle: () => void
  onOpenTelegram?: () => void
  onCopyLink?: () => void
  onDismissLinkOffer?: () => void
}

export function TelegramProxyCard({
  telegram,
  busy,
  offerLink,
  onToggle,
  onOpenTelegram,
  onCopyLink,
  onDismissLinkOffer,
}: TelegramProxyCardProps) {
  const { t, locale } = useLocale()
  const [tick, setTick] = useState(() => Date.now())
  const [linkOpen, setLinkOpen] = useState(false)

  const isOn = telegram.status === 'running' || telegram.status === 'starting'
  const isBusy =
    busy || telegram.status === 'starting' || telegram.status === 'stopping'

  useEffect(() => {
    if (telegram.status !== 'running') return
    const id = window.setInterval(() => setTick(Date.now()), 1000)
    return () => window.clearInterval(id)
  }, [telegram.status])

  useEffect(() => {
    if (offerLink && telegram.status === 'running') {
      setLinkOpen(true)
    }
  }, [offerLink, telegram.status])

  const uptimeSeconds =
    telegram.status === 'running' && telegram.sessionStartedAt
      ? Math.max(
          0,
          Math.floor((tick - new Date(telegram.sessionStartedAt).getTime()) / 1000),
        )
      : telegram.uptimeSeconds

  const statusTitle = (() => {
    switch (telegram.status) {
      case 'running':
        return t('status.running')
      case 'stopped':
        return t('status.stopped')
      case 'starting':
        return t('status.starting')
      case 'stopping':
        return t('status.stopping')
      case 'error':
        return t('status.error')
    }
  })()

  const closeLink = () => {
    setLinkOpen(false)
    onDismissLinkOffer?.()
  }

  return (
    <>
      <section className={[styles.card, isOn ? styles.cardOn : ''].join(' ')}>
        <div className={styles.hero}>
          <button
            type="button"
            className={[
              styles.power,
              isOn ? styles.powerOn : styles.powerOff,
              isBusy ? styles.powerBusy : '',
            ].join(' ')}
            onClick={onToggle}
            disabled={isBusy}
            aria-pressed={isOn}
            aria-label={isOn ? t('telegram.disable') : t('telegram.enable')}
          >
            <span className={styles.powerRing} aria-hidden />
            <span className={styles.powerCore}>
              <Power size={24} strokeWidth={2.25} />
            </span>
          </button>

          <div className={styles.heroCopy}>
            <div className={styles.label}>{t('telegram.label')}</div>
            <h2 className={styles.statusText}>{statusTitle}</h2>
            <p className={styles.hint}>
              {telegram.installed
                ? isOn
                  ? t('telegram.onHint')
                  : t('telegram.offHint')
                : t('telegram.notInstalled')}
              {' · '}
              {t('telegram.separateFromBypass')}
            </p>

            {telegram.status === 'running' && (
              <div className={styles.session}>
                <span className={styles.sessionLabel}>{t('home.session')}</span>
                <span className={styles.sessionValue}>
                  {formatUptime(uptimeSeconds, { withSeconds: true, locale })}
                </span>
                <span className={styles.port}>
                  {telegram.host}:{telegram.port}
                </span>
              </div>
            )}
          </div>

          {telegram.status === 'running' && (
            <div className={styles.actions}>
              <button
                type="button"
                className={styles.actionBtn}
                onClick={onOpenTelegram}
                title={t('telegram.openInTelegram')}
              >
                <ExternalLink size={14} />
                {t('telegram.open')}
              </button>
              <button
                type="button"
                className={styles.actionBtn}
                onClick={onCopyLink}
                title={t('telegram.copyLink')}
              >
                <Copy size={14} />
                {t('telegram.copy')}
              </button>
            </div>
          )}
        </div>

        {telegram.lastError && (
          <div className={styles.errorBox} role="alert">
            <div className={styles.errorTitle}>{t('connection.lastError')}</div>
            <div className={styles.errorText}>{telegram.lastError}</div>
          </div>
        )}
      </section>

      <Dialog
        open={linkOpen}
        title={t('telegram.linkTitle')}
        description={t('telegram.linkBody')}
        onClose={closeLink}
        footer={
          <>
            <Button variant="ghost" onClick={closeLink}>
              {t('common.close')}
            </Button>
            <Button
              variant="secondary"
              onClick={() => {
                onCopyLink?.()
              }}
            >
              <Copy size={14} />
              {t('telegram.copyLink')}
            </Button>
            <Button
              variant="primary"
              onClick={() => {
                onOpenTelegram?.()
                closeLink()
              }}
            >
              <ExternalLink size={14} />
              {t('telegram.openInTelegram')}
            </Button>
          </>
        }
      />
    </>
  )
}
