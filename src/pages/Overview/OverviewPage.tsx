// ╔════════════════════════════════════════════════════╗
// ║           pages/Overview/OverviewPage            ║
// ╚════════════════════════════════════════════════════╝

import { useEffect, useRef, useState } from 'react'
import { ChevronDown, Link as LinkIcon } from 'lucide-react'
import { Link, useNavigate } from 'react-router-dom'
import { ConnectionCard } from '@/components/domain/ConnectionCard/ConnectionCard'
import { PageHeader } from '@/components/layout/PageHeader/PageHeader'
import { Button } from '@/components/ui/Button/Button'
import { Hint } from '@/components/ui/Hint/Hint'
import { StatusBadge } from '@/components/ui/StatusBadge/StatusBadge'

import { useAppState } from '@/context/AppStateContext'
import { useLocale } from '@/context/LocaleContext'
import { formatDateTime, formatLatency, formatRelative } from '@/utils/format'
import { sourceLabelKey } from '@/utils/modeLabels'
import { serviceIcon } from '@/utils/serviceIcons'
import { displaySiteStatus, isSiteCheckFresh, siteStatusLabelKey } from '@/utils/siteStatus'
import { formatVersion } from '@/utils/version'
import page from '../shared/page.module.css'
import styles from './OverviewPage.module.css'

function friendlyModeName(name: string): string {
  // "general (FAKE TLS AUTO)" → keep readable stem before the technical paren.
  const cut = name.indexOf(' (')
  if (cut > 0) return name.slice(0, cut)
  return name
}

