import { reactive } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface Group {
  id: string
  name: string
  color: string
  icon?: string
  createdAt: number
}

// 后端返回的数据结构
interface BackendGroup {
  id: string
  name: string
  color: string
  icon?: string
  created_at: number
}

// 全局分组状态
const state = reactive({
  groups: [] as Group[],
  selectedGroupId: null as string | null,
  isLoading: false
})

// 加载分组列表
export const loadGroups = async () => {
  try {
    const result = await invoke<BackendGroup[]>('get_groups')
    state.groups = result.map(g => ({
      id: g.id,
      name: g.name,
      color: g.color,
      icon: g.icon,
      createdAt: g.created_at
    }))
  } catch (err) {
    console.error('Failed to load groups:', err)
  }
}

// 创建分组
export const createGroup = async (group: Omit<Group, 'createdAt'>) => {
  try {
    await invoke('create_group', {
      id: group.id,
      name: group.name,
      color: group.color,
      icon: group.icon
    })
    await loadGroups()
  } catch (err) {
    console.error('Failed to create group:', err)
    throw err
  }
}

// 更新分组
export const updateGroup = async (group: Omit<Group, 'createdAt'>) => {
  try {
    await invoke('update_group', {
      id: group.id,
      name: group.name,
      color: group.color,
      icon: group.icon
    })
    await loadGroups()
  } catch (err) {
    console.error('Failed to update group:', err)
    throw err
  }
}

// 删除分组
export const deleteGroup = async (id: string) => {
  try {
    await invoke('delete_group', { id })
    // 如果删除的分组被选中，取消选中
    if (state.selectedGroupId === id) {
      state.selectedGroupId = null
    }
    await loadGroups()
  } catch (err) {
    console.error('Failed to delete group:', err)
    throw err
  }
}

// 绑定项目到分组
export const bindProjectToGroup = async (projectId: string, groupId: string | null) => {
  try {
    await invoke('bind_project_to_group', {
      projectId,
      groupId
    })
  } catch (err) {
    console.error('Failed to bind project to group:', err)
    throw err
  }
}

// 选中分组
export const selectGroup = (groupId: string | null) => {
  state.selectedGroupId = groupId
}

export const useGroupStore = () => {
  return {
    state,
    loadGroups,
    createGroup,
    updateGroup,
    deleteGroup,
    bindProjectToGroup,
    selectGroup
  }
}
