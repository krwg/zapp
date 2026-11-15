// ╔════════════════════════════════════════════════════╗
// ║               utils/serviceIcons                 ║
// ╚════════════════════════════════════════════════════╝

import youtube from '@/assets/services/youtube.svg'
import discord from '@/assets/services/discord.svg'
import github from '@/assets/services/github.svg'

const MAP: Record<string, string> = {
  youtube,
  'youtube.com': youtube,
  discord,
  'discord.com': discord,
  github,
  'github.com': github,
}

export function serviceIcon(nameOrHost: string): string | null {
  const key = nameOrHost.trim().toLowerCase()
  if (MAP[key]) return MAP[key]
  for (const [k, src] of Object.entries(MAP)) {
    if (key.includes(k)) return src
  }
  return null
}
