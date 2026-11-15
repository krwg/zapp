// ╔════════════════════════════════════════════════════╗
// ║                    i18n/index                    ║
// ╚════════════════════════════════════════════════════╝

import { en, type MessageTree } from './en'
import { ru } from './ru'

export type Locale = 'ru' | 'en'
export type { MessageTree }

const catalogs: Record<Locale, MessageTree> = { en, ru }

export function getCatalog(locale: Locale): MessageTree {
  return catalogs[locale] ?? en
}

type Leaves<T, P extends string = ''> = T extends string
  ? P
  : {
      [K in keyof T & string]: Leaves<T[K], P extends '' ? K : `${P}.${K}`>
    }[keyof T & string]

export type MessageKey = Leaves<MessageTree>

function readPath(obj: unknown, path: string): string | undefined {
  const parts = path.split('.')
  let cur: unknown = obj
  for (const part of parts) {
    if (cur == null || typeof cur !== 'object') return undefined
    cur = (cur as Record<string, unknown>)[part]
  }
  return typeof cur === 'string' ? cur : undefined
}

export function translate(
  locale: Locale,
  key: MessageKey,
  vars?: Record<string, string | number>,
): string {
  const raw = readPath(catalogs[locale], key) ?? readPath(en, key) ?? key
  if (!vars) return raw
  return Object.entries(vars).reduce(
    (acc, [k, v]) => acc.replaceAll(`{${k}}`, String(v)),
    raw,
  )
}

export { en, ru }
