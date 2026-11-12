// ╔════════════════════════════════════════════════════╗
// ║               hooks/useSystemTheme               ║
// ╚════════════════════════════════════════════════════╝

import { useEffect, useState } from 'react'

export function useSystemTheme(): 'light' | 'dark' {
  const get = () =>
    window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'

  const [theme, setTheme] = useState<'light' | 'dark'>(get)

  useEffect(() => {
    const media = window.matchMedia('(prefers-color-scheme: dark)')
    const onChange = () => setTheme(media.matches ? 'dark' : 'light')
    media.addEventListener('change', onChange)
    return () => media.removeEventListener('change', onChange)
  }, [])

  return theme
}
