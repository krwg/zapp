// ╔════════════════════════════════════════════════════╗
// ║                  utils/version                   ║
// ╚════════════════════════════════════════════════════╝

/** Display form: 0.2.2 → [0.2.2] */
export function formatVersion(version: string | null | undefined): string {
  if (!version) return '[—]'
  const cleaned = version.trim().replace(/^v/i, '')
  return `[${cleaned}]`
}
