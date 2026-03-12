<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { Input } from 'ant-design-vue'
import { Icon } from '@iconify/vue'
import { useProjectStore } from '../stores/projectStore'
import ProjectCard from '../components/ProjectCard.vue'
import type { Project } from '../types/project'

const { state: projectState, addProject, removeProject, refreshAllProjects } = useProjectStore()

const isDragging = ref(false)
const searchQuery = ref('')
const columnCount = ref(2)

let unlisten: UnlistenFn | null = null

// 过滤后的项目列表
const filteredProjects = computed(() => {
  const projects = projectState.projects
  if (!searchQuery.value.trim()) {
    return projects
  }
  const query = searchQuery.value.toLowerCase()
  return projects.filter(p => p.name.toLowerCase().includes(query))
})

// 切换布局列数
const setColumnCount = (count: number) => {
  columnCount.value = count
}

const handleDrop = async (paths: string[]) => {
  console.log('Drop paths:', paths)
  for (const path of paths) {
    const name = path.split(/[/\\]/).pop() || 'Unknown'
    const project: Project = {
      id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
      name,
      path,
      addedAt: Date.now()
    }
    console.log('Adding project:', project)
    await addProject(project)
  }
}

onMounted(async () => {
  // 监听 Tauri 的拖拽事件
  unlisten = await listen<{ paths: string[] }>('tauri://drag-drop', async (event) => {
    console.log('Drag drop event:', event.payload)
    isDragging.value = false
    await handleDrop(event.payload.paths)
  })

  await listen('tauri://drag-enter', () => {
    isDragging.value = true
  })

  await listen('tauri://drag-leave', () => {
    isDragging.value = false
  })
})

onUnmounted(() => {
  if (unlisten) {
    unlisten()
  }
})

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
      </div>
    </div>

    <!-- 项目列表 -->
    <div
      class="project-list"
      :class="{ 'dragging': isDragging }"
      :style="{ gridTemplateColumns: `repeat(${columnCount}, 1fr)` }"
    >
      <div v-if="filteredProjects.length === 0" class="empty-state">
        <div class="empty-icon">📁</div>
        <p v-if="searchQuery">没有找到匹配的项目</p>
        <p v-else>拖拽项目文件夹到这里添加</p>
      </div>
      <ProjectCard
        v-for="project in filteredProjects"
        :key="project.id"
        :project="project"
        @remove="handleRemoveProject"
      />
    </div>
    <div v-if="isDragging" class="drop-overlay">
      <div class="drop-text">释放以添加项目</div>
    </div>
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
  margin-bottom: 20px;
  padding: 12px 16px;
  background: #fff;
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
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  background: #fff;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  color: #666;
  transition: all 0.2s;
}

.layout-btn:hover {
  border-color: #1890ff;
  color: #1890ff;
}

.layout-btn.active {
  background: #1890ff;
  border-color: #1890ff;
  color: #fff;
}

.refresh-all-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  background: #1890ff;
  color: #fff;
  border: none;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
  transition: background 0.2s;
}

.refresh-all-btn:hover:not(:disabled) {
  background: #40a9ff;
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

.project-list {
  display: grid;
  gap: 20px;
}

.project-list.dragging {
  background: rgba(24, 144, 255, 0.1);
}

.empty-state {
  grid-column: 1 / -1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 400px;
  color: #999;
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
  background: rgba(24, 144, 255, 0.2);
  border: 2px dashed #1890ff;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
  z-index: 100;
}

.drop-text {
  font-size: 24px;
  color: #1890ff;
  font-weight: 500;
}
</style>