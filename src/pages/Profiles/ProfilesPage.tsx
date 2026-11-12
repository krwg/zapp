// ╔════════════════════════════════════════════════════╗
// ║           pages/Profiles/ProfilesPage            ║
// ╚════════════════════════════════════════════════════╝

import { useMemo, useState } from 'react'
import { Download, Plus, Upload } from 'lucide-react'
import { ProfileCard } from '@/components/domain/ProfileCard/ProfileCard'
import { PageHeader } from '@/components/layout/PageHeader/PageHeader'
import { Button } from '@/components/ui/Button/Button'
import { Dialog } from '@/components/ui/Dialog/Dialog'
import { useAppState } from '@/context/AppStateContext'
import { useToast } from '@/context/ToastContext'
import type { Profile } from '@/types'
import { formatDateTime } from '@/utils/format'
import page from '../shared/page.module.css'

export function ProfilesPage() {
  const { snapshot, setActiveProfile, busy } = useAppState()
  const { push } = useToast()
  const [details, setDetails] = useState<Profile | null>(null)
  const [createOpen, setCreateOpen] = useState(false)

  const profiles = snapshot?.profiles ?? []
  const active = useMemo(() => profiles.find((p) => p.isActive) ?? null, [profiles])
  const auto = profiles.filter(
    (p) => (p.isAutoSelected || p.source === 'auto') && !p.isActive,
  )
  const rest = profiles.filter(
    (p) => !(p.isAutoSelected || p.source === 'auto') && !p.isActive,
  )

  return (
    <div>
      <PageHeader
        title="Profiles"
        description="Управление стратегиями и конфигурациями. Подходит и для обычного, и для продвинутого сценария."
        actions={
          <>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => push('Импорт будет доступен после подключения backend', 'neutral')}
            >
              <Upload size={14} />
              Import
            </Button>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => push('Экспорт будет доступен после подключения backend', 'neutral')}
            >
              <Download size={14} />
              Export
            </Button>
            <Button size="sm" variant="primary" onClick={() => setCreateOpen(true)}>
              <Plus size={14} />
              New profile
            </Button>
          </>
        }
      />

      <div className={page.stack}>
        {active && (
          <section className={page.section}>
            <h2 className={page.sectionTitle}>Active</h2>
            <div className={page.grid2}>
              <ProfileCard
                profile={active}
                busy={busy[`profile:${active.id}`]}
                onActivate={() => undefined}
                onDetails={() => setDetails(active)}
              />
              <div className={page.card}>
                <div className={page.cardTitle}>Quick facts</div>
                <div className={page.kv}>
                  <div className={page.kvRow}>
                    <span className={page.kvLabel}>Kind</span>
                    <span className={page.kvValue}>{active.kind}</span>
                  </div>
                  <div className={page.kvRow}>
                    <span className={page.kvLabel}>Source</span>
                    <span className={page.kvValue}>{active.source}</span>
                  </div>
                  <div className={page.kvRow}>
                    <span className={page.kvLabel}>Last updated</span>
                    <span className={page.kvValue}>{formatDateTime(active.lastUpdatedAt)}</span>
                  </div>
                </div>
              </div>
            </div>
          </section>
        )}

        {auto.length > 0 && (
          <section className={page.section}>
            <h2 className={page.sectionTitle}>Automatically selected</h2>
            <div className={page.grid3}>
              {auto.map((profile) => (
                <ProfileCard
                  key={profile.id}
                  profile={profile}
                  busy={busy[`profile:${profile.id}`]}
                  onActivate={() => void setActiveProfile(profile.id)}
                  onDetails={() => setDetails(profile)}
                />
              ))}
            </div>
          </section>
        )}

        <section className={page.section}>
          <h2 className={page.sectionTitle}>All strategies</h2>
          <div className={page.grid3}>
            {(rest.length ? rest : profiles).map((profile) => (
              <ProfileCard
                key={profile.id}
                profile={profile}
                busy={busy[`profile:${profile.id}`]}
                onActivate={() => void setActiveProfile(profile.id)}
                onDetails={() => setDetails(profile)}
              />
            ))}
          </div>
        </section>
      </div>

      <Dialog
        open={!!details}
        title={details?.name ?? ''}
        description={details?.description}
        onClose={() => setDetails(null)}
        footer={
          <Button variant="secondary" onClick={() => setDetails(null)}>
            Close
          </Button>
        }
      >
        {details && (
          <div className={page.kv}>
            <div className={page.kvRow}>
              <span className={page.kvLabel}>Strategy type</span>
              <span className={page.kvValue}>{details.strategyType}</span>
            </div>
            <div className={page.kvRow}>
              <span className={page.kvLabel}>Source</span>
              <span className={page.kvValue}>{details.source}</span>
            </div>
            <div className={page.kvRow}>
              <span className={page.kvLabel}>Kind</span>
              <span className={page.kvValue}>{details.kind}</span>
            </div>
            <div className={page.kvRow}>
              <span className={page.kvLabel}>Last checked</span>
              <span className={page.kvValue}>{formatDateTime(details.lastCheckedAt)}</span>
            </div>
            <div className={page.kvRow}>
              <span className={page.kvLabel}>Updated</span>
              <span className={page.kvValue}>{formatDateTime(details.lastUpdatedAt)}</span>
            </div>
          </div>
        )}
      </Dialog>

      <Dialog
        open={createOpen}
        title="New profile"
        description="Создание пользовательского профиля будет подключено к backend. Сейчас это UI-заглушка."
        onClose={() => setCreateOpen(false)}
        footer={
          <>
            <Button variant="ghost" onClick={() => setCreateOpen(false)}>
              Cancel
            </Button>
            <Button
              variant="primary"
              onClick={() => {
                setCreateOpen(false)
                push('Создание профиля доступно после интеграции с движком', 'warning')
              }}
            >
              Create
            </Button>
          </>
        }
      >
        <p className={page.muted}>
          Здесь появится форма имени, типа стратегии и параметров конфигурации.
        </p>
      </Dialog>
    </div>
  )
}
