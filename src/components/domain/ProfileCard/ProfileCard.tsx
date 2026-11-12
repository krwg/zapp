// ╔════════════════════════════════════════════════════╗
// ║    components/domain/ProfileCard/ProfileCard     ║
// ╚════════════════════════════════════════════════════╝

import { Button } from '@/components/ui/Button/Button'
import { StatusBadge } from '@/components/ui/StatusBadge/StatusBadge'
import type { Profile } from '@/types'
import { formatDateTime } from '@/utils/format'
import styles from './ProfileCard.module.css'

interface ProfileCardProps {
  profile: Profile
  busy?: boolean
  onActivate: () => void
  onDetails?: () => void
}

export function ProfileCard({ profile, busy, onActivate, onDetails }: ProfileCardProps) {
  return (
    <article className={[styles.card, profile.isActive ? styles.cardActive : ''].join(' ')}>
      <div className={styles.header}>
        <div>
          <h3 className={styles.title}>{profile.name}</h3>
          <p className={styles.subtitle}>{profile.strategyType}</p>
        </div>
        {profile.isActive ? (
          <StatusBadge label="Active" tone="success" />
        ) : profile.isAutoSelected ? (
          <StatusBadge label="Auto" tone="neutral" />
        ) : null}
      </div>

      <p className={styles.description}>{profile.description}</p>

      {profile.tags.length > 0 && (
        <div className={styles.tags}>
          {profile.tags.map((tag) => (
            <span key={tag} className={styles.tag}>
              {tag}
            </span>
          ))}
        </div>
      )}

      <div className={styles.meta}>
        <div>
          <div className={styles.metaLabel}>Source</div>
          <div className={styles.metaValue}>{profile.source}</div>
        </div>
        <div>
          <div className={styles.metaLabel}>Updated</div>
          <div className={styles.metaValue}>{formatDateTime(profile.lastUpdatedAt)}</div>
        </div>
      </div>

      <div className={styles.actions}>
        {onDetails && (
          <Button size="sm" variant="ghost" onClick={onDetails}>
            Details
          </Button>
        )}
        <Button
          size="sm"
          variant={profile.isActive ? 'secondary' : 'primary'}
          onClick={onActivate}
          disabled={busy || profile.isActive}
        >
          {profile.isActive ? 'Active' : 'Activate'}
        </Button>
      </div>
    </article>
  )
}
