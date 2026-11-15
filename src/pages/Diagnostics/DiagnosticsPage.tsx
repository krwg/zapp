// ╔════════════════════════════════════════════════════╗
// ║        pages/Diagnostics/DiagnosticsPage         ║
// ╚════════════════════════════════════════════════════╝

import { DiagnosticProgress } from '@/components/domain/DiagnosticProgress/DiagnosticProgress'
import { PageHeader } from '@/components/layout/PageHeader/PageHeader'
import { Button } from '@/components/ui/Button/Button'
import { Hint } from '@/components/ui/Hint/Hint'
import { StatusBadge } from '@/components/ui/StatusBadge/StatusBadge'
import { useAppState } from '@/context/AppStateContext'
import { useLocale } from '@/context/LocaleContext'
import { formatLatency } from '@/utils/format'
import page from '../shared/page.module.css'
import styles from './DiagnosticsPage.module.css'

export function DiagnosticsPage() {
  const {
    diagnosticsLive,
    startDiagnostics,
    cancelDiagnostics,
    selectDiagnosticStrategy,
    busy,
  } = useAppState()
  const { t } = useLocale()

  if (!diagnosticsLive) return null

  const sorted = [...diagnosticsLive.results].sort((a, b) => b.score - a.score)
  const bestScore = sorted[0]?.score ?? 0

  return (
    <div>
      <PageHeader
        title={t('smartCheck.title')}
        description={t('smartCheck.description')}
        titleAddon={
          <Hint
            label={t('hints.smartCheckLabel')}
            title={t('hints.smartCheckTitle')}
            body={t('hints.smartCheckBody')}
            wide
          />
        }
      />

      <div className={page.stack}>
        <DiagnosticProgress
          run={diagnosticsLive}
          busy={busy.diagnostics}
          onStart={() => void startDiagnostics()}
          onCancel={() => void cancelDiagnostics()}
        />

        <section className={styles.results}>
          <div className={styles.resultsHead}>
            <div className={styles.resultsTitleRow}>
              <h2 className={styles.resultsTitle}>{t('smartCheck.results')}</h2>
              <Hint
                label={t('hints.resultsLabel')}
                title={t('hints.resultsTitle')}
                body={t('hints.resultsBody')}
                wide
              />
            </div>
            <span className={styles.resultsCount}>
              {t('smartCheck.workingOf', {
                working: diagnosticsLive.workingCount,
                total: diagnosticsLive.results.length,
              })}
            </span>
          </div>

          {sorted.length === 0 ? (
            <div className={styles.empty}>{t('smartCheck.empty')}</div>
          ) : (
            <div className={styles.rankList}>
              {sorted.map((row, index) => {
                const rank = index + 1
                const width = bestScore > 0 ? Math.round((row.score / bestScore) * 100) : 0
                return (
                  <article
                    key={row.id}
                    className={[styles.rankCard, row.success ? styles.rankOk : styles.rankFail].join(
                      ' ',
                    )}
                  >
                    <div className={styles.rankIndex}>#{rank}</div>
                    <div className={styles.rankBody}>
                      <div className={styles.rankTop}>
                        <div>
                          <div className={styles.rankName}>{row.name}</div>
                          <div className={styles.rankType}>{row.strategyType}</div>
                        </div>
                        <StatusBadge
                          label={row.success ? t('status.working') : t('status.failed')}
                          tone={row.success ? 'success' : 'error'}
                        />
                      </div>
                      <div className={styles.scoreTrack} aria-hidden>
                        <span className={styles.scoreFill} style={{ width: `${width}%` }} />
                      </div>
                      <div className={styles.rankMeta}>
                        <span>
                          {t('smartCheck.score')}: <strong>{row.score}</strong>
                        </span>
                        <span>
                          {t('smartCheck.latency')}: <strong>{formatLatency(row.latencyMs)}</strong>
                        </span>
                        {row.notes ? <span className={styles.notes}>{row.notes}</span> : null}
                      </div>
                    </div>
                    <div className={styles.rankAction}>
                      <Button
                        size="sm"
                        variant={rank === 1 && row.success ? 'primary' : 'secondary'}
                        disabled={!row.success || busy[`diag-select:${row.id}`]}
                        onClick={() => void selectDiagnosticStrategy(row.id)}
                      >
                        {t('common.use')}
                      </Button>
                    </div>
                  </article>
                )
              })}
            </div>
          )}
        </section>
      </div>
    </div>
  )
}
