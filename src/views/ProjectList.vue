<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { Input, Modal, Switch, message } from 'ant-design-vue'
import { Icon } from '@iconify/vue'
import { useProjectStore } from '../stores/projectStore'
import { useGroupStore } from '../stores/groupStore'
import ProjectCard from '../components/ProjectCard.vue'
import type { Project } from '../types/project'

const { state: projectState, addProject, removeProject, refreshAllProjects } = useProjectStore()
const { state: groupState, loadGroups, createGroup, updateGroup, deleteGroup, selectGroup } = useGroupStore()

const isDragging = ref(false)
const searchQuery = ref('')
const columnCount = ref(2)

// 批量导入弹窗
const isBatchImportVisible = ref(false)
const batchImportFolder = ref('')
const autoCreateGroup = ref(false)
const bindToCreatedGroup = ref(true)
const groupNameFromFolder = ref('')
const isScanning = ref(false)
const foundProjects = ref<string[]>([])

// 新建分组弹窗
const isGroupModalVisible = ref(false)
const isEditGroup = ref(false)
const editingGroupId = ref('')
const groupName = ref('')
const groupColor = ref('#1890ff')
const groupIcon = ref('')
const selectedProjectIds = ref<string[]>([])


let unlisten: UnlistenFn | null = null
let unlistenDragEnter: UnlistenFn | null = null
let unlistenDragLeave: UnlistenFn | null = null

// 过滤后的项目列表
const filteredProjects = computed(() => {
  let projects = projectState.projects

  // 按分组筛选
  if (groupState.selectedGroupId) {
    projects = projects.filter(p => p.groupId === groupState.selectedGroupId)
  }

  // 按搜索词筛选
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    projects = projects.filter(p => p.name.toLowerCase().includes(query))
  }

  return projects
})

// 切换布局列数
const setColumnCount = (count: number) => {
  columnCount.value = count
}

const handleDrop = async (paths: string[]) => {
  console.log('Drop paths:', paths)
  // 检查当前是否选中了分组
  const currentGroupId = groupState.selectedGroupId
  for (const path of paths) {
    const name = path.split(/[/\\]/).pop() || 'Unknown'
    const project: Project = {
      id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
      name,
      path,
      addedAt: Date.now(),
      groupId: currentGroupId || undefined
    }
    console.log('Adding project:', project)
    await addProject(project)
  }
  // 如果有选中分组，刷新分组列表
  if (currentGroupId) {
    await useGroupStore().loadGroups()
  }
}

// 切换项目选择
const toggleProjectSelect = (projectId: string) => {
  const idx = selectedProjectIds.value.indexOf(projectId)
  if (idx > -1) {
    selectedProjectIds.value.splice(idx, 1)
  } else {
    selectedProjectIds.value.push(projectId)
  }
}

// 打开新建分组弹窗
const openCreateGroupModal = () => {
  isEditGroup.value = false
  editingGroupId.value = ''
  groupName.value = ''
  groupColor.value = '#1890ff'
  groupIcon.value = ''
  selectedProjectIds.value = []
  isGroupModalVisible.value = true
}

// 打开编辑分组弹窗
const openEditGroupModal = (group: { id: string; name: string; color: string; icon?: string }) => {
  isEditGroup.value = true
  editingGroupId.value = group.id
  groupName.value = group.name
  groupColor.value = group.color
  groupIcon.value = group.icon || ''
  // 选中属于该分组的项目
  selectedProjectIds.value = projectState.projects
    .filter(p => p.groupId === group.id)
    .map(p => p.id)
  isGroupModalVisible.value = true
}

// 保存分组
const saveGroup = async () => {
  if (!groupName.value.trim()) {
    message.warning('请输入分组名称')
    return
  }

  try {
    const groupData = {
      id: editingGroupId.value || Date.now().toString() + Math.random().toString(36).substr(2, 9),
      name: groupName.value.trim(),
      color: groupColor.value,
      icon: groupIcon.value.trim() || undefined
    }

    if (isEditGroup.value) {
      await updateGroup(groupData)
      message.success('分组已更新')
    } else {
      await createGroup(groupData)
      message.success('分组已创建')
    }

    // 绑定项目到分组
    for (const projectId of selectedProjectIds.value) {
      await useGroupStore().bindProjectToGroup(projectId, groupData.id)
    }

    // 刷新项目列表
    await useProjectStore().loadProjects()
    await useGroupStore().loadGroups()

    isGroupModalVisible.value = false
  } catch (err) {
    message.error('保存失败: ' + err)
  }
}

