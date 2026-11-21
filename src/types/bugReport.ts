// ╔════════════════════════════════════════════════════╗
// ║                  types/bugReport                 ║
// ╚════════════════════════════════════════════════════╝

export interface BugReportPrefill {
  summary?: string
  steps?: string
  expected?: string
  actual?: string
  areas?: {
    bypass?: boolean
    smart?: boolean
    updates?: boolean
  }
}

export interface AboutLocationState {
  openBugReport?: boolean
  bugPrefill?: BugReportPrefill
}

/** Auto-collected tech context for GitHub issue body (developer-facing). */
export interface BugReportBundle {
  appVersion: string
  engineVersion: string
  engineRoot: string
  engineInstalled: boolean
  elevated: boolean
  diagnosticsMode: string
  bypassDesired: boolean
  activeStrategy: string | null
  recentErrors: string[]
  recentLog: string[]
}
