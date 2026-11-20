// ╔════════════════════════════════════════════════════╗
// ║              pages/About/AboutPage               ║
// ╚════════════════════════════════════════════════════╝

import { ExternalLink } from 'lucide-react'
import { PageHeader } from '@/components/layout/PageHeader/PageHeader'
import { useAppState } from '@/context/AppStateContext'
import { useLocale } from '@/context/LocaleContext'
import shieldMark from '@/assets/brand/shield-128.png'
import { formatVersion } from '@/utils/version'
import styles from './AboutPage.module.css'

const AUTHOR_URL = 'https://krwg.github.io/krwg/'
const YEAR = 2026

export function AboutPage() {
  const { snapshot } = useAppState()
  const { t } = useLocale()
  if (!snapshot) return null

  const { about } = snapshot

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
      </div>
    </div>
  )
}
