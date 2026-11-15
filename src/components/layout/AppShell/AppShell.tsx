// ╔════════════════════════════════════════════════════╗
// ║       components/layout/AppShell/AppShell        ║
// ╚════════════════════════════════════════════════════╝

import { Outlet, useLocation } from 'react-router-dom'
import { isTauriRuntime } from '@/api/tauriClient'
import { BootLoader } from '@/components/ui/BootLoader/BootLoader'
import { useAppState } from '@/context/AppStateContext'
import { Sidebar } from '../Sidebar/Sidebar'
import styles from './AppShell.module.css'

export function AppShell() {
  const { loading, snapshot } = useAppState()
  const location = useLocation()
  const tauri = isTauriRuntime()

  if (loading || !snapshot) {
    return (
      <div className={[styles.shell, styles.shellBoot].join(' ')}>
        <div className={styles.loading}>
          <BootLoader />
        </div>
      </div>
    )
  }

  return (
    <div className={styles.shell}>
      <Sidebar
        appVersion={snapshot.about.version}
        engineStatus={snapshot.connection.status}
        connection={snapshot.connection}
        hideStatus={location.pathname === '/'}
      />
      <main className={styles.main}>
        <div className={styles.content}>
          {!tauri && (
            <div className={styles.demoBanner}>
              <span className={styles.demoMark}>Demo</span>
              Браузерный режим: mock API. Для полного zapp запустите npm run tauri:dev.
            </div>
          )}
          <div className="pageEnter" key={location.pathname}>
            <Outlet />
          </div>
        </div>
      </main>
    </div>
  )
}
