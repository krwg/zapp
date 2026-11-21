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

  const [timeoutMs, setTimeoutMs] = useState(8000)
  const debounceRef = useRef<number | null>(null)
  const pendingPatchRef = useRef<Parameters<typeof updateSettings>[0]>({})

  useEffect(() => {
    if (!snapshot) return
    setTimeoutMs(snapshot.settings.networkTimeoutMs)
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
    void updateSettings({ uiLocale: next }, { quiet: true })
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
            title={t('settings.restoreOnLogin')}
            hint={t('settings.restoreOnLoginHint')}
          >
            <Toggle
              checked={settings.restoreBypassOnLogin ?? true}
              disabled={disabled}
              onChange={(checked) => void updateSettings({ restoreBypassOnLogin: checked })}
            />
          </SettingsRow>
          <SettingsRow
            title={t('settings.restoreOnResume')}
            hint={t('settings.restoreOnResumeHint')}
          >
            <Toggle
              checked={settings.restoreBypassOnResume ?? true}
              disabled={disabled}
              onChange={(checked) => void updateSettings({ restoreBypassOnResume: checked })}
            />
          </SettingsRow>
          <SettingsRow
            title={t('settings.restoreOnNetwork')}
            hint={t('settings.restoreOnNetworkHint')}
          >
            <Toggle
              checked={settings.restoreBypassOnNetworkChange ?? true}
              disabled={disabled}
              onChange={(checked) =>
                void updateSettings({ restoreBypassOnNetworkChange: checked })
              }
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
            title={t('settings.watchdogEnabled')}
            hint={t('settings.watchdogEnabledHint')}
          >
            <Toggle
              checked={settings.watchdogEnabled}
              disabled={disabled}
              onChange={(checked) => void updateSettings({ watchdogEnabled: checked })}
            />
          </SettingsRow>
          <SettingsRow title={t('settings.watchdogInterval')} hint={t('settings.watchdogDesc')}>
            <input
              className={settingsStyles.input}
              type="number"
              min={1}
              max={120}
              step={1}
              value={settings.watchdogIntervalMinutes}
              disabled={disabled || !settings.watchdogEnabled}
              onChange={(e) => {
                const v = Math.max(1, Number(e.target.value) || 5)
                debouncePatch({ watchdogIntervalMinutes: v })
              }}
            />
          </SettingsRow>
          <SettingsRow
            title={t('settings.watchdogAutoRecheck')}
            hint={t('settings.watchdogAutoRecheckHint')}
          >
            <Toggle
              checked={settings.watchdogAutoRecheck}
              disabled={disabled || !settings.watchdogEnabled}
              onChange={(checked) => void updateSettings({ watchdogAutoRecheck: checked })}
            />
          </SettingsRow>
          <SettingsRow
            title={t('settings.reapplyWorkingModes')}
            hint={t('settings.reapplyWorkingModesHint')}
          >
            <Toggle
              checked={settings.reapplyWorkingModeOnNetworkChange}
              disabled={disabled}
              onChange={(checked) =>
                void updateSettings({ reapplyWorkingModeOnNetworkChange: checked })
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
              value={timeoutMs}
              onChange={(e) => {
                const v = Number(e.target.value) || 8000
                setTimeoutMs(v)
                debouncePatch({ networkTimeoutMs: v })
              }}
            />
          </SettingsRow>
        </SettingsGroup>
      </div>
    </div>
  )
}
