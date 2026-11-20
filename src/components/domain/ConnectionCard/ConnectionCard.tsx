// ╔════════════════════════════════════════════════════╗
// ║ components/domain/ConnectionCard/ConnectionCard  ║
// ╚════════════════════════════════════════════════════╝

import { Power } from 'lucide-react'
import { useEffect, useState } from 'react'
import { Dialog } from '@/components/ui/Dialog/Dialog'
import { Button } from '@/components/ui/Button/Button'
import { useLocale } from '@/context/LocaleContext'
import { useUiPrefs } from '@/context/UiPrefsContext'
import type { ConnectionState } from '@/types'
import { formatUptime } from '@/utils/format'
import styles from './ConnectionCard.module.css'

interface ConnectionCardProps {
  connection: ConnectionState
  busy?: boolean
  onToggle: () => void
  onReset?: () => void
  onReportBug?: () => void
}

export function ConnectionCard({
  connection,
  busy,
  onToggle,
  onReset,
  onReportBug,
}: ConnectionCardProps) {
  const { t, locale } = useLocale()
  const { confirmDisableProtection } = useUiPrefs()
  const [tick, setTick] = useState(() => Date.now())
  const [confirmOpen, setConfirmOpen] = useState(false)

  const isOn = connection.status === 'running' || connection.status === 'starting'
  const isBusy =
    busy || connection.status === 'starting' || connection.status === 'stopping'

  useEffect(() => {
    if (connection.status !== 'running') return
    const id = window.setInterval(() => setTick(Date.now()), 1000)
    return () => window.clearInterval(id)
  }, [connection.status])

  const uptimeSeconds =
    connection.status === 'running' && connection.sessionStartedAt
      ? Math.max(
          0,
          Math.floor((tick - new Date(connection.sessionStartedAt).getTime()) / 1000),
        )
      : connection.uptimeSeconds

  const statusTitle = (() => {
    switch (connection.status) {
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

  const handleToggle = () => {
    if (isOn && confirmDisableProtection) {
      setConfirmOpen(true)
      return
    }
    onToggle()
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
            onClick={handleToggle}
            disabled={isBusy}
            aria-pressed={isOn}
            aria-label={isOn ? t('connection.disable') : t('connection.enable')}
          >
            <span className={styles.powerRing} aria-hidden />
            <span className={styles.powerCore}>
              <Power size={28} strokeWidth={2.25} />
            </span>
          </button>

          <div className={styles.heroCopy}>
            <div className={styles.label}>{t('connection.label')}</div>
            <h2 className={styles.statusText}>{statusTitle}</h2>
            <p className={styles.hint}>
              {isOn ? t('home.protectionOnHint') : t('home.protectionOffHint')}
              {' · '}
              {isOn ? t('home.tapToDisable') : t('home.tapToEnable')}
            </p>

            {connection.status === 'running' && (
              <div className={styles.session}>
                <span className={styles.sessionLabel}>{t('home.session')}</span>
                <span className={styles.sessionValue}>
                  {formatUptime(uptimeSeconds, { withSeconds: true, locale })}
                </span>
              </div>
            )}
          </div>

          {onReset && (
            <button
              type="button"
              className={styles.repair}
              onClick={onReset}
              disabled={isBusy}
            >
              {t('connection.repair')}
            </button>
          )}
        </div>

        {connection.lastError && (
          <div className={styles.errorBox} role="alert">
            <div className={styles.errorTitle}>{t('connection.lastError')}</div>
            <div className={styles.errorText}>{connection.lastError}</div>
            {onReportBug && (
              <button
                type="button"
                className={styles.errorReport}
                onClick={onReportBug}
              >
                {t('connection.reportFromError')}
              </button>
            )}
          </div>
        )}
      </section>

      <Dialog
        open={confirmOpen}
        title={t('settings.confirmDisableTitle')}
        description={t('settings.confirmDisableBody')}
        onClose={() => setConfirmOpen(false)}
        footer={
          <>
            <Button variant="ghost" onClick={() => setConfirmOpen(false)}>
              {t('common.cancel')}
            </Button>
            <Button
              variant="primary"
              onClick={() => {
                setConfirmOpen(false)
                onToggle()
              }}
            >
              {t('common.confirm')}
            </Button>
          </>
        }
      />
    </>
  )
}
