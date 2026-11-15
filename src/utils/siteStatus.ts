// ╔════════════════════════════════════════════════════╗
// ║                 utils/siteStatus                 ║
// ╚════════════════════════════════════════════════════╝

import type { AvailabilityStatus, EngineStatus, MonitoredService } from '@/types'

/** Probes older than this are treated as stale on Home / Sites. */
export const SITE_FRESH_MS = 5 * 60 * 1000

export function isSiteCheckFresh(lastCheckedAt: string | null | undefined, now = Date.now()): boolean {
  if (!lastCheckedAt) return false
  const ts = new Date(lastCheckedAt).getTime()
  if (Number.isNaN(ts)) return false
  return now - ts <= SITE_FRESH_MS
}

/** Display status: grey out when bypass is off or the last probe is stale. */
export function displaySiteStatus(
  service: MonitoredService,
  engineStatus: EngineStatus,
  now = Date.now(),
): AvailabilityStatus {
  if (service.status === 'checking') return 'checking'
  const bypassOn = engineStatus === 'running' || engineStatus === 'starting'
  if (!bypassOn) return 'unknown'
  if (!isSiteCheckFresh(service.lastCheckedAt, now)) return 'unknown'
  return service.status
}
