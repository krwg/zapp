// ╔════════════════════════════════════════════════════╗
// ║           pages/Overview/OverviewPage            ║
// ╚════════════════════════════════════════════════════╝

import { Link } from 'react-router-dom'
import { ConnectionCard } from '@/components/domain/ConnectionCard/ConnectionCard'
import { ServiceCard } from '@/components/domain/ServiceCard/ServiceCard'
import { PageHeader } from '@/components/layout/PageHeader/PageHeader'
import { useAppState } from '@/context/AppStateContext'
import { formatDateTime, formatRelative } from '@/utils/format'
import page from '../shared/page.module.css'

export function OverviewPage() {
  const { snapshot, toggleEngine, resetEngine, checkService, busy } = useAppState()
  if (!snapshot) return null

  const { connection, activeProfile, services, system } = snapshot

  return (
    <div>
      <PageHeader
        title="Overview"
        description="Состояние движка, активный профиль и доступность ключевых сервисов."
      />

      <div className={page.stack}>
        {!system.elevated && (
          <section className={page.card}>
            <div className={page.cardTitle}>Administrator</div>
            <p className={page.muted}>
              При Start один раз подтвердите UAC для фонового helper. Дальше winws
              управляется без окон zapret и без повторных запросов прав.
            </p>
          </section>
        )}

        <div className={page.grid2}>
          <ConnectionCard
            connection={connection}
            busy={busy.engine}
            onToggle={() => void toggleEngine()}
            onReset={() => void resetEngine()}
          />

          <section className={page.card}>
            <div className={page.cardTitle}>Active Profile</div>
            {activeProfile ? (
              <div className={page.kv}>
                <div className={page.kvRow}>
                  <span className={page.kvLabel}>Name</span>
                  <span className={page.kvValue}>{activeProfile.name}</span>
                </div>
                <div className={page.kvRow}>
                  <span className={page.kvLabel}>Strategy</span>
                  <span className={page.kvValue}>{activeProfile.strategyType}</span>
                </div>
                <div className={page.kvRow}>
                  <span className={page.kvLabel}>Source</span>
                  <span className={page.kvValue}>{activeProfile.source}</span>
                </div>
                <div className={page.kvRow}>
                  <span className={page.kvLabel}>Last check</span>
                  <span className={page.kvValue}>
                    {formatDateTime(activeProfile.lastCheckedAt)}
                  </span>
                </div>
                <div style={{ paddingTop: 8 }}>
                  <Link to="/profiles" className={page.linkBtn}>
                    Manage profile →
                  </Link>
                </div>
              </div>
            ) : (
              <p className={page.muted}>Профиль не выбран</p>
            )}
          </section>
        </div>

        <section className={page.section}>
          <div className={page.sectionHeader}>
            <h2 className={page.sectionTitle}>Services</h2>
            <Link to="/services" className={page.linkBtn}>
              Open services →
            </Link>
          </div>
          <div className={page.grid3}>
            {services.map((service) => (
              <ServiceCard
                key={service.id}
                service={service}
                compact
                busy={busy[`service:${service.id}`]}
                onCheck={() => void checkService(service.id)}
              />
            ))}
          </div>
        </section>

        <section className={page.card}>
          <div className={page.cardTitle}>System Information</div>
          <div className={page.kv}>
            <div className={page.kvRow}>
              <span className={page.kvLabel}>Engine</span>
              <span className={page.kvValue}>{system.engineVersion}</span>
            </div>
            <div className={page.kvRow}>
              <span className={page.kvLabel}>Strategies DB</span>
              <span className={page.kvValue}>{system.strategiesVersion}</span>
            </div>
            <div className={page.kvRow}>
              <span className={page.kvLabel}>Last update check</span>
              <span className={page.kvValue}>{formatRelative(system.lastUpdateCheckAt)}</span>
            </div>
            <div className={page.kvRow}>
              <span className={page.kvLabel}>Auto-update</span>
              <span className={page.kvValue}>{system.autoUpdateEnabled ? 'Enabled' : 'Disabled'}</span>
            </div>
          </div>
        </section>
      </div>
    </div>
  )
}
