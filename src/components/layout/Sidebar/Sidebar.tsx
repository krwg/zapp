// ╔════════════════════════════════════════════════════╗
// ║        components/layout/Sidebar/Sidebar         ║
// ╚════════════════════════════════════════════════════╝

import { useEffect, useState } from 'react'
import { NavLink } from 'react-router-dom'
import {
  Activity,
  Boxes,
  Gauge,
  Info,
  LayoutDashboard,
  RefreshCw,
  Settings,
} from 'lucide-react'
import { useLocale } from '@/context/LocaleContext'
import { useUiPrefs } from '@/context/UiPrefsContext'
import type { ConnectionState, EngineStatus } from '@/types'
import type { MessageKey } from '@/i18n'
import shieldMark from '@/assets/brand/shield-64.png'
import { formatUptime } from '@/utils/format'
import { formatVersion } from '@/utils/version'
import styles from './Sidebar.module.css'

const NAV: { to: string; labelKey: MessageKey; icon: typeof LayoutDashboard; end?: boolean }[] = [
  { to: '/', labelKey: 'nav.home', icon: LayoutDashboard, end: true },
  { to: '/diagnostics', labelKey: 'nav.smartCheck', icon: Activity },
  { to: '/profiles', labelKey: 'nav.modes', icon: Boxes },
  { to: '/services', labelKey: 'nav.sites', icon: Gauge },
  { to: '/updates', labelKey: 'nav.updates', icon: RefreshCw },
]

interface SidebarProps {
  appVersion: string
  engineStatus: EngineStatus
  connection: ConnectionState
}

function statusLabel(status: EngineStatus, t: (key: MessageKey) => string): string {
  switch (status) {
    case 'running':
      return t('status.running')
    case 'stopped':
      return t('status.stopped')
    case 'starting':
      return t('status.starting')
    case 'stopping':
      return t('status.stopping')
    case 'error':
      return t('status.error')
  }
}

function dotClass(status: EngineStatus): string {
  if (status === 'running') return styles.dotRunning
  if (status === 'error') return styles.dotError
  if (status === 'starting' || status === 'stopping') return styles.dotBusy
  return ''
}

export function Sidebar({ appVersion, engineStatus, connection }: SidebarProps) {
  const { t, locale } = useLocale()
  const { showSidebarSession } = useUiPrefs()
  const [tick, setTick] = useState(() => Date.now())

  useEffect(() => {
    if (connection.status !== 'running') return
    const id = window.setInterval(() => setTick(Date.now()), 1000)
    return () => window.clearInterval(id)
  }, [connection.status])

  const uptimeSeconds =
    connection.status === 'running' && connection.sessionStartedAt
      ? Math.max(
          0,
          Math.floor((tick - new Date(connection.sessionStartedAt).getTime()) / 1000),
        )
      : connection.uptimeSeconds

  const showTime = showSidebarSession && connection.status === 'running'

  return (
    <aside className={styles.sidebar}>
      <div className={styles.brand}>
        <div className={styles.logo} aria-hidden>
          <img src={shieldMark} alt="" className={styles.logoImg} width={36} height={36} />
        </div>
        <div className={styles.brandText}>
          <div className={styles.name}>{t('brand.name')}</div>
          <div className={styles.version}>{formatVersion(appVersion)}</div>
        </div>
      </div>

      <nav className={styles.nav} aria-label={t('nav.ariaMain')}>
        {NAV.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            end={item.end ?? false}
            className={({ isActive }) =>
              [styles.link, isActive ? styles.linkActive : ''].join(' ')
            }
          >
            <item.icon size={16} strokeWidth={1.75} />
            {t(item.labelKey)}
          </NavLink>
        ))}
      </nav>

      <div className={styles.footer}>
        <div
          className={styles.statusChip}
          role="status"
          aria-label={`${t('nav.protection')}: ${statusLabel(engineStatus, t)}`}
        >
          <span className={[styles.dot, dotClass(engineStatus)].join(' ')} aria-hidden />
          <div className={styles.statusText}>
            <span className={styles.statusLabel}>{t('nav.protection')}</span>
            <span className={styles.statusMain}>{statusLabel(engineStatus, t)}</span>
            {showTime && (
              <span className={styles.statusTime}>
                {formatUptime(uptimeSeconds, { withSeconds: true, locale })}
              </span>
            )}
          </div>
        </div>

        <NavLink
          to="/settings"
          className={({ isActive }) => [styles.link, isActive ? styles.linkActive : ''].join(' ')}
        >
          <Settings size={16} strokeWidth={1.75} />
          {t('nav.settings')}
        </NavLink>
        <NavLink
          to="/about"
          className={({ isActive }) => [styles.link, isActive ? styles.linkActive : ''].join(' ')}
        >
          <Info size={16} strokeWidth={1.75} />
          {t('nav.about')}
        </NavLink>
      </div>
    </aside>
  )
}
