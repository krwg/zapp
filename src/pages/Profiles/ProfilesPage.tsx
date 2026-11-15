// ╔════════════════════════════════════════════════════╗
// ║           pages/Profiles/ProfilesPage            ║
// ╚════════════════════════════════════════════════════╝

import { useMemo, useState } from 'react'
import { Download, Plus, Upload } from 'lucide-react'
import { ProfileCard } from '@/components/domain/ProfileCard/ProfileCard'
import { PageHeader } from '@/components/layout/PageHeader/PageHeader'
import { Button } from '@/components/ui/Button/Button'
import { Dialog } from '@/components/ui/Dialog/Dialog'
import { Hint } from '@/components/ui/Hint/Hint'
import { StatusBadge } from '@/components/ui/StatusBadge/StatusBadge'
import { useAppState } from '@/context/AppStateContext'
import { useLocale } from '@/context/LocaleContext'
import { useToast } from '@/context/ToastContext'
import type { Profile } from '@/types'
import { formatDateTime } from '@/utils/format'
import { sourceLabelKey } from '@/utils/modeLabels'
import page from '../shared/page.module.css'
import styles from './ProfilesPage.module.css'

export function ProfilesPage() {
  const { snapshot, setActiveProfile, busy } = useAppState()
  const { push } = useToast()
  const { t, locale } = useLocale()
  const [details, setDetails] = useState<Profile | null>(null)
  const [createOpen, setCreateOpen] = useState(false)

  const profiles = snapshot?.profiles ?? []
  const active = useMemo(() => profiles.find((p) => p.isActive) ?? null, [profiles])
  const suggested = profiles.filter(
    (p) => (p.isAutoSelected || p.source === 'auto') && !p.isActive,
  )
  const rest = profiles.filter(
    (p) => p.id !== active?.id && !(p.isAutoSelected || p.source === 'auto'),
  )

  return (
    <div>
      <PageHeader
        title={t('modes.title')}
        description={t('modes.description')}
        titleAddon={
          <Hint
            label={t('hints.modesLabel')}
            title={t('hints.modesTitle')}
            body={t('hints.modesBody')}
            wide
          />
        }
        actions={
          <>
            <Button
              size="sm"
              variant="ghost"
              onClick={() =>
                push({
                  title: t('toast.importSoonTitle'),
                  description: t('toast.importSoonDesc'),
                  tone: 'neutral',
                })
              }
            >
              <Upload size={14} />
              {t('modes.import')}
            </Button>
            <Button
              size="sm"
              variant="ghost"
              onClick={() =>
                push({
                  title: t('toast.exportSoonTitle'),
                  description: t('toast.exportSoonDesc'),
                  tone: 'neutral',
                })
              }
            >
              <Download size={14} />
              {t('modes.export')}
            </Button>
            <Button size="sm" variant="primary" onClick={() => setCreateOpen(true)}>
              <Plus size={14} />
              {t('modes.newMode')}
            </Button>
          </>
        }
      />

      <div className={page.stack}>
        {active && (
          <section className={styles.activeBlock}>
            <div className={styles.sectionLabel}>{t('modes.active')}</div>
            <article className={styles.activeCard}>
              <div className={styles.activeTop}>
                <div className={styles.activeCopy}>
                  <h3 className={styles.activeName}>{active.name}</h3>
                  <p className={styles.activeType}>{active.strategyType}</p>
                </div>
                <StatusBadge label={t('modes.active')} tone="success" />
              </div>
              <p className={styles.activeDesc}>{active.description}</p>
              <div className={styles.activeMeta}>
                <span>
                  {t('modes.source')}: <strong>{t(sourceLabelKey(active.source))}</strong>
                </span>
                <span className={styles.dotSep} aria-hidden>
                  ·
                </span>
                <span>
                  {t('modes.lastUpdated')}:{' '}
                  <strong>{formatDateTime(active.lastUpdatedAt, locale)}</strong>
                </span>
              </div>
              <div className={styles.activeActions}>
                <Button size="sm" variant="ghost" onClick={() => setDetails(active)}>
                  {t('modes.details')}
                </Button>
              </div>
            </article>
          </section>
        )}

        {suggested.length > 0 && (
          <section className={styles.listBlock}>
            <div className={styles.sectionLabel}>{t('modes.autoSelected')}</div>
            <div className={styles.list}>
              {suggested.map((profile) => (
                <ProfileCard
                  key={profile.id}
                  profile={profile}
                  compact
                  busy={busy[`profile:${profile.id}`]}
                  onActivate={() => void setActiveProfile(profile.id)}
                  onDetails={() => setDetails(profile)}
                />
              ))}
            </div>
          </section>
        )}

        <section className={styles.listBlock}>
          <div className={styles.sectionLabel}>{t('modes.allModes')}</div>
          {(rest.length ? rest : profiles.filter((p) => p.id !== active?.id)).length === 0 ? (
            <p className={page.muted}>{t('modes.empty')}</p>
          ) : (
            <div className={styles.list}>
              {(rest.length ? rest : profiles.filter((p) => p.id !== active?.id)).map((profile) => (
                <ProfileCard
                  key={profile.id}
                  profile={profile}
                  compact
                  busy={busy[`profile:${profile.id}`]}
                  onActivate={() => void setActiveProfile(profile.id)}
                  onDetails={() => setDetails(profile)}
                />
              ))}
            </div>
          )}
        </section>
      </div>

      <Dialog
        open={!!details}
        title={details?.name ?? ''}
        description={details?.description}
        onClose={() => setDetails(null)}
        footer={
          <Button variant="secondary" onClick={() => setDetails(null)}>
            {t('common.close')}
          </Button>
        }
      >
        {details && (
          <div className={page.kv}>
            <div className={page.kvRow}>
              <span className={page.kvLabel}>{t('modes.strategyType')}</span>
              <span className={page.kvValue}>{details.strategyType}</span>
            </div>
            <div className={page.kvRow}>
              <span className={page.kvLabel}>{t('modes.source')}</span>
              <span className={page.kvValue}>{t(sourceLabelKey(details.source))}</span>
            </div>
            <div className={page.kvRow}>
              <span className={page.kvLabel}>{t('modes.kind')}</span>
              <span className={page.kvValue}>{details.kind}</span>
            </div>
            <div className={page.kvRow}>
              <span className={page.kvLabel}>{t('modes.lastChecked')}</span>
              <span className={page.kvValue}>{formatDateTime(details.lastCheckedAt, locale)}</span>
            </div>
            <div className={page.kvRow}>
              <span className={page.kvLabel}>{t('modes.lastUpdated')}</span>
              <span className={page.kvValue}>{formatDateTime(details.lastUpdatedAt, locale)}</span>
            </div>
          </div>
        )}
      </Dialog>

      <Dialog
        open={createOpen}
        title={t('modes.createTitle')}
        description={t('modes.createBody')}
        onClose={() => setCreateOpen(false)}
        footer={
          <>
            <Button variant="ghost" onClick={() => setCreateOpen(false)}>
              {t('common.cancel')}
            </Button>
            <Button
              variant="primary"
              onClick={() => {
                setCreateOpen(false)
                push({
                  title: t('toast.createSoonTitle'),
                  description: t('toast.createSoonDesc'),
                  tone: 'warning',
                })
              }}
            >
              {t('modes.create')}
            </Button>
          </>
        }
      >
        <p className={page.muted}>{t('modes.createHint')}</p>
      </Dialog>
    </div>
  )
}
