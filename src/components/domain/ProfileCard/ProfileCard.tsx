// ╔════════════════════════════════════════════════════╗
// ║    components/domain/ProfileCard/ProfileCard     ║
// ╚════════════════════════════════════════════════════╝

import { Button } from '@/components/ui/Button/Button'
import { StatusBadge } from '@/components/ui/StatusBadge/StatusBadge'
import { useLocale } from '@/context/LocaleContext'
import type { Profile } from '@/types'
import { formatDateTime } from '@/utils/format'
import { sourceLabelKey } from '@/utils/modeLabels'
import styles from './ProfileCard.module.css'

interface ProfileCardProps {
  profile: Profile
  busy?: boolean
  onActivate: () => void
  onDetails?: () => void
  compact?: boolean
}

export function ProfileCard({
  profile,
  busy,
  onActivate,
  onDetails,
  compact,
}: ProfileCardProps) {
  const { t, locale } = useLocale()

  return (
    <article
      className={[
        styles.card,
        profile.isActive ? styles.cardActive : '',
        compact ? styles.cardCompact : '',
      ].join(' ')}
    >
      <div className={styles.header}>
        <div>
          <h3 className={styles.title}>{profile.name}</h3>
          <p className={styles.subtitle}>{profile.strategyType}</p>
        </div>
        {profile.isActive ? (
          <StatusBadge label={t('modes.active')} tone="success" />
        ) : profile.isAutoSelected ? (
          <StatusBadge label={t('modes.auto')} tone="neutral" />
        ) : null}
      </div>

      {!compact && <p className={styles.description}>{profile.description}</p>}

      <div className={styles.meta}>
        <div>
          <div className={styles.metaLabel}>{t('modes.source')}</div>
          <div className={styles.metaValue}>{t(sourceLabelKey(profile.source))}</div>
        </div>
        <div>
          <div className={styles.metaLabel}>{t('modes.lastUpdated')}</div>
          <div className={styles.metaValue}>{formatDateTime(profile.lastUpdatedAt, locale)}</div>
        </div>
      </div>

      <div className={styles.actions}>
        {onDetails && (
          <Button size="sm" variant="ghost" onClick={onDetails}>
            {t('modes.details')}
          </Button>
        )}
        <Button
          size="sm"
          variant={profile.isActive ? 'secondary' : 'primary'}
          onClick={onActivate}
          disabled={busy || profile.isActive}
        >
          {profile.isActive ? t('modes.active') : t('modes.activate')}
        </Button>
      </div>
    </article>
  )
}
