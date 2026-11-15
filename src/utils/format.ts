// ╔════════════════════════════════════════════════════╗
// ║                   utils/format                   ║
// ╚════════════════════════════════════════════════════╝

import { translate, type Locale } from '@/i18n'

export function formatUptime(
  totalSeconds: number,
  opts?: { withSeconds?: boolean; locale?: Locale },
): string {
  const locale = opts?.locale ?? 'ru'
  const withSeconds = opts?.withSeconds ?? false
  const s = Math.max(0, Math.floor(totalSeconds))
  const hours = Math.floor(s / 3600)
  const minutes = Math.floor((s % 3600) / 60)
  const seconds = s % 60

  if (locale === 'ru') {
    if (withSeconds) {
      if (hours > 0) return `${hours}ч ${pad(minutes)}м ${pad(seconds)}с`
      if (minutes > 0) return `${minutes}м ${pad(seconds)}с`
      return `${seconds}с`
    }
    if (hours > 0) return `${hours}ч ${pad(minutes)}м`
    if (minutes > 0) return `${minutes}м ${pad(seconds)}с`
    return `${seconds}с`
  }

  if (withSeconds) {
    if (hours > 0) return `${hours}h ${pad(minutes)}m ${pad(seconds)}s`
    if (minutes > 0) return `${minutes}m ${pad(seconds)}s`
    return `${seconds}s`
  }
  if (hours > 0) return `${hours}h ${pad(minutes)}m`
  if (minutes > 0) return `${minutes}m ${pad(seconds)}s`
  return `${seconds}s`
}

function pad(n: number): string {
  return String(n).padStart(2, '0')
}

export function formatDateTime(
  value: string | null | undefined,
  locale: Locale = 'ru',
): string {
  if (!value) return '—'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return '—'
  return new Intl.DateTimeFormat(locale === 'ru' ? 'ru-RU' : 'en-US', {
    day: '2-digit',
    month: 'short',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date)
}

export function formatLatency(ms: number | null | undefined): string {
  if (ms == null) return '—'
  return `${ms} ms`
}

export function formatRelative(
  value: string | null | undefined,
  locale: Locale = 'ru',
): string {
  if (!value) return translate(locale, 'common.never')
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return '—'
  const diff = Date.now() - date.getTime()
  const minutes = Math.floor(diff / 60000)
  if (minutes < 1) return translate(locale, 'common.justNow')
  if (minutes < 60) return translate(locale, 'common.minutesAgo', { n: minutes })
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return translate(locale, 'common.hoursAgo', { n: hours })
  const days = Math.floor(hours / 24)
  return translate(locale, 'common.daysAgo', { n: days })
}
