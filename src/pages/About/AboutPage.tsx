// ╔════════════════════════════════════════════════════╗
// ║              pages/About/AboutPage               ║
// ╚════════════════════════════════════════════════════╝

import { Bug, ExternalLink } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useLocation, useNavigate } from 'react-router-dom'
import { PageHeader } from '@/components/layout/PageHeader/PageHeader'
import { Button } from '@/components/ui/Button/Button'
import { Dialog } from '@/components/ui/Dialog/Dialog'
import { useAppState } from '@/context/AppStateContext'
import { useLocale } from '@/context/LocaleContext'
import { useUiPrefs } from '@/context/UiPrefsContext'
import shieldMark from '@/assets/brand/shield-128.png'
import type {
  AboutLocationState,
  BugReportBundle,
  BugReportPrefill,
} from '@/types/bugReport'
import { formatVersion } from '@/utils/version'
import styles from './AboutPage.module.css'

const AUTHOR_URL = 'https://krwg.github.io/krwg/'
const YEAR = 2026
/** Keep under common browser / GitHub URL length limits. */
const MAX_ISSUE_BODY = 6500

export function AboutPage() {
  const { snapshot, getBugReportBundle } = useAppState()
  const { t } = useLocale()
  const location = useLocation()
  const navigate = useNavigate()
  const { bugReportHintSeen, setPref } = useUiPrefs()
  const [reportOpen, setReportOpen] = useState(false)
  const [summary, setSummary] = useState('')
  const [steps, setSteps] = useState('')
  const [expected, setExpected] = useState('')
  const [actual, setActual] = useState('')
  const [areas, setAreas] = useState({
    bypass: false,
    smart: false,
    updates: false,
  })
  const [bundle, setBundle] = useState<BugReportBundle | null>(null)

  useEffect(() => {
    if (!bugReportHintSeen) {
      setPref('bugReportHintSeen', true)
    }
  }, [bugReportHintSeen, setPref])

  const applyPrefill = (prefill?: BugReportPrefill) => {
    if (!prefill) return
    if (prefill.summary) setSummary(prefill.summary)
    if (prefill.steps !== undefined) setSteps(prefill.steps)
    if (prefill.expected) setExpected(prefill.expected)
    if (prefill.actual) setActual(prefill.actual)
    if (prefill.areas) {
      setAreas((a) => ({
        bypass: prefill.areas?.bypass ?? a.bypass,
        smart: prefill.areas?.smart ?? a.smart,
        updates: prefill.areas?.updates ?? a.updates,
      }))
    }
  }

  const openReport = async (prefill?: BugReportPrefill) => {
    applyPrefill(prefill)
    setReportOpen(true)
    try {
      setBundle(await getBugReportBundle())
    } catch (err) {
      console.warn('getBugReportBundle failed', err)
      // Fallback so the dialog is never empty if ACL/IPC glitches.
      setBundle({
        appVersion: snapshot?.system.appVersion ?? '?',
        engineVersion: snapshot?.system.engineVersion ?? '?',
        engineRoot: snapshot?.settings.enginePath ?? '(unknown)',
        engineInstalled: true,
        elevated: snapshot?.system.elevated ?? false,
        diagnosticsMode: String(snapshot?.settings.diagnosticsMode ?? '?'),
        bypassDesired: false,
        activeStrategy: null,
        recentErrors: [String(prefill?.actual ?? '(bundle unavailable)')],
        recentLog: [],
      })
    }
  }

  useEffect(() => {
    const state = location.state as AboutLocationState | null
    if (!state?.openBugReport) return
    void openReport(state.bugPrefill)
    navigate('.', { replace: true, state: {} })
    // intentionally only when landing with openBugReport
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [location.key])

  if (!snapshot) return null

  const { about, system, settings } = snapshot

  const buildIssueBody = (diag: BugReportBundle | null) => {
    const areaList = [
      areas.bypass ? 'Bypass' : null,
      areas.smart ? 'Smart check' : null,
      areas.updates ? 'Updates' : null,
    ]
      .filter(Boolean)
      .join(', ')

    const env = diag
      ? [
          `- App: ${diag.appVersion}`,
          `- Engine: ${diag.engineVersion} (installed=${diag.engineInstalled})`,
          `- Engine root: \`${diag.engineRoot}\``,
          `- Elevated: ${diag.elevated}`,
          `- Diagnostics mode: ${diag.diagnosticsMode}`,
          `- Bypass desired: ${diag.bypassDesired}`,
          `- Active strategy: ${diag.activeStrategy ?? '(none)'}`,
          `- OS: Windows`,
        ]
      : [
          `- App: ${system.appVersion}`,
          `- Engine: ${system.engineVersion}`,
          `- OS: Windows`,
          `- Elevated: ${system.elevated}`,
          `- Diagnostics mode: ${settings.diagnosticsMode}`,
        ]

    const errors = diag?.recentErrors?.length
      ? diag.recentErrors.map((e) => `- ${e}`).join('\n')
      : '- (none)'
    const log = diag?.recentLog?.length
      ? ['```', ...diag.recentLog, '```'].join('\n')
      : '(none)'

    let body = [
      '### Summary',
      summary.trim() || '(none)',
      '',
      '### Steps to reproduce',
      steps.trim() || '(none)',
      '',
      '### Expected',
      expected.trim() || '(none)',
      '',
      '### Actual',
      actual.trim() || '(none)',
      '',
      '### Areas',
      areaList || '(none)',
      '',
      '### Environment (auto)',
      ...env,
      '',
      '### Recent errors (auto)',
      errors,
      '',
      '### Recent log (auto)',
      log,
      '',
    ].join('\n')

    if (body.length > MAX_ISSUE_BODY) {
      body = `${body.slice(0, MAX_ISSUE_BODY - 40).trimEnd()}\n\n…(truncated for URL length)\n`
    }
    return body
  }

  const sendReport = async () => {
    const body = buildIssueBody(bundle)
    const title = summary.trim() || 'Bug report from zapp'
    const params = new URLSearchParams({
      title,
      body,
      labels: 'bug',
    })
    const url = `https://github.com/krwg/zapp/issues/new?${params.toString()}`
    const { getApi } = await import('@/api/client')
    try {
      await getApi().openExternalUrl(url)
    } catch {
      window.open(url, '_blank', 'noopener,noreferrer')
    }
    setReportOpen(false)
  }

  return (
    <div>
      <PageHeader title={t('aboutPage.title')} description={t('aboutPage.description')} />

      <div className={styles.stack}>
        <section className={styles.hero}>
          <div className={styles.mark}>
            <img src={shieldMark} alt="" width={72} height={72} className={styles.markImg} />
          </div>
          <div className={styles.heroCopy}>
            <h2 className={styles.brand}>{about.name}</h2>
            <p className={styles.intro}>{t('aboutPage.intro')}</p>
          </div>
        </section>

        <section className={styles.card}>
          <h3 className={styles.cardTitle}>{t('aboutPage.controlTitle')}</h3>
          <p className={styles.cardBody}>{t('aboutPage.controlBody')}</p>
        </section>

        <section className={styles.metaGrid}>
          <div className={styles.metaItem}>
            <div className={styles.metaLabel}>{t('aboutPage.version')}</div>
            <div className={styles.metaValue}>{formatVersion(about.version)}</div>
          </div>
          <div className={styles.metaItem}>
            <div className={styles.metaLabel}>{t('aboutPage.author')}</div>
            <a className={styles.metaLink} href={AUTHOR_URL} target="_blank" rel="noreferrer">
              {t('aboutPage.visitAuthor')}
              <ExternalLink size={13} />
            </a>
          </div>
          <div className={styles.metaItem}>
            <div className={styles.metaLabel}>{t('aboutPage.year')}</div>
            <div className={styles.metaValue}>{YEAR}</div>
          </div>
          <div className={styles.metaItem}>
            <div className={styles.metaLabel}>{t('aboutPage.license')}</div>
            <div className={styles.metaValue}>{about.license}</div>
          </div>
        </section>

        <section className={styles.repoCard}>
          <div>
            <div className={styles.metaLabel}>{t('aboutPage.repository')}</div>
            <div className={styles.repoUrl}>{about.githubUrl.replace(/^https?:\/\//, '')}</div>
          </div>
          <a className={styles.repoBtn} href={about.githubUrl} target="_blank" rel="noreferrer">
            <ExternalLink size={15} />
            {t('aboutPage.openGithub')}
          </a>
        </section>

        <section className={styles.repoCard}>
          <div>
            <div className={styles.metaLabel}>{t('aboutPage.reportTitle')}</div>
            <div className={styles.repoUrl}>{t('aboutPage.reportHint')}</div>
          </div>
          <button type="button" className={styles.repoBtn} onClick={() => void openReport()}>
            <Bug size={15} />
            {t('aboutPage.reportBug')}
          </button>
        </section>
      </div>

      <Dialog
        open={reportOpen}
        title={t('aboutPage.reportTitle')}
        description={t('aboutPage.reportHint')}
        onClose={() => setReportOpen(false)}
        footer={
          <>
            <Button variant="ghost" onClick={() => setReportOpen(false)}>
              {t('common.cancel')}
            </Button>
            <Button variant="primary" onClick={() => void sendReport()}>
              <ExternalLink size={14} />
              {t('aboutPage.reportSend')}
            </Button>
          </>
        }
      >
        <div className={styles.form}>
          <label className={styles.field}>
            <span>{t('aboutPage.reportSummary')}</span>
            <input
              value={summary}
              onChange={(e) => setSummary(e.target.value)}
              placeholder={t('aboutPage.reportSummaryPh')}
            />
          </label>
          <label className={styles.field}>
            <span>{t('aboutPage.reportSteps')}</span>
            <textarea
              value={steps}
              onChange={(e) => setSteps(e.target.value)}
              placeholder={t('aboutPage.reportStepsPh')}
              rows={3}
            />
          </label>
          <label className={styles.field}>
            <span>{t('aboutPage.reportExpected')}</span>
            <textarea
              value={expected}
              onChange={(e) => setExpected(e.target.value)}
              placeholder={t('aboutPage.reportExpectedPh')}
              rows={2}
            />
          </label>
          <label className={styles.field}>
            <span>{t('aboutPage.reportActual')}</span>
            <textarea
              value={actual}
              onChange={(e) => setActual(e.target.value)}
              placeholder={t('aboutPage.reportActualPh')}
              rows={2}
            />
          </label>
          <div className={styles.areas}>
            <span>{t('aboutPage.reportAreas')}</span>
            <label>
              <input
                type="checkbox"
                checked={areas.bypass}
                onChange={(e) => setAreas((a) => ({ ...a, bypass: e.target.checked }))}
              />
              {t('aboutPage.reportAreaBypass')}
            </label>
            <label>
              <input
                type="checkbox"
                checked={areas.smart}
                onChange={(e) => setAreas((a) => ({ ...a, smart: e.target.checked }))}
              />
              {t('aboutPage.reportAreaSmart')}
            </label>
            <label>
              <input
                type="checkbox"
                checked={areas.updates}
                onChange={(e) => setAreas((a) => ({ ...a, updates: e.target.checked }))}
              />
              {t('aboutPage.reportAreaUpdates')}
            </label>
          </div>

          <div className={styles.diag}>
            <div className={styles.diagTitle}>{t('aboutPage.reportDiagTitle')}</div>
            <p className={styles.diagHint}>{t('aboutPage.reportDiagHint')}</p>
            <pre className={styles.diagPre}>
              {bundle
                ? [
                    `App ${bundle.appVersion} · Engine ${bundle.engineVersion}`,
                    `Root ${bundle.engineRoot}`,
                    `Elevated ${bundle.elevated} · Mode ${bundle.diagnosticsMode}`,
                    `Bypass desired ${bundle.bypassDesired} · Strategy ${bundle.activeStrategy ?? '—'}`,
                    '',
                    'Errors:',
                    ...(bundle.recentErrors.length
                      ? bundle.recentErrors.slice(0, 6)
                      : ['(none)']),
                    '',
                    'Log:',
                    ...(bundle.recentLog.length
                      ? bundle.recentLog.slice(0, 10)
                      : ['(none)']),
                  ].join('\n')
                : '…'}
            </pre>
          </div>
        </div>
      </Dialog>
    </div>
  )
}
