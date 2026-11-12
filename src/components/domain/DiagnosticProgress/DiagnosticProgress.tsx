// ╔════════════════════════════════════════════════════╗
// ║ nts/domain/DiagnosticProgress/DiagnosticProgress ║
// ╚════════════════════════════════════════════════════╝

import { Play, RotateCcw, Square } from 'lucide-react'
import { Button } from '@/components/ui/Button/Button'
import { ProgressBar } from '@/components/ui/ProgressBar/ProgressBar'
import { StatusBadge, type StatusTone } from '@/components/ui/StatusBadge/StatusBadge'
import type { DiagnosticPhase, DiagnosticRun } from '@/types'
import styles from './DiagnosticProgress.module.css'

interface DiagnosticProgressProps {
  run: DiagnosticRun
  busy?: boolean
  onStart: () => void
  onCancel: () => void
}

function phaseMeta(phase: DiagnosticPhase): { label: string; tone: StatusTone; pulse?: boolean } {
  switch (phase) {
    case 'idle':
      return { label: 'Idle', tone: 'neutral' }
    case 'preparing':
      return { label: 'Preparing', tone: 'warning', pulse: true }
    case 'testing':
      return { label: 'Testing', tone: 'warning', pulse: true }
    case 'analyzing':
      return { label: 'Analyzing', tone: 'warning', pulse: true }
    case 'completed':
      return { label: 'Completed', tone: 'success' }
    case 'failed':
      return { label: 'Failed', tone: 'error' }
    case 'cancelled':
      return { label: 'Cancelled', tone: 'neutral' }
  }
}

function phaseText(phase: DiagnosticPhase): string {
  switch (phase) {
    case 'idle':
      return 'Готово к запуску автоматического подбора стратегии.'
    case 'preparing':
      return 'Подготовка тестового окружения…'
    case 'testing':
      return 'Поочерёдная проверка стратегий.'
    case 'analyzing':
      return 'Сравнение результатов и ранжирование.'
    case 'completed':
      return 'Тестирование завершено. Можно выбрать рабочую стратегию.'
    case 'failed':
      return 'Диагностика завершилась с ошибкой.'
    case 'cancelled':
      return 'Диагностика отменена пользователем.'
  }
}

export function DiagnosticProgress({ run, busy, onStart, onCancel }: DiagnosticProgressProps) {
  const meta = phaseMeta(run.phase)
  const running = run.phase === 'preparing' || run.phase === 'testing' || run.phase === 'analyzing'

  return (
    <section className={styles.card}>
      <div className={styles.header}>
        <div>
          <h2 className={styles.title}>Automatic diagnostics</h2>
          <p className={styles.phase}>{phaseText(run.phase)}</p>
          {run.isDemo && <span className={styles.demo}>Демонстрационные данные</span>}
        </div>
        <StatusBadge label={meta.label} tone={meta.tone} pulse={meta.pulse} />
      </div>

      <ProgressBar
        value={run.progress}
        label="Progress"
        indeterminate={run.phase === 'preparing' || run.phase === 'analyzing'}
      />

      <div className={styles.stats}>
        <div className={styles.stat}>
          <div className={styles.statLabel}>Tested</div>
          <div className={styles.statValue}>
            {run.testedCount}/{run.totalCount}
          </div>
        </div>
        <div className={styles.stat}>
          <div className={styles.statLabel}>Working</div>
          <div className={styles.statValue}>{run.workingCount}</div>
        </div>
        <div className={styles.stat}>
          <div className={styles.statLabel}>Progress</div>
          <div className={styles.statValue}>{run.progress}%</div>
        </div>
      </div>

      {run.currentStrategy && (
        <p className={styles.current}>
          Current strategy: <strong>{run.currentStrategy}</strong>
        </p>
      )}

      <div className={styles.actions}>
        {running ? (
          <Button variant="danger" onClick={onCancel} disabled={busy}>
            <Square size={14} />
            Cancel
          </Button>
        ) : (
          <Button variant="primary" onClick={onStart} disabled={busy}>
            {run.phase === 'completed' || run.phase === 'cancelled' ? (
              <RotateCcw size={14} />
            ) : (
              <Play size={14} />
            )}
            {run.phase === 'completed' || run.phase === 'cancelled' ? 'Run again' : 'Start diagnostics'}
          </Button>
        )}
      </div>
    </section>
  )
}
