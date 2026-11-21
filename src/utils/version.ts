// ╔════════════════════════════════════════════════════╗
// ║                  utils/version                   ║
// ╚════════════════════════════════════════════════════╝


export function formatVersion(version: string | null | undefined): string {
  if (!version) return '[—]'
  const cleaned = version.trim().replace(/^v/i, '')
  return `[${cleaned}]`
}

