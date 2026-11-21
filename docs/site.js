const COPY = {
  en: {
    navGithub: 'GitHub',
    navDownload: 'Download',
    heroTitle: 'Quiet Windows bypass for YouTube & Discord',
    heroLede:
      'Soft-minimal client on the Flowseal engine. One Smart check finds a working mode — then it stays out of the way in the tray.',
    ctaDownload: 'Get for Windows',
    ctaSource: 'View source',
    themeLight: 'Light',
    themeDark: 'Dark',
  },
  ru: {
    navGithub: 'GitHub',
    navDownload: 'Скачать',
    heroTitle: 'Тихий Windows-обход для YouTube и Discord',
    heroLede:
      'Soft-minimal клиент на движке Flowseal. Одна умная проверка подбирает режим — дальше zapp сидит в трее и не мешает.',
    ctaDownload: 'Скачать для Windows',
    ctaSource: 'Исходники',
    themeLight: 'Светлая',
    themeDark: 'Тёмная',
  },
}

const LANG_KEY = 'zapp.site.lang'
const THEME_KEY = 'zapp.site.theme'

function detectLang() {
  const stored = localStorage.getItem(LANG_KEY)
  if (stored === 'ru' || stored === 'en') return stored
  return navigator.language.toLowerCase().startsWith('ru') ? 'ru' : 'en'
}

function detectTheme() {
  const stored = localStorage.getItem(THEME_KEY)
  if (stored === 'light' || stored === 'dark') return stored
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

function applyLang(lang) {
  const dict = COPY[lang] || COPY.en
  document.documentElement.lang = lang
  document.querySelectorAll('[data-i18n]').forEach((el) => {
    const key = el.getAttribute('data-i18n')
    if (key && dict[key]) el.textContent = dict[key]
  })
  document.querySelectorAll('#lang-toggle, #lang-toggle-m').forEach((btn) => {
    btn.textContent = lang.toUpperCase()
  })
  localStorage.setItem(LANG_KEY, lang)
}

function applyTheme(theme) {
  document.documentElement.setAttribute('data-theme', theme)
  const label =
    COPY[document.documentElement.lang]?.[theme === 'dark' ? 'themeDark' : 'themeLight'] ||
    theme
  document.querySelectorAll('#theme-toggle, #theme-toggle-m').forEach((btn) => {
    btn.textContent = label
  })
  localStorage.setItem(THEME_KEY, theme)
}

function toggleLang() {
  const next = document.documentElement.lang === 'ru' ? 'en' : 'ru'
  applyLang(next)
  applyTheme(document.documentElement.getAttribute('data-theme') || 'light')
}

function toggleTheme() {
  const cur = document.documentElement.getAttribute('data-theme') || 'light'
  applyTheme(cur === 'dark' ? 'light' : 'dark')
}

function setupMenu() {
  const btn = document.getElementById('menu-toggle')
  const menu = document.getElementById('mobile-menu')
  if (!btn || !menu) return
  btn.addEventListener('click', () => {
    const open = menu.hasAttribute('hidden')
    if (open) {
      menu.removeAttribute('hidden')
      btn.setAttribute('aria-expanded', 'true')
    } else {
      menu.setAttribute('hidden', '')
      btn.setAttribute('aria-expanded', 'false')
    }
  })
}

document.addEventListener('DOMContentLoaded', () => {
  applyLang(detectLang())
  applyTheme(detectTheme())
  setupMenu()
  document.getElementById('lang-toggle')?.addEventListener('click', toggleLang)
  document.getElementById('lang-toggle-m')?.addEventListener('click', toggleLang)
  document.getElementById('theme-toggle')?.addEventListener('click', toggleTheme)
  document.getElementById('theme-toggle-m')?.addEventListener('click', toggleTheme)
})
