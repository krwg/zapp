// ╔════════════════════════════════════════════════════╗
// ║        pages/Diagnostics/DiagnosticsPage         ║
// ╚════════════════════════════════════════════════════╝

import { useState } from 'react'
import { ChevronDown } from 'lucide-react'
import { DiagnosticProgress } from '@/components/domain/DiagnosticProgress/DiagnosticProgress'
import { PageHeader } from '@/components/layout/PageHeader/PageHeader'
import { Button } from '@/components/ui/Button/Button'
import { Hint } from '@/components/ui/Hint/Hint'
import { StatusBadge } from '@/components/ui/StatusBadge/StatusBadge'
import { useAppState } from '@/context/AppStateContext'
import { useLocale } from '@/context/LocaleContext'
import type { DiagnosticStrategyResult } from '@/types'
import { formatLatency } from '@/utils/format'
import page from '../shared/page.module.css'
import styles from './DiagnosticsPage.module.css'

function rankKey(a: DiagnosticStrategyResult, b: DiagnosticStrategyResult) {
  if (b.score !== a.score) return b.score - a.score
  const la = a.latencyMs ?? 99999
  const lb = b.latencyMs ?? 99999
  return la - lb
}

export function DiagnosticsPage() {
  const {
    diagnosticsLive,
    snapshot,
    startDiagnostics,
    cancelDiagnostics,
    selectDiagnosticStrategy,
    updateSettings,
    busy,
  } = useAppState()
  const { t } = useLocale()
  const [openId, setOpenId] = useState<string | null>(null)

  if (!diagnosticsLive || !snapshot) return null

  const sorted = [...diagnosticsLive.results].sort(rankKey)
  const bestScore = sorted[0]?.score ?? 0
  const mode = snapshot.settings.diagnosticsMode ?? 'fast'

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
          busy={!!busy['diagnostics:start']}
          cancelBusy={!!busy['diagnostics:cancel']}
          mode={mode}
          onModeChange={(next) => void updateSettings({ diagnosticsMode: next })}
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
                const expanded = openId === row.id
                return (
                  <article
                    key={row.id}
                    className={[styles.rankCard, row.success ? styles.rankOk : styles.rankFail].join(
                      ' ',
                    )}
                  >
                    <div className={styles.rankIndex}>#{rank}</div>
                    <div className={styles.rankBody}>
                      <button
                        type="button"
                        className={styles.rankToggle}
                        aria-expanded={expanded}
                        onClick={() => setOpenId(expanded ? null : row.id)}
                      >
                        <div className={styles.rankTop}>
                          <div>
                            <div className={styles.rankName}>{row.name}</div>
                            <div className={styles.rankSummary}>
                              {t('smartCheck.score')}: <strong>{row.score}</strong>
                              <span className={styles.dot}>·</span>
                              {t('smartCheck.latency')}:{' '}
                              <strong>{formatLatency(row.latencyMs)}</strong>
                            </div>
                          </div>
                          <div className={styles.rankTopRight}>
                            <StatusBadge
                              label={row.success ? t('status.working') : t('status.failed')}
                              tone={row.success ? 'success' : 'error'}
                            />
                            <ChevronDown
                              size={16}
                              className={[styles.chevron, expanded ? styles.chevronOpen : ''].join(
                                ' ',
                              )}
                            />
                          </div>
                        </div>
                        <div className={styles.scoreTrack} aria-hidden>
                          <span className={styles.scoreFill} style={{ width: `${width}%` }} />
                        </div>
                      </button>

                      {expanded ? (
                        <div className={styles.rankDetails}>
                          <div className={styles.rankType}>{row.strategyType}</div>
                          <div className={styles.detailGrid}>
                            <div>
                              <span className={styles.detailLabel}>{t('smartCheck.latencyMedian')}</span>
                              <strong>{formatLatency(row.latencyMedianMs)}</strong>
                            </div>
                            <div>
                              <span className={styles.detailLabel}>{t('smartCheck.latencyMax')}</span>
                              <strong>{formatLatency(row.latencyMaxMs)}</strong>
                            </div>
                            <div>
                              <span className={styles.detailLabel}>{t('smartCheck.softScore')}</span>
                              <strong>{row.softScore}</strong>
                            </div>
                          </div>
                          {row.services?.length ? (
                            <div className={styles.serviceList}>
                              {row.services.map((s) => (
                                <div key={s.service} className={styles.serviceRow}>
                                  <span>{s.service}</span>
                                  <span>
                                    {s.score}
                                    {s.ok ? '' : '!'} · {formatLatency(s.latencyMs)}
                                  </span>
                                </div>
                              ))}
                            </div>
                          ) : null}
                          {row.hosts?.length ? (
                            <div className={styles.hostList}>
                              {row.hosts.map((h, i) => (
                                <div key={`${h.host}-${i}`} className={styles.hostRow}>
                                  <span className={h.ok ? styles.hostOk : styles.hostFail}>
                                    {h.host}
                                  </span>
                                  <span>
                                    {formatLatency(h.latencyMs)} · {h.note}
                                  </span>
                                </div>
                              ))}
                            </div>
                          ) : null}
                          {row.notes ? <p className={styles.notesFull}>{row.notes}</p> : null}
                        </div>
                      ) : null}
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
