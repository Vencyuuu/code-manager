export interface Project {
  id: string
  name: string
  path: string
  logo?: string
  addedAt: number
  isGitRepo?: boolean
  productNameTemplate?: string
  addTimestamp?: boolean
  addBranch?: boolean
  addEnv?: boolean
  envName?: string
  addVersion?: boolean
  versionName?: string
  groupId?: string
  lastUpdatedAt?: number
}