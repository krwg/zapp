// ╔════════════════════════════════════════════════════╗
// ║                       App                        ║
// ╚════════════════════════════════════════════════════╝

import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom'
import { AppShell } from '@/components/layout/AppShell/AppShell'
import { AppStateProvider } from '@/context/AppStateContext'
import { ThemeProvider } from '@/context/ThemeContext'
import { ToastProvider } from '@/context/ToastContext'
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
    <ThemeProvider>
      <ToastProvider>
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
                <Route path="*" element={<Navigate to="/" replace />} />
              </Route>
            </Routes>
          </BrowserRouter>
        </AppStateProvider>
      </ToastProvider>
    </ThemeProvider>
  )
}
