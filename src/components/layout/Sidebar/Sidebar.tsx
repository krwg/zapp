// ╔════════════════════════════════════════════════════╗
// ║        components/layout/Sidebar/Sidebar         ║
// ╚════════════════════════════════════════════════════╝

import { Link, NavLink } from 'react-router-dom'
import {
  Activity,
  Boxes,
  Gauge,
  Info,
  LayoutDashboard,
  RefreshCw,
  Settings,
  Shield,
} from 'lucide-react'
import type { EngineStatus } from '@/types'
import styles from './Sidebar.module.css'

const NAV = [
  { to: '/', label: 'Overview', icon: LayoutDashboard, end: true },
  { to: '/diagnostics', label: 'Diagnostics', icon: Activity },
  { to: '/profiles', label: 'Profiles', icon: Boxes },
  { to: '/services', label: 'Services', icon: Gauge },
  { to: '/updates', label: 'Updates', icon: RefreshCw },
] as const

interface SidebarProps {
  appVersion: string
  engineStatus: EngineStatus
}

function statusLabel(status: EngineStatus): string {
  switch (status) {
    case 'running':
      return 'Running'
    case 'stopped':
      return 'Stopped'
    case 'starting':
      return 'Starting'
    case 'stopping':
      return 'Stopping'
    case 'error':
      return 'Error'
  }
}

function dotClass(status: EngineStatus): string {
  if (status === 'running') return styles.dotRunning
  if (status === 'error') return styles.dotError
  if (status === 'starting' || status === 'stopping') return styles.dotBusy
  return ''
}

export function Sidebar({ appVersion, engineStatus }: SidebarProps) {
  return (
    <aside className={styles.sidebar}>
      <div className={styles.brand}>
        <div className={styles.logo} aria-hidden>
          <Shield size={18} strokeWidth={2.25} />
        </div>
        <div className={styles.brandText}>
          <div className={styles.name}>zapp</div>
          <div className={styles.version}>v{appVersion}</div>
        </div>
      </div>

      <nav className={styles.nav} aria-label="Основная навигация">
        {NAV.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            end={'end' in item ? item.end : false}
            className={({ isActive }) =>
              [styles.link, isActive ? styles.linkActive : ''].join(' ')
            }
          >
            <item.icon size={16} strokeWidth={1.75} />
            {item.label}
          </NavLink>
        ))}
      </nav>

      <div className={styles.footer}>
        <div className={styles.engineStatus}>
          <span className={[styles.dot, dotClass(engineStatus)].join(' ')} aria-hidden />
          <div className={styles.engineCopy}>
            <div className={styles.engineLabel}>Engine</div>
            <div className={styles.engineValue}>{statusLabel(engineStatus)}</div>
          </div>
        </div>

        <NavLink
          to="/settings"
          className={({ isActive }) => [styles.link, isActive ? styles.linkActive : ''].join(' ')}
        >
          <Settings size={16} strokeWidth={1.75} />
          Settings
        </NavLink>
        <Link to="/settings#about" className={styles.link}>
          <Info size={16} strokeWidth={1.75} />
          About
        </Link>
      </div>
    </aside>
  )
}
