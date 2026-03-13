import { reactive } from 'vue'

// 跟踪项目数据加载状态
const loadedProjects = reactive(new Set<string>())

export const markProjectLoaded = (projectId: string) => {
  loadedProjects.add(projectId)
}

export const isProjectLoaded = (projectId: string): boolean => {
  return loadedProjects.has(projectId)
}

export const clearProjectLoadStatus = (projectId: string) => {
  loadedProjects.delete(projectId)
}

// 清除所有项目的加载状态
export const clearAllProjectLoadStatus = () => {
  loadedProjects.clear()
}

export const useProjectDataStore = () => {
  return {
    markProjectLoaded,
    isProjectLoaded,
    clearProjectLoadStatus,
    clearAllProjectLoadStatus
  }
}