// ╔════════════════════════════════════════════════════╗
// ║           pages/Services/ServicesPage            ║
// ╚════════════════════════════════════════════════════╝

import { useState } from 'react'
import { RefreshCw } from 'lucide-react'
import { ServiceCard } from '@/components/domain/ServiceCard/ServiceCard'
import { PageHeader } from '@/components/layout/PageHeader/PageHeader'
import { Button } from '@/components/ui/Button/Button'
import { DataTable, type DataColumn } from '@/components/ui/DataTable/DataTable'
import { Dialog } from '@/components/ui/Dialog/Dialog'
import { StatusBadge } from '@/components/ui/StatusBadge/StatusBadge'
import { useAppState } from '@/context/AppStateContext'
import type { MonitoredService, ServiceCheckHistoryItem } from '@/types'
import { formatDateTime, formatLatency } from '@/utils/format'
import page from '../shared/page.module.css'

export function ServicesPage() {
  const { snapshot, checkService, checkAllServices, busy } = useAppState()
  const [selected, setSelected] = useState<MonitoredService | null>(null)

  if (!snapshot) return null

  const historyColumns: DataColumn<ServiceCheckHistoryItem>[] = [
    {
      key: 'time',
      header: 'Time',
      render: (row) => formatDateTime(row.checkedAt),
    },
    {
      key: 'status',
      header: 'Status',
      render: (row) =>
        row.status === 'available' ? (
          <StatusBadge label="Available" tone="success" />
        ) : row.status === 'unavailable' ? (
          <StatusBadge label="Unavailable" tone="error" />
        ) : (
          <StatusBadge label={row.status} tone="neutral" />
        ),
    },
    {
      key: 'latency',
      header: 'Latency',
      render: (row) => formatLatency(row.latencyMs),
    },
    {
      key: 'note',
      header: 'Note',
      render: (row) => row.note ?? '—',
    },
  ]

  return (
    <div>
      <PageHeader
        title="Services"
        description="Мониторинг доступности YouTube, Discord и пользовательских ресурсов."
        actions={
          <Button
            size="sm"
            variant="secondary"
            onClick={() => void checkAllServices()}
            disabled={busy['services:all']}
          >
            <RefreshCw size={14} />
            Check all
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
                Details
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
              Close
            </Button>
            {selected && (
              <Button
                variant="primary"
                disabled={busy[`service:${selected.id}`]}
                onClick={() => void checkService(selected.id)}
              >
                <RefreshCw size={14} />
                Recheck
              </Button>
            )}
          </>
        }
      >
        {selected && (
          <div className={page.stack}>
            <div className={page.kv}>
              <div className={page.kvRow}>
                <span className={page.kvLabel}>Latency</span>
                <span className={page.kvValue}>{formatLatency(selected.latencyMs)}</span>
              </div>
              <div className={page.kvRow}>
                <span className={page.kvLabel}>Last check</span>
                <span className={page.kvValue}>{formatDateTime(selected.lastCheckedAt)}</span>
              </div>
            </div>
            <DataTable
              columns={historyColumns}
              rows={selected.history}
              rowKey={(row) => row.id}
              emptyMessage="История проверок пуста."
            />
          </div>
        )}
      </Dialog>
    </div>
  )
}
