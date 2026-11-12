// ╔════════════════════════════════════════════════════╗
// ║                    api/client                    ║
// ╚════════════════════════════════════════════════════╝

import { mockApi } from './mockClient'
import { createTauriApi, isTauriRuntime } from './tauriClient'
import type { ZappApi } from './types'

export function getApi(): ZappApi {
  if (isTauriRuntime()) {
    return createTauriApi()
  }
  return mockApi
}

export type { ZappApi } from './types'
