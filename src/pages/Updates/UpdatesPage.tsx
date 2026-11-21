// ╔════════════════════════════════════════════════════╗
// ║            pages/Updates/UpdatesPage             ║
// ╚════════════════════════════════════════════════════╝

import { FolderOpen, RefreshCw } from 'lucide-react'
import { UpdateCard } from '@/components/domain/UpdateCard/UpdateCard'
import { PageHeader } from '@/components/layout/PageHeader/PageHeader'
import { Button } from '@/components/ui/Button/Button'
import { Hint } from '@/components/ui/Hint/Hint'
import { useAppState } from '@/context/AppStateContext'
import { useLocale } from '@/context/LocaleContext'
import { formatVersion } from '@/utils/version'
import styles from './UpdatesPage.module.css'

export function UpdatesPage() {
  const { updatesLive, checkUpdates, updateChannel, importEnginePack, busy, snapshot } =
    useAppState()
  const { t } = useLocale()

  const hintBody = snapshot
    ? `${t('hints.updatesBody')} ${t('home.appVersion')} ${formatVersion(snapshot.system.appVersion)}, ${t('home.engineVersion')} ${snapshot.system.engineVersion}.`
    : t('hints.updatesBody')

  return (
    <div>
      <PageHeader
        title={t('updates.title')}
        description={t('updates.description')}
        titleAddon={
          <Hint
            label={t('hints.updatesLabel')}
            title={t('hints.updatesTitle')}
            body={hintBody}
            wide
          />
        }
        actions={
          <>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => void importEnginePack()}
              disabled={busy['update:engine']}
            >
              <FolderOpen size={14} />
              {t('updates.importZip')}
            </Button>
            <Button
              size="sm"
              variant="secondary"
              onClick={() => void checkUpdates()}
              disabled={busy['updates:check']}
            >
              <RefreshCw size={14} />
              {t('updates.check')}
            </Button>
          </>
        }
      />

      <div className={styles.stack}>
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
  )
}
