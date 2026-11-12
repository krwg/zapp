// ╔════════════════════════════════════════════════════╗
// ║        pages/Diagnostics/DiagnosticsPage         ║
// ╚════════════════════════════════════════════════════╝

import { DiagnosticProgress } from '@/components/domain/DiagnosticProgress/DiagnosticProgress'
import { PageHeader } from '@/components/layout/PageHeader/PageHeader'
import { Button } from '@/components/ui/Button/Button'
import { DataTable, type DataColumn } from '@/components/ui/DataTable/DataTable'
import { StatusBadge } from '@/components/ui/StatusBadge/StatusBadge'
import { useAppState } from '@/context/AppStateContext'
import type { DiagnosticStrategyResult } from '@/types'
import { formatDateTime, formatLatency } from '@/utils/format'
import page from '../shared/page.module.css'

export function DiagnosticsPage() {
  const {
    diagnosticsLive,
    startDiagnostics,
    cancelDiagnostics,
    selectDiagnosticStrategy,
    busy,
  } = useAppState()

  if (!diagnosticsLive) return null

  const columns: DataColumn<DiagnosticStrategyResult>[] = [
    {
      key: 'name',
      header: 'Strategy',
      render: (row) => (
        <div>
          <div style={{ fontWeight: 500 }}>{row.name}</div>
          <div style={{ fontSize: 12, color: 'var(--text-tertiary)', marginTop: 2 }}>
            {row.strategyType}
          </div>
        </div>
      ),
    },
    {
      key: 'result',
      header: 'Result',
      width: '120px',
      render: (row) =>
        row.success ? (
          <StatusBadge label="Working" tone="success" />
        ) : (
          <StatusBadge label="Failed" tone="error" />
        ),
    },
    {
      key: 'latency',
      header: 'Latency',
      width: '100px',
      render: (row) => formatLatency(row.latencyMs),
    },
    {
      key: 'score',
      header: 'Score',
      width: '80px',
      render: (row) => row.score,
    },
    {
      key: 'notes',
      header: 'Notes',
      render: (row) => row.notes,
    },
    {
      key: 'tested',
      header: 'Tested',
      width: '120px',
      render: (row) => formatDateTime(row.testedAt),
    },
    {
      key: 'action',
      header: '',
      width: '110px',
      render: (row) => (
        <Button
          size="sm"
          variant="secondary"
          disabled={!row.success || busy[`diag-select:${row.id}`]}
          onClick={() => void selectDiagnosticStrategy(row.id)}
        >
          Use
        </Button>
      ),
    },
  ]

  const sorted = [...diagnosticsLive.results].sort((a, b) => b.score - a.score)

  return (
    <div>
      <PageHeader
        title="Diagnostics"
        description="zapp прогоняет все профили скрыто, без окон zapret, выбирает лучший и включает его."
      />

      <div className={page.stack}>
        <DiagnosticProgress
          run={diagnosticsLive}
          busy={busy.diagnostics}
          onStart={() => void startDiagnostics()}
          onCancel={() => void cancelDiagnostics()}
        />

        <section className={page.section}>
          <div className={page.sectionHeader}>
            <h2 className={page.sectionTitle}>Results</h2>
            <span className={page.muted}>
              {diagnosticsLive.workingCount} working of {diagnosticsLive.results.length}
            </span>
          </div>
          <DataTable
            columns={columns}
            rows={sorted}
            rowKey={(row) => row.id}
            emptyMessage="Запустите диагностику — zapp проверит все стратегии."
          />
        </section>
      </div>
    </div>
  )
}
