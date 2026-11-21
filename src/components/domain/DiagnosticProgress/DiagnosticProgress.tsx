// ╔════════════════════════════════════════════════════╗
// ║ nts/domain/DiagnosticProgress/DiagnosticProgress ║
// ╚════════════════════════════════════════════════════╝

import { CircleStop, Play, RotateCcw } from 'lucide-react'
import { Button } from '@/components/ui/Button/Button'
import { useLocale } from '@/context/LocaleContext'
import type { MessageKey } from '@/i18n'
import type { DiagnosticPhase, DiagnosticRun, DiagnosticsMode } from '@/types'
import styles from './DiagnosticProgress.module.css'

interface DiagnosticProgressProps {
  run: DiagnosticRun
  busy?: boolean
  
  cancelBusy?: boolean
  mode: DiagnosticsMode
  onModeChange: (mode: DiagnosticsMode) => void
  onStart: () => void
  onCancel: () => void
}

function phaseText(phase: DiagnosticPhase, t: (key: MessageKey) => string): string {
  switch (phase) {
    case 'idle':
      return t('smartCheck.phaseIdle')
    case 'preparing':
      return t('smartCheck.phasePreparing')
    case 'testing':
      return t('smartCheck.phaseTesting')
    case 'analyzing':
      return t('smartCheck.phaseAnalyzing')
    case 'completed':
      return t('smartCheck.phaseCompleted')
    case 'failed':
      return t('smartCheck.phaseFailed')
    case 'cancelled':
      return t('smartCheck.phaseCancelled')
  }
}

export function DiagnosticProgress({
  run,
  busy,
  cancelBusy,
  mode,
  onModeChange,
  onStart,
  onCancel,
}: DiagnosticProgressProps) {
  const { t } = useLocale()
  const running = run.phase === 'preparing' || run.phase === 'testing' || run.phase === 'analyzing'
  const progress = Math.max(0, Math.min(100, run.progress))
  const angle = progress * 3.6

  return (
    <section className={[styles.hero, running ? styles.heroLive : ''].join(' ')}>
      <div className={styles.orbitWrap} aria-hidden>
        <div className={styles.orbitOuter} />
        <div className={styles.orbitMid} />
        <div
          className={styles.orbitFill}
          style={{
            background: `conic-gradient(var(--text-primary) ${angle}deg, transparent ${angle}deg)`,
          }}
        />
        <div className={styles.orbitCore}>
          <div className={styles.percent}>{progress}%</div>
          <div className={styles.percentLabel}>{t('smartCheck.progress')}</div>
        </div>
        {running && <span className={styles.pulseDot} />}
      </div>

      <div className={styles.copy}>
        <p className={styles.phase}>{phaseText(run.phase, t)}</p>
        {run.currentStrategy ? (
          <p className={styles.current}>
            {t('smartCheck.currentMode')} <strong>{run.currentStrategy}</strong>
          </p>
        ) : null}
        {run.isDemo && <span className={styles.demo}>{t('smartCheck.demo')}</span>}

        <div className={styles.meters}>
          <div className={styles.meter}>
            <span>{t('smartCheck.tested')}</span>
            <strong>
              {run.testedCount}/{run.totalCount || '—'}
            </strong>
          </div>
          <div className={styles.meter}>
            <span>{t('smartCheck.working')}</span>
            <strong>{run.workingCount}</strong>
          </div>
        </div>

        <div
          className={styles.modeSegment}
          role="group"
          aria-label={t('smartCheck.mode')}
          title={t('smartCheck.modeHint')}
        >
          <button
            type="button"
            className={mode === 'fast' ? styles.modeActive : undefined}
            disabled={running || busy}
            onClick={() => onModeChange('fast')}
          >
            {t('smartCheck.modeFast')}
          </button>
          <button
            type="button"
            className={mode === 'thorough' ? styles.modeActive : undefined}
            disabled={running || busy}
            onClick={() => onModeChange('thorough')}
          >
            {t('smartCheck.modeThorough')}
          </button>
        </div>

        <div className={styles.actions}>
          {running ? (
            <Button variant="danger" onClick={onCancel} disabled={!!cancelBusy}>
              <CircleStop size={15} strokeWidth={2} />
              {t('smartCheck.cancel')}
            </Button>
          ) : (
            <Button variant="primary" onClick={onStart} disabled={busy}>
              {run.phase === 'completed' || run.phase === 'cancelled' ? (
                <RotateCcw size={14} />
              ) : (
                <Play size={14} />
              )}
              {run.phase === 'completed' || run.phase === 'cancelled'
                ? t('smartCheck.runAgain')
                : t('smartCheck.start')}
            </Button>
          )}
        </div>
      </div>
    </section>
  )
}

