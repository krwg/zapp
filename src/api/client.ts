// ╔════════════════════════════════════════════════════╗
// ║                    api/client                    ║
// ╚════════════════════════════════════════════════════╝

import { mockApi } from './mockClient'
import { createTauriApi, isTauriRuntime } from './tauriClient'
import type { ZappApi } from './types'

let tauriApi: ZappApi | null = null

export function getApi(): ZappApi {
  if (isTauriRuntime()) {
    if (!tauriApi) {
      tauriApi = createTauriApi()
    }
    return tauriApi
  }
  return mockApi
}

export type { ZappApi } from './types'