// 删除分组
const handleDeleteGroup = async (groupId: string) => {
  try {
    await deleteGroup(groupId)
    message.success('分组已删除')
    // 刷新项目列表
    await useProjectStore().loadProjects()
    // 关闭弹窗
    isGroupModalVisible.value = false
  } catch (err) {
    message.error('删除失败: ' + err)
  }
}

// 选择分组筛选
const handleSelectGroup = (groupId: string | null) => {
  selectGroup(groupId)
}

onMounted(async () => {
  // 加载分组列表
  await loadGroups()

  // 监听 Tauri 的拖拽事件
  unlisten = await listen<{ paths: string[] }>('tauri://drag-drop', async (event) => {
    console.log('Drag drop event:', event.payload)
    isDragging.value = false
    await handleDrop(event.payload.paths)
  })

  unlistenDragEnter = await listen('tauri://drag-enter', () => {
    isDragging.value = true
  })

  unlistenDragLeave = await listen('tauri://drag-leave', () => {
    isDragging.value = false
  })
})

onUnmounted(() => {
  if (unlisten) {
    unlisten()
  }
  if (unlistenDragEnter) {
    unlistenDragEnter()
  }
  if (unlistenDragLeave) {
    unlistenDragLeave()
  }
})

// 批量导入
const openBatchImportModal = async () => {
  const selected = await open({
    directory: true,
    multiple: false,
    title: '选择要导入的文件夹'
  })
  if (selected) {
    batchImportFolder.value = selected as string
    // 从文件夹名提取默认分组名
    const folderName = batchImportFolder.value.split(/[/\\]/).pop() || ''
    groupNameFromFolder.value = folderName
    // 扫描文件夹下的子项目
    isScanning.value = true
    foundProjects.value = []
    try {
      const projects = await invoke<string[]>('scan_subfolders', { path: batchImportFolder.value })
      foundProjects.value = projects
    } catch (err) {
      console.error('扫描失败:', err)
      message.error('扫描文件夹失败: ' + err)
    } finally {
      isScanning.value = false
    }
    isBatchImportVisible.value = true
  }
}

// 执行批量导入
const executeBatchImport = async () => {
  if (foundProjects.value.length === 0) {
    message.warning('没有找到可导入的项目')
    return
  }

  let groupId = ''

  // 如果启用自动创建分组
  if (autoCreateGroup.value && groupNameFromFolder.value.trim()) {
    try {
      const newGroupId = Date.now().toString() + Math.random().toString(36).substr(2, 9)
      await useGroupStore().createGroup({
        id: newGroupId,
        name: groupNameFromFolder.value.trim(),
        color: '#1890ff'
      })
      // 只有当"绑定到该分组"开关开启时才设置groupId
      if (bindToCreatedGroup.value) {
        groupId = newGroupId
      }
      message.success('分组已创建')
    } catch (err) {
      console.error('创建分组失败:', err)
    }
  }

  let importedCount = 0
  for (const projectPath of foundProjects.value) {
    const name = projectPath.split(/[/\\]/).pop() || 'Unknown'
    const project: Project = {
      id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
      name,
      path: projectPath,
      addedAt: Date.now(),
      groupId: groupId || undefined
    }
    await addProject(project)
    importedCount++
  }

  message.success(`成功导入 ${importedCount} 个项目`)
  isBatchImportVisible.value = false
  // 重置状态
  batchImportFolder.value = ''
  autoCreateGroup.value = false
  bindToCreatedGroup.value = true
  groupNameFromFolder.value = ''
  foundProjects.value = []
}

const handleRemoveProject = async (id: string) => {
  await removeProject(id)
}
</script>

