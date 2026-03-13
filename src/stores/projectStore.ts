import { reactive } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Project } from '../types/project'
import { clearAllProjectLoadStatus } from './projectDataStore'

// 后端返回的数据结构
interface BackendProject {
  id: string
  name: string
  path: string
  logo?: string
  added_at: number
  is_git_repo?: boolean
  product_name_template?: string
  add_timestamp?: boolean
  last_updated_at?: number
}

// 全局项目状态
const state = reactive({
  projects: [] as Project[],
  isLoading: false
})

// 加载项目列表
export const loadProjects = async () => {
  try {
    const result = await invoke<BackendProject[]>('get_projects')
    state.projects = result.map(p => ({
      id: p.id,
      name: p.name,
      path: p.path,
      logo: p.logo,
      addedAt: p.added_at,
      isGitRepo: p.is_git_repo,
      productNameTemplate: p.product_name_template,
      addTimestamp: p.add_timestamp,
      lastUpdatedAt: p.last_updated_at
    }))
  } catch (err) {
    console.error('Failed to load projects:', err)
  }
}

// 添加项目
export const addProject = async (project: Project) => {
  try {
    await invoke('add_project', {
      id: project.id,
      name: project.name,
      path: project.path,
      logo: project.logo
    })
    state.projects.push(project)
    // 刷新所有项目以获取 git 和 npm 脚本信息
    await refreshAllProjects()
  } catch (err) {
    console.error('Failed to add project:', err)
  }
}

// 删除项目
export const removeProject = async (id: string) => {
  try {
    await invoke('remove_project', { id })
    state.projects = state.projects.filter(p => p.id !== id)
  } catch (err) {
    console.error('Failed to remove project:', err)
  }
}

// 刷新所有项目
export const refreshAllProjects = async () => {
  state.isLoading = true
  try {
    const result = await invoke<BackendProject[]>('refresh_all_projects')
    state.projects = result.map(p => ({
      id: p.id,
      name: p.name,
      path: p.path,
      logo: p.logo,
      addedAt: p.added_at,
      isGitRepo: p.is_git_repo,
      productNameTemplate: p.product_name_template,
      addTimestamp: p.add_timestamp,
      lastUpdatedAt: p.last_updated_at
    }))
    // 刷新后清除加载状态，以便项目卡片可以重新加载最新数据
    clearAllProjectLoadStatus()
  } catch (err) {
    console.error('Failed to refresh all projects:', err)
  } finally {
    state.isLoading = false
  }
}

export const useProjectStore = () => {
  return {
    state,
    loadProjects,
    addProject,
    removeProject,
    refreshAllProjects
  }
}