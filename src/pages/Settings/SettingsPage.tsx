// ╔════════════════════════════════════════════════════╗
// ║           pages/Settings/SettingsPage            ║
// ╚════════════════════════════════════════════════════╝

import {
  SettingsGroup,
  SettingsRow,
  settingsStyles,
} from '@/components/domain/SettingsGroup/SettingsGroup'
import { PageHeader } from '@/components/layout/PageHeader/PageHeader'
import { Toggle } from '@/components/ui/Toggle/Toggle'
import { useAppState } from '@/context/AppStateContext'
import { useTheme } from '@/context/ThemeContext'
import type { ThemeMode } from '@/types'
import page from '../shared/page.module.css'

const THEME_OPTIONS: { value: ThemeMode; label: string }[] = [
  { value: 'light', label: 'Light' },
  { value: 'dark', label: 'Dark' },
  { value: 'system', label: 'System' },
]

export function SettingsPage() {
  const { snapshot, updateSettings, busy } = useAppState()
  const { mode, setMode } = useTheme()

  if (!snapshot) return null
  const { settings, about } = snapshot
  const disabled = !!busy.settings

  return (
    <div>
      <PageHeader
        title="Settings"
        description="Внешний вид, автозапуск, обновления, диагностика и расширенные параметры."
      />

      <div className={page.stack}>
        <SettingsGroup title="Appearance" description="Тема интерфейса.">
          <SettingsRow title="Theme" hint="Light, Dark или следовать системе.">
            <div className={settingsStyles.segment} role="group" aria-label="Theme">
              {THEME_OPTIONS.map((opt) => (
                <button
                  key={opt.value}
                  type="button"
                  className={[
                    settingsStyles.segmentBtn,
                    mode === opt.value ? settingsStyles.segmentBtnActive : '',
                  ].join(' ')}
                  disabled={disabled}
                  onClick={() => {
                    setMode(opt.value)
                    void updateSettings({ appearance: opt.value })
                  }}
                >
                  {opt.label}
                </button>
              ))}
            </div>
          </SettingsRow>
        </SettingsGroup>

        <SettingsGroup title="Startup" description="Поведение при входе в систему.">
          <SettingsRow title="Launch at startup" hint="Запускать zapp вместе с Windows.">
            <Toggle
              checked={settings.startWithSystem}
              disabled={disabled}
              onChange={(checked) => void updateSettings({ startWithSystem: checked })}
            />
          </SettingsRow>
          <SettingsRow title="Minimize to tray" hint="Скрывать окно в трей вместо закрытия.">
            <Toggle
              checked={settings.minimizeToTray}
              disabled={disabled}
              onChange={(checked) => void updateSettings({ minimizeToTray: checked })}
            />
          </SettingsRow>
        </SettingsGroup>

        <SettingsGroup
          title="Updates"
          description="Автопроверка каналов Application / Engine / Strategies."
        >
          <SettingsRow title="Auto-check updates">
            <Toggle
              checked={settings.autoCheckUpdates}
              disabled={disabled}
              onChange={(checked) => void updateSettings({ autoCheckUpdates: checked })}
            />
          </SettingsRow>
          <SettingsRow
            title="Auto-install updates"
            hint="Устанавливать доступные обновления без подтверждения."
          >
            <Toggle
              checked={settings.autoInstallUpdates}
              disabled={disabled}
              onChange={(checked) => void updateSettings({ autoInstallUpdates: checked })}
            />
          </SettingsRow>
        </SettingsGroup>

        <SettingsGroup title="Diagnostics" description="Периодический автоподбор стратегий.">
          <SettingsRow title="Automatic diagnostics">
            <Toggle
              checked={settings.autoDiagnostics}
              disabled={disabled}
              onChange={(checked) => void updateSettings({ autoDiagnostics: checked })}
            />
          </SettingsRow>
          <SettingsRow title="Interval (hours)" hint="Как часто повторять автодиагностику.">
            <input
              className={settingsStyles.input}
              type="number"
              min={1}
              max={168}
              value={settings.diagnosticsIntervalHours}
              disabled={disabled}
              onChange={(e) =>
                void updateSettings({ diagnosticsIntervalHours: Number(e.target.value) || 24 })
              }
            />
          </SettingsRow>
        </SettingsGroup>

        <SettingsGroup title="Network" description="Параметры проверки доступности сервисов.">
          <SettingsRow title="Timeout (ms)">
            <input
              className={settingsStyles.input}
              type="number"
              min={1000}
              max={60000}
              step={500}
              value={settings.networkTimeoutMs}
              disabled={disabled}
              onChange={(e) =>
                void updateSettings({ networkTimeoutMs: Number(e.target.value) || 8000 })
              }
            />
          </SettingsRow>
          <SettingsRow title="Retries">
            <input
              className={settingsStyles.input}
              type="number"
              min={0}
              max={10}
              value={settings.networkRetries}
              disabled={disabled}
              onChange={(e) =>
                void updateSettings({ networkRetries: Number(e.target.value) || 0 })
              }
            />
          </SettingsRow>
        </SettingsGroup>

        <SettingsGroup title="Advanced" description="Пути и журналирование для интеграции с zapret.">
          <SettingsRow title="Verbose logging" hint="Подробные логи для отладки движка.">
            <Toggle
              checked={settings.advancedLogging}
              disabled={disabled}
              onChange={(checked) => void updateSettings({ advancedLogging: checked })}
            />
          </SettingsRow>
          <SettingsRow title="Engine path" hint="Пусто — использовать управляемый движок клиента.">
            <input
              className={settingsStyles.select}
              style={{ minWidth: 220 }}
              placeholder="Managed by app"
              value={settings.enginePath ?? ''}
              disabled={disabled}
              onChange={(e) =>
                void updateSettings({ enginePath: e.target.value || null })
              }
            />
          </SettingsRow>
          <SettingsRow title="Strategies path">
            <input
              className={settingsStyles.select}
              style={{ minWidth: 220 }}
              placeholder="Managed by app"
              value={settings.strategiesPath ?? ''}
              disabled={disabled}
              onChange={(e) =>
                void updateSettings({ strategiesPath: e.target.value || null })
              }
            />
          </SettingsRow>
        </SettingsGroup>

        <SettingsGroup id="about" title="About" description="О приложении.">
          <SettingsRow title="Application" hint={`${about.name} · ${about.license}`}>
            <span style={{ fontSize: 13, fontWeight: 500 }}>v{about.version}</span>
          </SettingsRow>
          <SettingsRow title="Authors" hint={about.authors.join(', ')}>
            <span />
          </SettingsRow>
          <div className={settingsStyles.row}>
            <div className={settingsStyles.rowCopy}>
              <div className={settingsStyles.rowTitle}>Links</div>
              <div className={settingsStyles.linkList} style={{ marginTop: 8 }}>
                <a href={about.githubUrl} target="_blank" rel="noreferrer">
                  GitHub repository
                </a>
                <a href={about.homepageUrl} target="_blank" rel="noreferrer">
                  Homepage
                </a>
              </div>
            </div>
          </div>
        </SettingsGroup>
      </div>
    </div>
  )
}
