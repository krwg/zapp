// ╔════════════════════════════════════════════════════╗
// ║           pages/Services/ServicesPage            ║
// ╚════════════════════════════════════════════════════╝

import { useState } from 'react'
import { RefreshCw } from 'lucide-react'
import { ServiceCard } from '@/components/domain/ServiceCard/ServiceCard'
import { PageHeader } from '@/components/layout/PageHeader/PageHeader'
import { Button } from '@/components/ui/Button/Button'
import { Hint } from '@/components/ui/Hint/Hint'
import { DataTable, type DataColumn } from '@/components/ui/DataTable/DataTable'
import { Dialog } from '@/components/ui/Dialog/Dialog'
import { StatusBadge } from '@/components/ui/StatusBadge/StatusBadge'
import { useAppState } from '@/context/AppStateContext'
import { useLocale } from '@/context/LocaleContext'
import type { MonitoredService, ServiceCheckHistoryItem } from '@/types'
import { formatDateTime, formatLatency } from '@/utils/format'
import page from '../shared/page.module.css'

export function ServicesPage() {
  const { snapshot, checkService, checkAllServices, busy } = useAppState()
  const { t, locale } = useLocale()
  const [selected, setSelected] = useState<MonitoredService | null>(null)

  if (!snapshot) return null

  const historyColumns: DataColumn<ServiceCheckHistoryItem>[] = [
    {
      key: 'time',
      header: t('sites.time'),
      render: (row) => formatDateTime(row.checkedAt, locale),
    },
    {
      key: 'status',
      header: t('sites.status'),
      render: (row) =>
        row.status === 'available' ? (
          <StatusBadge label={t('status.available')} tone="success" />
        ) : row.status === 'unavailable' ? (
          <StatusBadge label={t('status.unavailable')} tone="error" />
        ) : (
          <StatusBadge label={row.status} tone="neutral" />
        ),
    },
    {
      key: 'latency',
      header: t('sites.latency'),
      render: (row) => formatLatency(row.latencyMs),
    },
    {
      key: 'note',
      header: t('sites.note'),
      render: (row) => row.note ?? t('common.dash'),
    },
  ]

  return (
    <div>
      <PageHeader
        title={t('sites.title')}
        description={t('sites.description')}
        titleAddon={
          <Hint
            label={t('hints.sitesLabel')}
            title={t('hints.sitesTitle')}
            body={t('hints.sitesBody')}
            wide
          />
        }
        actions={
          <Button
            size="sm"
            variant="secondary"
            onClick={() => void checkAllServices()}
            disabled={busy['services:all']}
          >
            <RefreshCw size={14} />
            {t('sites.checkAll')}
          </Button>
        }
      />

      <div className={page.stack}>
        <div className={page.grid3}>
          {snapshot.services.map((service) => (
            <div key={service.id} style={{ display: 'grid', gap: 8 }}>
              <ServiceCard
                service={service}
                busy={busy[`service:${service.id}`]}
                onCheck={() => void checkService(service.id)}
              />
              <Button size="sm" variant="ghost" onClick={() => setSelected(service)}>
                {t('common.details')}
              </Button>
            </div>
          ))}
        </div>
      </div>

      <Dialog
        open={!!selected}
        title={selected?.name ?? ''}
        description={selected ? `${selected.host} · ${selected.diagnosticSummary}` : undefined}
        onClose={() => setSelected(null)}
        footer={
          <>
            <Button variant="ghost" onClick={() => setSelected(null)}>
              {t('common.close')}
            </Button>
            {selected && (
              <Button
                variant="primary"
                disabled={busy[`service:${selected.id}`]}
                onClick={() => void checkService(selected.id)}
              >
                <RefreshCw size={14} />
                {t('sites.recheck')}
              </Button>
            )}
          </>
        }
      >
        {selected && (
          <div className={page.stack}>
            <div className={page.kv}>
              <div className={page.kvRow}>
                <span className={page.kvLabel}>{t('sites.latency')}</span>
                <span className={page.kvValue}>{formatLatency(selected.latencyMs)}</span>
              </div>
              <div className={page.kvRow}>
                <span className={page.kvLabel}>{t('sites.lastCheck')}</span>
                <span className={page.kvValue}>
                  {formatDateTime(selected.lastCheckedAt, locale)}
                </span>
              </div>
            </div>
            <DataTable
              columns={historyColumns}
              rows={selected.history}
              rowKey={(row) => row.id}
              emptyMessage={t('sites.emptyHistory')}
            />
          </div>
        )}
      </Dialog>
    </div>
  )
}