<template>
  <div class="project-list-container">
    <!-- 工具栏 -->
    <div class="toolbar">
      <div class="toolbar-left">
        <Input
          v-model:value="searchQuery"
          placeholder="搜索项目名称..."
          class="search-input"
          allow-clear
        />
      </div>
      <div class="toolbar-right">
        <div class="layout-buttons">
          <button
            class="layout-btn"
            :class="{ active: columnCount === 2 }"
            @click="setColumnCount(2)"
            title="两列"
          >
            <Icon icon="mdi:view-grid" />
          </button>
          <button
            class="layout-btn"
            :class="{ active: columnCount === 4 }"
            @click="setColumnCount(4)"
            title="四列"
          >
            <Icon icon="mdi:view-grid-outline" />
          </button>
        </div>
        <button
          class="refresh-all-btn"
          @click="refreshAllProjects"
          :disabled="projectState.isLoading"
          title="刷新所有项目"
        >
          <Icon icon="mdi:refresh" :class="{ 'spin': projectState.isLoading }" />
          {{ projectState.isLoading ? '刷新中...' : '刷新全部' }}
        </button>
        <button
          class="refresh-all-btn batch-import-btn"
          @click="openBatchImportModal"
          title="批量导入项目"
        >
          <Icon icon="mdi:folder-plus" />
          批量导入
        </button>
      </div>
    </div>

    <!-- 分组筛选栏 -->
    <div class="group-filter-bar">
      <div class="group-filter-left">
        <button
          class="group-filter-btn"
          :class="{ active: groupState.selectedGroupId === null }"
          @click="handleSelectGroup(null)"
        >
          <span class="group-name">全部</span>
        </button>
        <div
          v-for="group in groupState.groups"
          :key="group.id"
          class="group-filter-item"
          :style="{ '--group-color': group.color }"
        >
          <button
            class="group-filter-btn"
            :class="{ active: groupState.selectedGroupId === group.id }"
            @click="handleSelectGroup(group.id)"
          >
            <span v-if="group.icon" class="group-icon" :style="{ color: group.color }">
              <Icon :icon="group.icon" />
            </span>
            <span v-else class="group-dot" :style="{ background: group.color }"></span>
            <span class="group-name">{{ group.name }}</span>
            <button class="group-settings-btn" @click.stop="openEditGroupModal(group)">
              <Icon icon="mdi:cog" />
            </button>
          </button>
        </div>
      </div>
      <div class="group-filter-right">
        <button class="new-group-btn" @click="openCreateGroupModal">
          <Icon icon="mdi:plus" />
          新建分组
        </button>
      </div>
    </div>

    <!-- 项目列表 -->
    <div
      class="project-list"
      :class="{ 'dragging': isDragging }"
      :style="{ gridTemplateColumns: `repeat(${columnCount}, 1fr)` }"
    >
      <!-- 全局 Loading 遮罩 -->
      <div v-if="projectState.isLoading" class="global-loading-overlay">
        <Icon icon="mdi:loading" class="global-loading-spinner" />
        <span>正在刷新所有项目...</span>
      </div>
      <div v-if="filteredProjects.length === 0 && !projectState.isLoading" class="empty-state">
        <div class="empty-icon">📁</div>
        <p v-if="searchQuery">没有找到匹配的项目</p>
        <p v-else-if="groupState.selectedGroupId">该分组下没有项目</p>
        <p v-else>拖拽项目文件夹到这里添加</p>
      </div>
      <ProjectCard
        v-for="project in filteredProjects"
        :key="project.id"
        :project="project"
        :group-color="groupState.groups.find(g => g.id === project.groupId)?.color"
        @remove="handleRemoveProject"
      />
    </div>
    <div v-if="isDragging" class="drop-overlay">
      <div class="drop-text">释放以添加项目</div>
    </div>

    <!-- 批量导入弹窗 -->
    <Modal
      v-model:open="isBatchImportVisible"
      title="批量导入项目"
      :footer="null"
      :width="500"
    >
      <div class="batch-import-form">
        <div class="form-item">
          <label>选择文件夹:</label>
          <div class="folder-path">{{ batchImportFolder || '未选择' }}</div>
        </div>

        <div class="form-item">
          <label>找到的项目 ({{ foundProjects.length }}):</label>
          <div v-if="isScanning" class="scanning-tip">
            <Icon icon="mdi:loading" class="spin" />
            正在扫描...
          </div>
          <div v-else class="found-projects-list">
            <div v-for="project in foundProjects" :key="project" class="found-project-item">
              <Icon icon="mdi:folder" />
              {{ project.split(/[/\\]/).pop() }}
            </div>
            <div v-if="foundProjects.length === 0" class="no-projects">
              未找到可导入的项目
            </div>
          </div>
        </div>

        <div class="form-item">
          <label>自动创建分组:</label>
          <Switch v-model:checked="autoCreateGroup" />
        </div>

        <div v-if="autoCreateGroup" class="form-item">
          <label>分组名称:</label>
          <Input v-model:value="groupNameFromFolder" placeholder="请输入分组名称" />
        </div>

        <div v-if="autoCreateGroup" class="form-item">
          <label>绑定到该分组:</label>
          <Switch v-model:checked="bindToCreatedGroup" />
        </div>

        <div class="form-actions">
          <button class="cancel-btn" @click="isBatchImportVisible = false">取消</button>
          <button class="save-btn" @click="executeBatchImport" :disabled="foundProjects.length === 0">
            导入 {{ foundProjects.length }} 个项目
          </button>
        </div>
      </div>
    </Modal>

    <!-- 新建/编辑分组弹窗 -->
    <Modal
      v-model:open="isGroupModalVisible"
      :title="isEditGroup ? '编辑分组' : '新建分组'"
      :footer="null"
      :width="500"
      @ok="saveGroup"
    >
      <div class="group-form">
        <div class="form-item">
          <label>分组名称</label>
          <Input v-model:value="groupName" placeholder="请输入分组名称" />
        </div>
        <div class="form-item">
          <label>主题颜色</label>
          <div class="color-picker">
            <input type="color" v-model="groupColor" />
            <Input v-model:value="groupColor" placeholder="#1890ff" class="color-input" />
          </div>
        </div>
        <div class="form-item">
          <label>分组图标 (Iconify 图标名)</label>
          <Input v-model:value="groupIcon" placeholder="如: mdi:folder" allow-clear />
        </div>
        <div class="form-item">
          <label>选择项目</label>
          <div class="project-select-list">
            <div
              v-for="project in projectState.projects"
              :key="project.id"
              class="project-select-item"
              :class="{ selected: selectedProjectIds.includes(project.id) }"
              @click="toggleProjectSelect(project.id)"
            >
              <Icon v-if="selectedProjectIds.includes(project.id)" icon="mdi:checkbox-marked" class="check-icon" />
              <Icon v-else icon="mdi:checkbox-blank-outline" class="check-icon" />
              <span>{{ project.name }}</span>
            </div>
          </div>
        </div>
        <div class="form-actions">
          <button class="cancel-btn" @click="isGroupModalVisible = false">取消</button>
          <button class="save-btn" @click="saveGroup">保存</button>
          <button v-if="isEditGroup" class="delete-btn" @click="handleDeleteGroup(editingGroupId)">删除</button>
        </div>
      </div>
    </Modal>
  </div>
