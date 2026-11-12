// ╔════════════════════════════════════════════════════╗
// ║            pages/Updates/UpdatesPage             ║
// ╚════════════════════════════════════════════════════╝

import { RefreshCw } from 'lucide-react'
import { UpdateCard } from '@/components/domain/UpdateCard/UpdateCard'
import { PageHeader } from '@/components/layout/PageHeader/PageHeader'
import { Button } from '@/components/ui/Button/Button'
import { useAppState } from '@/context/AppStateContext'
import page from '../shared/page.module.css'

export function UpdatesPage() {
  const { updatesLive, checkUpdates, updateChannel, busy, snapshot } = useAppState()

  return (
    <div>
      <PageHeader
        title="Updates"
        description="Независимые каналы: приложение, движок zapret и база стратегий. Клиент не привязан к одной версии движка."
        actions={
          <Button
            size="sm"
            variant="secondary"
            onClick={() => void checkUpdates()}
            disabled={busy['updates:check']}
          >
            <RefreshCw size={14} />
            Check for updates
          </Button>
        }
      />

      <div className={page.stack}>
        <div className={page.card}>
          <div className={page.cardTitle}>Update model</div>
          <p className={page.muted}>
            zapp — клиент-ядро над zapret. Application, Engine и Strategies обновляются
            раздельно: можно подтянуть новый билд движка, не меняя UI, и наоборот.
            {snapshot
              ? ` Сейчас engine ${snapshot.system.engineVersion}, strategies ${snapshot.system.strategiesVersion}.`
              : ''}
          </p>
        </div>

        <div className={page.grid3}>
          {updatesLive.map((channel) => (
            <UpdateCard
              key={channel.id}
              channel={channel}
              busy={busy[`update:${channel.id}`]}
              onUpdate={() => void updateChannel(channel.id)}
            />
          ))}
        </div>
      </div>
    </div>
  )
}
