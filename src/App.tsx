// ╔════════════════════════════════════════════════════╗
// ║                       App                        ║
// ╚════════════════════════════════════════════════════╝

import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom'
import { AppShell } from '@/components/layout/AppShell/AppShell'
import { AppStateProvider } from '@/context/AppStateContext'
import { LocaleProvider } from '@/context/LocaleContext'
import { ThemeProvider } from '@/context/ThemeContext'
import { ToastProvider } from '@/context/ToastContext'
import { UiPrefsProvider } from '@/context/UiPrefsContext'
import { AboutPage } from '@/pages/About/AboutPage'
import { DiagnosticsPage } from '@/pages/Diagnostics/DiagnosticsPage'
import { OverviewPage } from '@/pages/Overview/OverviewPage'
import { ProfilesPage } from '@/pages/Profiles/ProfilesPage'
import { ServicesPage } from '@/pages/Services/ServicesPage'
import { SettingsPage } from '@/pages/Settings/SettingsPage'
import { UpdatesPage } from '@/pages/Updates/UpdatesPage'
import '@/styles/global.css'
import '@/styles/animations.css'

export default function App() {
  return (
    <LocaleProvider>
      <ThemeProvider>
        <ToastProvider>
          <UiPrefsProvider>
            <AppStateProvider>
              <BrowserRouter>
                <Routes>
                  <Route element={<AppShell />}>
                    <Route index element={<OverviewPage />} />
                    <Route path="diagnostics" element={<DiagnosticsPage />} />
                    <Route path="profiles" element={<ProfilesPage />} />
                    <Route path="services" element={<ServicesPage />} />
                    <Route path="updates" element={<UpdatesPage />} />
                    <Route path="settings" element={<SettingsPage />} />
                    <Route path="about" element={<AboutPage />} />
                    <Route path="*" element={<Navigate to="/" replace />} />
                  </Route>
                </Routes>
              </BrowserRouter>
            </AppStateProvider>
          </UiPrefsProvider>
        </ToastProvider>
      </ThemeProvider>
    </LocaleProvider>
  )
}
