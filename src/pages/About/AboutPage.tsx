// ╔════════════════════════════════════════════════════╗
// ║              pages/About/AboutPage               ║
// ╚════════════════════════════════════════════════════╝

import { Bug, ExternalLink } from 'lucide-react'
import { useEffect, useState } from 'react'
import { PageHeader } from '@/components/layout/PageHeader/PageHeader'
import { Button } from '@/components/ui/Button/Button'
import { Dialog } from '@/components/ui/Dialog/Dialog'
import { useAppState } from '@/context/AppStateContext'
import { useLocale } from '@/context/LocaleContext'
import { useUiPrefs } from '@/context/UiPrefsContext'
import shieldMark from '@/assets/brand/shield-128.png'
import { formatVersion } from '@/utils/version'
import styles from './AboutPage.module.css'

const AUTHOR_URL = 'https://krwg.github.io/krwg/'
const YEAR = 2026
const TG_PROXY_URL = 'https://github.com/Flowseal/tg-ws-proxy'

export function AboutPage() {
  const { snapshot, getRecentErrors } = useAppState()
  const { t } = useLocale()
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
    telegram: false,
  })
  const [recentErrors, setRecentErrors] = useState<string[]>([])

  useEffect(() => {
    if (!bugReportHintSeen) {
      setPref('bugReportHintSeen', true)
    }
  }, [bugReportHintSeen, setPref])

  if (!snapshot) return null

  const { about, system, telegram, settings } = snapshot

  const openReport = async () => {
    try {
      setRecentErrors(await getRecentErrors())
    } catch {
      setRecentErrors([])
    }
    setReportOpen(true)
  }

  const buildIssue = () => {
    const areaList = [
      areas.bypass ? 'Bypass' : null,
      areas.smart ? 'Smart check' : null,
      areas.updates ? 'Updates' : null,
      areas.telegram ? 'Telegram' : null,
    ]
      .filter(Boolean)
      .join(', ')

    const body = [
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
      `- App: ${system.appVersion}`,
      `- Engine: ${system.engineVersion}`,
      `- Telegram proxy: ${telegram.version} (installed=${telegram.installed})`,
      `- OS: Windows`,
      `- Elevated: ${system.elevated}`,
      `- Diagnostics mode: ${settings.diagnosticsMode}`,
      '',
      '### Recent errors (auto, no secrets)',
      recentErrors.length ? recentErrors.map((e) => `- ${e}`).join('\n') : '- (none)',
      '',
    ].join('\n')

    const title = summary.trim() || 'Bug report from zapp'
    const params = new URLSearchParams({
      title,
      body,
      labels: 'bug',
    })
    return `https://github.com/krwg/zapp/issues/new?${params.toString()}`
  }

  const sendReport = async () => {
    const url = buildIssue()
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
            <div className={styles.metaLabel}>{t('aboutPage.creditsTitle')}</div>
            <div className={styles.repoUrl}>{t('telegram.credits')}</div>
          </div>
          <a className={styles.repoBtn} href={TG_PROXY_URL} target="_blank" rel="noreferrer">
            <ExternalLink size={15} />
            tg-ws-proxy
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
            <label>
              <input
                type="checkbox"
                checked={areas.telegram}
                onChange={(e) => setAreas((a) => ({ ...a, telegram: e.target.checked }))}
              />
              {t('aboutPage.reportAreaTelegram')}
            </label>
          </div>
        </div>
      </Dialog>
    </div>
  )
}
