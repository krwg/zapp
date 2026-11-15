// ╔════════════════════════════════════════════════════╗
// ║           pages/Settings/SettingsPage            ║
// ╚════════════════════════════════════════════════════╝

import {
  SettingsGroup,
  SettingsRow,
  settingsStyles,
} from '@/components/domain/SettingsGroup/SettingsGroup'
import { PageHeader } from '@/components/layout/PageHeader/PageHeader'
import { Hint } from '@/components/ui/Hint/Hint'
import { Toggle } from '@/components/ui/Toggle/Toggle'
import { useAppState } from '@/context/AppStateContext'
import { useLocale } from '@/context/LocaleContext'
import { useTheme } from '@/context/ThemeContext'
import { useToast } from '@/context/ToastContext'
import { useUiPrefs } from '@/context/UiPrefsContext'
import { translate, type Locale } from '@/i18n'
import type { ThemeMode } from '@/types'
import page from '../shared/page.module.css'

export function SettingsPage() {
  const { snapshot, updateSettings, busy } = useAppState()
  const { mode, setMode } = useTheme()
  const { t, locale, setLocale } = useLocale()
  const { push } = useToast()
  const { confirmDisableProtection, showSidebarSession, setPref } = useUiPrefs()

  if (!snapshot) return null
  const { settings } = snapshot
  const disabled = !!busy.settings

  const themeOptions: { value: ThemeMode; label: string }[] = [
    { value: 'light', label: t('settings.light') },
    { value: 'dark', label: t('settings.dark') },
    { value: 'system', label: t('settings.system') },
  ]

  const localeOptions: { value: Locale; label: string }[] = [
    { value: 'ru', label: 'Русский' },
    { value: 'en', label: 'English' },
  ]

  const changeLocale = (next: Locale) => {
    if (next === locale) return
    setLocale(next)
    const langName = next === 'ru' ? 'Русский' : 'English'
    push({
      title: translate(next, 'toast.languageChangedTitle'),
      description: translate(next, 'toast.languageChangedDesc', { lang: langName }),
      tone: 'success',
    })
  }

  return (
    <div>
      <PageHeader
        title={t('settings.title')}
        description={t('settings.description')}
        titleAddon={
          <Hint
            label={t('hints.settingsLabel')}
            title={t('hints.settingsTitle')}
            body={t('hints.settingsBody')}
            wide
          />
        }
      />

      <div className={page.stack}>
        <SettingsGroup>
          <SettingsRow title={t('settings.language')} hint={t('settings.languageHint')}>
            <div className={settingsStyles.segment} role="group" aria-label={t('settings.language')}>
              {localeOptions.map((opt) => (
                <button
                  key={opt.value}
                  type="button"
                  className={[
                    settingsStyles.segmentBtn,
                    locale === opt.value ? settingsStyles.segmentBtnActive : '',
                  ].join(' ')}
                  onClick={() => changeLocale(opt.value)}
                >
                  {opt.label}
                </button>
              ))}
            </div>
          </SettingsRow>
        </SettingsGroup>

        <SettingsGroup>
          <SettingsRow title={t('settings.theme')} hint={t('settings.themeHint')}>
            <div className={settingsStyles.segment} role="group" aria-label={t('settings.theme')}>
              {themeOptions.map((opt) => (
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

        <SettingsGroup>
          <SettingsRow
            title={t('settings.launchAtStartup')}
            hint={t('settings.launchAtStartupHint')}
          >
            <Toggle
              checked={settings.startWithSystem}
              disabled={disabled}
              onChange={(checked) => void updateSettings({ startWithSystem: checked })}
            />
          </SettingsRow>
          <SettingsRow
            title={t('settings.minimizeToTray')}
            hint={t('settings.minimizeToTrayHint')}
          >
            <Toggle
              checked={settings.minimizeToTray}
              disabled={disabled}
              onChange={(checked) => void updateSettings({ minimizeToTray: checked })}
            />
          </SettingsRow>
        </SettingsGroup>

        <SettingsGroup>
          <SettingsRow
            title={t('settings.confirmDisable')}
            hint={t('settings.confirmDisableHint')}
          >
            <Toggle
              checked={confirmDisableProtection}
              onChange={(checked) => setPref('confirmDisableProtection', checked)}
            />
          </SettingsRow>
          <SettingsRow
            title={t('settings.showSidebarSession')}
            hint={t('settings.showSidebarSessionHint')}
          >
            <Toggle
              checked={showSidebarSession}
              onChange={(checked) => setPref('showSidebarSession', checked)}
            />
          </SettingsRow>
        </SettingsGroup>

        <SettingsGroup>
          <SettingsRow title={t('settings.autoCheck')} hint={t('settings.updatesDesc')}>
            <Toggle
              checked={settings.autoCheckUpdates}
              disabled={disabled}
              onChange={(checked) => void updateSettings({ autoCheckUpdates: checked })}
            />
          </SettingsRow>
          <SettingsRow
            title={t('settings.autoInstall')}
            hint={t('settings.autoInstallHint')}
          >
            <Toggle
              checked={settings.autoInstallUpdates}
              disabled={disabled}
              onChange={(checked) => void updateSettings({ autoInstallUpdates: checked })}
            />
          </SettingsRow>
        </SettingsGroup>

        <SettingsGroup>
          <SettingsRow
            title={t('settings.autoDiagnostics')}
            hint={t('settings.diagnosticsDesc')}
          >
            <Toggle
              checked={settings.autoDiagnostics}
              disabled={disabled}
              onChange={(checked) => void updateSettings({ autoDiagnostics: checked })}
            />
          </SettingsRow>
          <SettingsRow
            title={t('settings.intervalHours')}
            hint={t('settings.intervalHint')}
          >
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

        <SettingsGroup>
          <SettingsRow title={t('settings.timeoutMs')} hint={t('settings.networkDesc')}>
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
          <SettingsRow title={t('settings.retries')}>
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

        <SettingsGroup>
          <SettingsRow title={t('settings.verboseLogging')} hint={t('settings.verboseHint')}>
            <Toggle
              checked={settings.advancedLogging}
              disabled={disabled}
              onChange={(checked) => void updateSettings({ advancedLogging: checked })}
            />
          </SettingsRow>
          <SettingsRow title={t('settings.enginePath')} hint={t('settings.enginePathHint')}>
            <input
              className={settingsStyles.select}
              style={{ minWidth: 220 }}
              placeholder={t('settings.managedPlaceholder')}
              value={settings.enginePath ?? ''}
              disabled={disabled}
              onChange={(e) => void updateSettings({ enginePath: e.target.value || null })}
            />
          </SettingsRow>
          <SettingsRow title={t('settings.strategiesPath')}>
            <input
              className={settingsStyles.select}
              style={{ minWidth: 220 }}
              placeholder={t('settings.managedPlaceholder')}
              value={settings.strategiesPath ?? ''}
              disabled={disabled}
              onChange={(e) => void updateSettings({ strategiesPath: e.target.value || null })}
            />
          </SettingsRow>
        </SettingsGroup>
      </div>
    </div>
  )
}
