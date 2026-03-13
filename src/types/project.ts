export interface Project {
  id: string
  name: string
  path: string
  logo?: string
  addedAt: number
  isGitRepo?: boolean
  productNameTemplate?: string
  addTimestamp?: boolean
  lastUpdatedAt?: number
}