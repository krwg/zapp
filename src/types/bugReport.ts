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