export function OverviewPage() {
  const {
    snapshot,
    diagnosticsLive,
    toggleEngine,
    resetEngine,
    checkService,
    checkAllServices,
    busy,
  } = useAppState()
  const { t, locale } = useLocale()
  const navigate = useNavigate()
  const [modeOpen, setModeOpen] = useState(false)
  const lastSilentProbeAt = useRef(0)

  useEffect(() => {
    if (!snapshot) return
    if (snapshot.connection.status !== 'running') return
    const phase = diagnosticsLive?.phase
    if (phase === 'preparing' || phase === 'testing' || phase === 'analyzing') return
    const now = Date.now()
    if (now - lastSilentProbeAt.current < 45_000) return
    lastSilentProbeAt.current = now
    void checkAllServices({ silent: true })
  }, [snapshot, diagnosticsLive?.phase, checkAllServices])

  if (!snapshot) return null

  const { connection, activeProfile, services, system } = snapshot
  const neverChecked =
    !activeProfile?.lastCheckedAt &&
    !(diagnosticsLive?.results?.length) &&
    diagnosticsLive?.phase !== 'completed'

  const yt = services.find((s) => /youtube/i.test(s.name) || /youtube/i.test(s.host))
  const discord = services.find((s) => /discord/i.test(s.name) || /discord/i.test(s.host))
  const sitesFreshOk =
    connection.status === 'running' &&
    !!yt &&
    !!discord &&
    displaySiteStatus(yt, connection.status) === 'available' &&
    displaySiteStatus(discord, connection.status) === 'available'

  const modeSummary = (() => {
    if (!activeProfile) return null
    if (neverChecked) return null
    const parts: string[] = []
    parts.push(
      activeProfile.isAutoSelected ? t('home.modeAuto') : t('home.modeManual'),
    )
    if (activeProfile.lastCheckedAt) {
      parts.push(
        t('home.modeChecked', {
          when: formatRelative(activeProfile.lastCheckedAt, locale),
        }),
      )
    }
    if (sitesFreshOk) {
      parts.push(t('home.modeSitesOk'))
    } else if (connection.status === 'running') {
      parts.push(t('home.modeSitesMixed'))
    }
    return parts.join(' · ')
  })()

  const statusTone = (service: (typeof services)[number]) => {
    const key = siteStatusLabelKey(service, connection.status)
    switch (key) {
      case 'status.available':
        return { label: t(key), tone: 'success' as const }
      case 'status.unavailable':
        return { label: t(key), tone: 'error' as const }
      case 'status.checking':
        return { label: t(key), tone: 'warning' as const, pulse: true }
      default:
        return { label: t(key), tone: 'neutral' as const }
    }
  }

  return (
    <div>
      <PageHeader
        title={t('home.title')}
        description={t('home.description')}
        titleAddon={
          <Hint
            label={t('hints.homeLabel')}
            title={t('hints.homeTitle')}
            body={t('hints.homeBody')}
            wide
          />
        }
      />

      <div className={page.stack}>
        <ConnectionCard
          connection={connection}
          busy={busy.engine}
          onToggle={() => void toggleEngine()}
          onReset={() => void resetEngine()}
        />

        <section className={styles.modeCard}>
          <button
            type="button"
            className={styles.modeHeader}
            onClick={() => setModeOpen((v) => !v)}
            aria-expanded={modeOpen}
          >
            <div className={styles.modeHeaderCopy}>
              <div className={styles.modeEyebrow}>{t('home.activeMode')}</div>
              <div className={styles.modeName}>
                {activeProfile ? friendlyModeName(activeProfile.name) : t('home.noMode')}
              </div>
              {modeSummary && !modeOpen && (
                <div className={styles.modePreview}>{modeSummary}</div>
              )}
              {neverChecked && !modeOpen && (
                <div className={styles.modePreview}>{t('home.runSmartCheckHint')}</div>
              )}
            </div>
            <ChevronDown
              size={18}
              className={[styles.chevron, modeOpen ? styles.chevronOpen : ''].join(' ')}
            />
          </button>

          {neverChecked && (
            <div className={styles.modeCta}>
              <Button
                variant="primary"
                size="sm"
                onClick={() => navigate('/diagnostics')}
              >
                {t('home.runSmartCheck')}
              </Button>
            </div>
          )}

          {modeOpen && activeProfile && (
            <div className={styles.modeBody}>
              {modeSummary && <p className={styles.modeSummaryLine}>{modeSummary}</p>}
              <div className={page.kv}>
                <div className={page.kvRow}>
                  <span className={page.kvLabel}>{t('home.name')}</span>
                  <span className={page.kvValue}>{activeProfile.name}</span>
                </div>
                <div className={page.kvRow}>
                  <span className={page.kvLabel}>{t('home.type')}</span>
                  <span className={page.kvValue}>{activeProfile.strategyType}</span>
                </div>
                <div className={page.kvRow}>
                  <span className={page.kvLabel}>{t('home.source')}</span>
                  <span className={page.kvValue}>{t(sourceLabelKey(activeProfile.source))}</span>
                </div>
                <div className={page.kvRow}>
                  <span className={page.kvLabel}>{t('home.lastCheck')}</span>
                  <span className={page.kvValue}>
                    {formatDateTime(activeProfile.lastCheckedAt, locale)}
                  </span>
                </div>
              </div>
              <Link to="/profiles" className={page.linkBtn}>
                {t('home.manageModes')}
              </Link>
            </div>
          )}

          {modeOpen && !activeProfile && (
            <div className={styles.modeBody}>
              <p className={page.muted}>{t('home.noMode')}</p>
              <Link to="/profiles" className={page.linkBtn}>
                {t('home.manageModes')}
              </Link>
            </div>
          )}
        </section>

        <section className={styles.panel}>
          <div className={styles.panelHeader}>
            <h2 className={styles.panelTitle}>{t('home.sites')}</h2>
            <Link to="/services" className={page.linkBtn}>
              {t('home.openSites')}
            </Link>
          </div>
          <div className={styles.siteList}>
            {services.map((service) => {
              const shown = displaySiteStatus(service, connection.status)
              const st = statusTone(service)
              const icon = serviceIcon(service.name) ?? serviceIcon(service.host)
              const fresh =
                connection.status === 'running' && isSiteCheckFresh(service.lastCheckedAt)
              const metaKey = siteStatusLabelKey(service, connection.status)
              return (
                <button
                  key={service.id}
                  type="button"
                  className={[styles.siteRow, shown === 'unknown' ? styles.siteStale : ''].join(
                    ' ',
                  )}
                  disabled={busy[`service:${service.id}`] || service.status === 'checking'}
                  onClick={() => void checkService(service.id)}
                >
                  <div className={styles.siteIcon} aria-hidden>
                    {icon ? (
                      <img src={icon} alt="" width={16} height={16} />
                    ) : (
                      <LinkIcon size={14} />
                    )}
                  </div>
                  <div className={styles.siteCopy}>
                    <div className={styles.siteName}>{service.name}</div>
                    <div className={styles.siteHost}>{service.host}</div>
                  </div>
                  <div className={styles.siteMeta}>
                    <StatusBadge label={st.label} tone={st.tone} pulse={st.pulse} />
                    <div className={styles.siteStats}>
                      {fresh
                        ? `${formatLatency(service.latencyMs)} · ${formatRelative(service.lastCheckedAt, locale)}`
                        : t(metaKey)}
                    </div>
                  </div>
                </button>
              )
            })}
          </div>
        </section>

        <section className={styles.systemCard}>
          <div className={styles.systemTop}>
            <div>
              <div className={styles.systemEyebrow}>{t('home.system')}</div>
              <div className={styles.systemHeadline}>
                zapp {formatVersion(system.appVersion)}
              </div>
            </div>
            <Link to="/updates" className={page.linkBtn}>
              {t('nav.updates')} →
            </Link>
          </div>

          <div className={styles.systemSplit}>
            <div className={styles.systemCol}>
              <div className={styles.systemRow}>
                <span>{t('home.engineVersion')}</span>
                <strong>{system.engineVersion}</strong>
              </div>
              <div className={styles.systemRow}>
                <span>{t('home.strategiesDb')}</span>
                <strong>{system.strategiesVersion}</strong>
              </div>
            </div>
            <div className={styles.systemDivider} aria-hidden />
            <div className={styles.systemCol}>
              <div className={styles.systemRow}>
                <span>{t('home.lastUpdateCheck')}</span>
                <strong>{formatRelative(system.lastUpdateCheckAt, locale)}</strong>
              </div>
              <div className={styles.systemRow}>
                <span>{t('home.autoUpdate')}</span>
                <strong>
                  {system.autoUpdateEnabled ? t('status.enabled') : t('status.disabled')}
                </strong>
              </div>
            </div>
          </div>
        </section>
      </div>
    </div>
  )
}
