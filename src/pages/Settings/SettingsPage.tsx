// ╔════════════════════════════════════════════════════╗
// ║           pages/Settings/SettingsPage            ║
// ╚════════════════════════════════════════════════════╝

import { useEffect, useRef, useState } from 'react'
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

const DEBOUNCE_MS = 450

export function SettingsPage() {
  const { snapshot, updateSettings, busy } = useAppState()
  const { mode, setMode } = useTheme()
  const { t, locale, setLocale } = useLocale()
  const { push } = useToast()
  const { confirmDisableProtection, showSidebarSession, setPref } = useUiPrefs()

  const [intervalHours, setIntervalHours] = useState(24)
  const [timeoutMs, setTimeoutMs] = useState(8000)
  const [retries, setRetries] = useState(0)
  const [enginePath, setEnginePath] = useState('')
  const [strategiesPath, setStrategiesPath] = useState('')
  const debounceRef = useRef<number | null>(null)
  const pendingPatchRef = useRef<Parameters<typeof updateSettings>[0]>({})

  useEffect(() => {
    if (!snapshot) return
    setIntervalHours(snapshot.settings.diagnosticsIntervalHours)
    setTimeoutMs(snapshot.settings.networkTimeoutMs)
    setRetries(snapshot.settings.networkRetries)
    setEnginePath(snapshot.settings.enginePath ?? '')
    setStrategiesPath(snapshot.settings.strategiesPath ?? '')
  }, [snapshot])

  useEffect(() => {
    return () => {
      if (debounceRef.current) window.clearTimeout(debounceRef.current)
    }
  }, [])

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

  const debouncePatch = (patch: Parameters<typeof updateSettings>[0]) => {
    pendingPatchRef.current = { ...pendingPatchRef.current, ...patch }
    if (debounceRef.current) window.clearTimeout(debounceRef.current)
    debounceRef.current = window.setTimeout(() => {
      const flush = pendingPatchRef.current
      pendingPatchRef.current = {}
      void updateSettings(flush)
    }, DEBOUNCE_MS)
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
            title={t('settings.telegramAutoStart')}
            hint={t('settings.telegramAutoStartHint')}
          >
            <Toggle
              checked={settings.telegramProxyEnabled}
              disabled={disabled}
              onChange={(checked) =>
                void updateSettings({ telegramProxyEnabled: checked })
              }
            />
          </SettingsRow>
          <SettingsRow
            title={t('settings.telegramOfferLink')}
            hint={t('settings.telegramOfferLinkHint')}
          >
            <Toggle
              checked={settings.telegramOfferLinkOnStart}
              disabled={disabled}
              onChange={(checked) =>
                void updateSettings({ telegramOfferLinkOnStart: checked })
              }
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
              value={intervalHours}
              onChange={(e) => {
                const v = Number(e.target.value) || 24
                setIntervalHours(v)
                debouncePatch({ diagnosticsIntervalHours: v })
              }}
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
              value={timeoutMs}
              onChange={(e) => {
                const v = Number(e.target.value) || 8000
                setTimeoutMs(v)
                debouncePatch({ networkTimeoutMs: v })
              }}
            />
          </SettingsRow>
          <SettingsRow title={t('settings.retries')}>
            <input
              className={settingsStyles.input}
              type="number"
              min={0}
              max={10}
              value={retries}
              onChange={(e) => {
                const v = Number(e.target.value) || 0
                setRetries(v)
                debouncePatch({ networkRetries: v })
              }}
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
              value={enginePath}
              onChange={(e) => {
                const v = e.target.value
                setEnginePath(v)
                debouncePatch({ enginePath: v || null })
              }}
            />
          </SettingsRow>
          <SettingsRow title={t('settings.strategiesPath')}>
            <input
              className={settingsStyles.select}
              style={{ minWidth: 220 }}
              placeholder={t('settings.managedPlaceholder')}
              value={strategiesPath}
              onChange={(e) => {
                const v = e.target.value
                setStrategiesPath(v)
                debouncePatch({ strategiesPath: v || null })
              }}
            />
          </SettingsRow>
        </SettingsGroup>
      </div>
    </div>
  )
}