</template>

<style scoped>
.project-list-container {
  min-height: 100%;
  position: relative;
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding: 12px 16px;
  background: var(--bg-card);
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
}

.toolbar-left {
  flex: 1;
  max-width: 300px;
}

.search-input {
  width: 100%;
}

/* 暗色模式搜索输入框 */
[data-theme="dark"] .search-input .ant-input {
  background: var(--bg-card) !important;
  color: var(--text-primary) !important;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.layout-buttons {
  display: flex;
  gap: 4px;
}

.layout-btn {
  width: 36px;
  height: 36px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: var(--bg-card);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  color: var(--text-secondary);
  transition: all 0.2s;
}

.layout-btn:hover {
  border-color: var(--primary-color);
  color: var(--primary-color);
}

.layout-btn.active {
  background: var(--primary-color);
  border-color: var(--primary-color);
  color: #fff;
}

.refresh-all-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  background: var(--primary-color);
  color: #fff;
  border: none;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
  transition: background 0.2s;
}

.refresh-all-btn:hover:not(:disabled) {
  background: var(--primary-hover);
}

.refresh-all-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* 分组筛选栏 */
.group-filter-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding: 8px 16px;
  background: var(--bg-card);
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
}

.group-filter-left {
  display: flex;
  gap: 12px;
  align-items: center;
  flex-wrap: wrap;
}

.group-filter-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  padding-right: 6px;
  border: 1px solid var(--border-color);
  border-radius: 16px;
  background: var(--bg-card);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.group-filter-btn:hover {
  border-color: var(--primary-color);
  color: var(--primary-color);
}

.group-filter-btn.active {
  background: var(--group-color, var(--primary-color));
  border-color: var(--group-color, var(--primary-color));
  color: #fff;
}

/* 暗色模式下分组按钮选中样式 - 降低亮度 */
[data-theme="dark"] .group-filter-btn.active {
  background: var(--group-color, var(--primary-color));
  filter: brightness(0.7);
  border-color: var(--group-color, var(--primary-color));
  color: #fff;
}

