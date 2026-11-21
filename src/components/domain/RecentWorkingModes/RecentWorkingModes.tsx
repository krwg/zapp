// ╔════════════════════════════════════════════════════╗
// ║   components/domain/RecentWorkingModes           ║
// ╚════════════════════════════════════════════════════╝

import { Button } from '@/components/ui/Button/Button'
import { useLocale } from '@/context/LocaleContext'
import type { WorkingModeEntry } from '@/types'
import { formatRelative } from '@/utils/format'
import styles from './RecentWorkingModes.module.css'

interface RecentWorkingModesProps {
  modes: WorkingModeEntry[]
  busyKey?: string
  busy?: Record<string, boolean>
  onApply: (strategyName: string) => void
}

function friendlyName(name: string): string {
  const cut = name.indexOf(' (')
  if (cut > 0) return name.slice(0, cut)
  return name
}

export function RecentWorkingModes({
  modes,
  busy = {},
  onApply,
}: RecentWorkingModesProps) {
  const { t, locale } = useLocale()
  if (!modes.length) return null

  const list = modes.slice(0, 3)

  return (
    <section className={styles.panel}>
      <div className={styles.head}>
        <h2 className={styles.title}>{t('home.recentModes')}</h2>
      </div>
      <div className={styles.list}>
        {list.map((m) => {
          const key = `working:${m.strategyName}`
          return (
            <div key={`${m.strategyName}:${m.testedAt}`} className={styles.row}>
              <div className={styles.copy}>
                <div className={styles.name}>{friendlyName(m.strategyName)}</div>
                <div className={styles.meta}>
                  {m.score}/{m.softScore} · {formatRelative(m.testedAt, locale)}
                </div>
              </div>
              <Button
                size="sm"
                variant="secondary"
                disabled={!!busy[key]}
                onClick={() => onApply(m.strategyName)}
              >
                {t('home.reapplyMode')}
              </Button>
            </div>
          )
        })}
      </div>
    </section>
  )
}
