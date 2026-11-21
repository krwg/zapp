// ╔════════════════════════════════════════════════════╗
// ║   components/domain/FirstRunWizard               ║
// ╚════════════════════════════════════════════════════╝

import { useMemo, useState } from 'react'
import { Button } from '@/components/ui/Button/Button'
import { useAppState } from '@/context/AppStateContext'
import { useLocale } from '@/context/LocaleContext'
import styles from './FirstRunWizard.module.css'

type Step = 'engine' | 'uac' | 'smart' | 'done'

export function FirstRunWizard() {
  const {
    snapshot,
    updatesLive,
    updateChannel,
    toggleEngine,
    startDiagnostics,
    updateSettings,
    busy,
  } = useAppState()
  const { t } = useLocale()
  const [busyLocal, setBusyLocal] = useState(false)
  const [localError, setLocalError] = useState<string | null>(null)
  const [uacTried, setUacTried] = useState(false)

  const engine = updatesLive.find((c) => c.id === 'engine')
  const engineOk = !!snapshot?.system.engineInstalled
  const running = snapshot?.connection.status === 'running'
  const diagDone =
    snapshot?.diagnostics.phase === 'completed' ||
    (snapshot?.diagnostics.results?.length ?? 0) > 0
  const lastError = snapshot?.connection.lastError ?? localError

  const step: Step = useMemo(() => {
    if (!engineOk) return 'engine'
    if (!running && !diagDone) return 'uac'
    if (!diagDone) return 'smart'
    return 'done'
  }, [diagDone, engineOk, running])

  if (!snapshot || snapshot.settings.wizardCompleted) return null

  const finish = async () => {
    setBusyLocal(true)
    try {
      await updateSettings({ wizardCompleted: true }, { quiet: true })
    } finally {
      setBusyLocal(false)
    }
  }

  const skip = () => void finish()

  const runEngine = async () => {
    setLocalError(null)
    setBusyLocal(true)
    try {
      await updateChannel('engine')
    } catch (e) {
      setLocalError(e instanceof Error ? e.message : String(e))
    } finally {
      setBusyLocal(false)
    }
  }

  const runUac = async () => {
    setLocalError(null)
    setBusyLocal(true)
    try {
      if (snapshot.connection.status !== 'running') {
        await toggleEngine()
      }
    } catch (e) {
      setLocalError(e instanceof Error ? e.message : String(e))
    } finally {
      setUacTried(true)
      setBusyLocal(false)
    }
  }

  const runSmart = async () => {
    setLocalError(null)
    setBusyLocal(true)
    try {
      await startDiagnostics()
    } catch (e) {
      setLocalError(e instanceof Error ? e.message : String(e))
    } finally {
      setBusyLocal(false)
    }
  }

  const downloading =
    engine?.status === 'downloading' ||
    engine?.status === 'installing' ||
    !!busy['update:engine']

  const blocked = busyLocal || downloading || !!busy.engine || !!busy['diagnostics:start']

  const copy =
    step === 'engine'
      ? {
          title: t('wizard.stepEngine'),
          body: t('wizard.stepEngineBody'),
          cta: t('wizard.stepEngineCta'),
          action: runEngine,
        }
      : step === 'uac'
        ? {
            title: t('wizard.stepUac'),
            body: t('wizard.stepUacBody'),
            cta: t('wizard.stepUacCta'),
            action: runUac,
          }
        : step === 'smart'
          ? {
              title: t('wizard.stepSmart'),
              body: t('wizard.stepSmartBody'),
              cta: t('wizard.stepSmartCta'),
              action: runSmart,
            }
          : {
              title: t('wizard.stepDone'),
              body: t('wizard.stepDoneBody'),
              cta: t('wizard.finish'),
              action: finish,
            }

  const steps: Step[] = ['engine', 'uac', 'smart', 'done']
  const idx = steps.indexOf(step)

  return (
    <div className={styles.overlay} role="dialog" aria-modal="true" aria-labelledby="wizard-title">
      <div className={styles.card}>
        <div className={styles.kicker}>{t('wizard.title')}</div>
        <div className={styles.progress} aria-hidden>
          {steps.map((s, i) => (
            <span
              key={s}
              className={[
                styles.dot,
                i < idx ? styles.dotDone : '',
                i === idx ? styles.dotActive : '',
              ].join(' ')}
            />
          ))}
        </div>
        <h2 id="wizard-title" className={styles.title}>
          {copy.title}
        </h2>
        <p className={styles.body}>{copy.body}</p>
        {lastError && step !== 'done' && (
          <p className={styles.error} role="alert">
            {lastError}
          </p>
        )}
        {downloading && engine?.downloadProgress != null && (
          <div className={styles.barTrack}>
            <div
              className={styles.barFill}
              style={{ width: `${Math.min(100, engine.downloadProgress)}%` }}
            />
          </div>
        )}
        <div className={styles.actions}>
          <Button variant="primary" disabled={blocked} onClick={() => void copy.action()}>
            {copy.cta}
          </Button>
          {step === 'uac' && uacTried && (
            <Button
              variant="secondary"
              disabled={blocked}
              onClick={() => {
                setUacTried(false)
                setLocalError(null)
              }}
            >
              {t('wizard.retryUac')}
            </Button>
          )}
          {step !== 'done' && (
            <Button variant="ghost" disabled={blocked} onClick={skip}>
              {t('wizard.skip')}
            </Button>
          )}
        </div>
      </div>
    </div>
  )
}
