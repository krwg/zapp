// ╔════════════════════════════════════════════════════╗
// ║                utils/modeLabels                  ║
// ╚════════════════════════════════════════════════════╝

import type { MessageKey } from '@/i18n'
import type { StrategySource } from '@/types'

export function sourceLabelKey(source: StrategySource): MessageKey {
  switch (source) {
    case 'bundled':
      return 'modes.sourceBundled'
    case 'remote':
      return 'modes.sourceRemote'
    case 'auto':
      return 'modes.sourceAuto'
    case 'user':
      return 'modes.sourceUser'
    case 'imported':
      return 'modes.sourceImported'
  }
}
