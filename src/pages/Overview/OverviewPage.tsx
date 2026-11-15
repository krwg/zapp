// ╔════════════════════════════════════════════════════╗
// ║           pages/Overview/OverviewPage            ║
// ╚════════════════════════════════════════════════════╝

import { useState } from 'react'
import { ChevronDown, Link as LinkIcon } from 'lucide-react'
import { Link } from 'react-router-dom'
import { ConnectionCard } from '@/components/domain/ConnectionCard/ConnectionCard'
import { PageHeader } from '@/components/layout/PageHeader/PageHeader'
import { Hint } from '@/components/ui/Hint/Hint'
import { StatusBadge } from '@/components/ui/StatusBadge/StatusBadge'

import { useAppState } from '@/context/AppStateContext'
import { useLocale } from '@/context/LocaleContext'
import type { AvailabilityStatus } from '@/types'
import { formatDateTime, formatLatency, formatRelative } from '@/utils/format'
import { sourceLabelKey } from '@/utils/modeLabels'
import { serviceIcon } from '@/utils/serviceIcons'
import { formatVersion } from '@/utils/version'
import page from '../shared/page.module.css'
import styles from './OverviewPage.module.css'

export function OverviewPage() {
  const { snapshot, toggleEngine, resetEngine, checkService, busy } = useAppState()
  const { t, locale } = useLocale()
  const [modeOpen, setModeOpen] = useState(false)

  if (!snapshot) return null

  const { connection, activeProfile, services, system } = snapshot

  const statusTone = (status: AvailabilityStatus) => {
    switch (status) {
      case 'available':
        return { label: t('status.available'), tone: 'success' as const }
      case 'unavailable':
        return { label: t('status.unavailable'), tone: 'error' as const }
      case 'checking':
        return { label: t('status.checking'), tone: 'warning' as const, pulse: true }
      default:
        return { label: t('status.unknown'), tone: 'neutral' as const }
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
                {activeProfile?.name ?? t('home.noMode')}
              </div>
              {activeProfile && !modeOpen && (
                <div className={styles.modePreview}>{activeProfile.strategyType}</div>
              )}
            </div>
            <ChevronDown
              size={18}
              className={[styles.chevron, modeOpen ? styles.chevronOpen : ''].join(' ')}
            />
          </button>

          {modeOpen && activeProfile && (
            <div className={styles.modeBody}>
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
              const st = statusTone(service.status)
              const icon = serviceIcon(service.name) ?? serviceIcon(service.host)
              return (
                <button
                  key={service.id}
                  type="button"
                  className={styles.siteRow}
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
                      {formatLatency(service.latencyMs)} ·{' '}
                      {formatRelative(service.lastCheckedAt, locale)}
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