[data-theme="dark"] .group-filter-btn.active .group-settings-btn {
  color: #fff;
}

[data-theme="dark"] .group-filter-btn.active .group-settings-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

[data-theme="dark"] .group-settings-btn {
  color: var(--text-secondary);
}

[data-theme="dark"] .group-settings-btn:hover {
  background: rgba(255, 255, 255, 0.1);
}

.group-name {
  flex: 1;
  white-space: nowrap;
}

.group-settings-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: #999;
  cursor: pointer;
  transition: all 0.2s;
  margin-left: 4px;
}

.group-settings-btn:hover {
  background: rgba(0, 0, 0, 0.1);
}

.group-filter-btn.active .group-settings-btn {
  color: #fff;
}

.group-filter-btn.active .group-settings-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.group-filter-item {
  display: inline-flex;
}

.group-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.group-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  flex-shrink: 0;
}

.group-filter-right {
  display: flex;
  gap: 8px;
}

.new-group-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  border: 1px dashed var(--border-color);
  border-radius: 4px;
  background: var(--bg-card);
  font-size: 13px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.new-group-btn:hover {
  border-color: var(--primary-color);
  color: var(--primary-color);
}

/* 项目列表 */
.project-list {
  display: grid;
  gap: 20px;
  position: relative;
  min-height: 200px;
}

.global-loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--bg-card);
  opacity: 0.95;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  z-index: 20;
  border-radius: 8px;
}

.global-loading-spinner {
  font-size: 40px;
  color: var(--primary-color);
  animation: spin 1s linear infinite;
}

.project-list.dragging {
  background: var(--primary-color);
  opacity: 0.1;
}

.empty-state {
  grid-column: 1 / -1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 400px;
  color: var(--text-secondary);
}

.empty-icon {
  font-size: 64px;
  margin-bottom: 16px;
}

.empty-state p {
  font-size: 16px;
}

.drop-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--primary-color);
  opacity: 0.2;
  border: 2px dashed var(--primary-color);
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
  z-index: 100;
}

.drop-text {
  font-size: 24px;
  color: var(--primary-color);
  font-weight: 500;
}

/* 批量导入按钮 */
.batch-import-btn {
  background: var(--primary-color) !important;
}

.batch-import-btn:hover {
  background: var(--primary-hover) !important;
}

/* 批量导入表单 */
.batch-import-form {
  padding: 8px 0;
}

.folder-path {
  padding: 8px 12px;
  background: var(--bg-color);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-size: 13px;
  color: var(--text-primary);
  word-break: break-all;
}

.scanning-tip {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  color: var(--text-secondary);
}

.found-projects-list {
  max-height: 200px;
  overflow-y: auto;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: var(--bg-card);
}

.found-project-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  font-size: 13px;
  color: var(--text-primary);
  border-bottom: 1px solid var(--border-color);
}

.found-project-item:last-child {
  border-bottom: none;
}

.no-projects {
  padding: 20px;
  text-align: center;
  color: var(--text-secondary);
  font-size: 13px;
}

/* 分组表单 */
.group-form {
  padding: 8px 0;
}

.form-item {
  margin-bottom: 16px;
}

.form-item label {
  display: block;
  margin-bottom: 8px;
  font-size: 14px;
  color: var(--text-primary);
  font-weight: 500;
}

.color-picker {
  display: flex;
  gap: 8px;
  align-items: center;
}

.color-picker input[type="color"] {
  width: 40px;
  height: 32px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  cursor: pointer;
}

.color-input {
  flex: 1;
}

.project-select-list {
  max-height: 200px;
  overflow-y: auto;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: var(--bg-card);
}

.project-select-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  cursor: pointer;
  transition: background 0.2s;
  color: var(--text-primary);
}

.project-select-item:hover {
  background: var(--bg-color);
}

.project-select-item.selected {
  background: rgba(24, 144, 255, 0.1);
}

.check-icon {
  color: #1890ff;
}

.form-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  margin-top: 24px;
}

.cancel-btn {
  padding: 8px 16px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: var(--bg-card);
  color: var(--text-primary);
  font-size: 14px;
  cursor: pointer;
}

.save-btn {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  background: var(--primary-color);
  color: #fff;
  font-size: 14px;
  cursor: pointer;
}

.delete-btn {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  background: #ff4d4f;
  color: #fff;
  font-size: 14px;
  cursor: pointer;
}
</style>
