import { reactive } from 'vue'

export interface CachedProjectData {
  branches: { name: string; current: boolean }[]
  currentBranch: string
  npmScripts: { name: string; script: string }[]
  loaded: boolean
}

// 全局缓存项目数据
const cachedData = reactive<Record<string, CachedProjectData>>({})

export const cacheProjectData = (projectId: string, data: CachedProjectData) => {
  cachedData[projectId] = data
}

export const getCachedProjectData = (projectId: string): CachedProjectData | undefined => {
  return cachedData[projectId]
}

export const updateProjectBranches = (projectId: string, branches: { name: string; current: boolean }[], currentBranch: string) => {
  if (cachedData[projectId]) {
    cachedData[projectId].branches = branches
    cachedData[projectId].currentBranch = currentBranch
  }
}

export const updateProjectNpmScripts = (projectId: string, npmScripts: { name: string; script: string }[]) => {
  if (cachedData[projectId]) {
    cachedData[projectId].npmScripts = npmScripts
  }
}

export const clearAllCachedData = () => {
  Object.keys(cachedData).forEach(key => {
    delete cachedData[key]
  })
}

export const useCachedProjectData = () => {
  return {
    cacheProjectData,
    getCachedProjectData,
    updateProjectBranches,
    updateProjectNpmScripts,
    clearAllCachedData
  }
}