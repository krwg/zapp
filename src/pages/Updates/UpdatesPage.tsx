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
            Application качается с GitHub <code>krwg/zapp</code> (msi/exe/zip). Engine и
            Strategies — с Flowseal releases. При включённых Auto-check / Auto-install
            проверка и установка идут сами при старте; иначе — кнопками ниже.
            {snapshot
              ? ` Сейчас app ${snapshot.system.appVersion}, engine ${snapshot.system.engineVersion}.`
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
