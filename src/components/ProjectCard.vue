<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { Modal, Switch, message } from 'ant-design-vue'
import { Icon } from '@iconify/vue'
import type { Project } from '../types/project'
import { useScriptStore } from '../stores/scriptStore'
import { markProjectLoaded, isProjectLoaded } from '../stores/projectDataStore'
import { cacheProjectData, getCachedProjectData } from '../stores/cachedProjectData'

const props = defineProps<{
  project: Project
}>()

const emit = defineEmits<{
  (e: 'remove', id: string): void
}>()

const { addScriptState, updateScriptOutput, markScriptCompleted, getScriptState } = useScriptStore()

// 固定展开，无折叠功能
const isBranchesCollapsed = ref(false)
const branches = ref<{ name: string; current: boolean }[]>([])
const currentBranch = ref('')
const npmScripts = ref<{ name: string; script: string }[]>([])
const customScript = ref('')
const scriptOutput = ref('')
const isRunningScript = ref(false)
const isSettingsVisible = ref(false)
const isLoading = ref(false)
const isDataLoaded = ref(false)

// 输出 Modal
const isOutputModalVisible = ref(false)

// 产物命名配置
const productNameTemplate = ref('')
const addTimestamp = ref(false)

// 计算最后更新时间
const lastUpdatedText = computed(() => {
  if (!props.project.lastUpdatedAt) return ''
  const date = new Date(props.project.lastUpdatedAt * 1000)
  const now = new Date()
  const diff = Math.floor((now.getTime() - date.getTime()) / 1000)

  if (diff < 60) return '刚刚更新'
  if (diff < 3600) return `${Math.floor(diff / 60)}分钟前更新`
  if (diff < 86400) return `${Math.floor(diff / 3600)}小时前更新`
  return date.toLocaleString('zh-CN')
})

// Git 状态
const isGitRepo = computed(() => props.project.isGitRepo === true)

let unlistenOutput: UnlistenFn | null = null

// 加载项目数据
const loadProjectData = async () => {
  // 初始化产物命名配置
  productNameTemplate.value = props.project.productNameTemplate || props.project.name
  addTimestamp.value = props.project.addTimestamp || false

  // 先从缓存恢复数据（如果缓存中有）
  const cached = getCachedProjectData(props.project.id)
  if (cached && cached.loaded) {
    branches.value = cached.branches
    currentBranch.value = cached.currentBranch
    npmScripts.value = cached.npmScripts
  }

  // 如果已经加载过，直接返回
  if (isDataLoaded.value || isProjectLoaded(props.project.id)) {
    return
  }

  isLoading.value = true
  console.log('[ProjectCard] Loading project data for:', props.project.name, props.project.id)
  try {
    // 直接使用 props 中的 isGitRepo，如果不存在则检查
    let isGit = props.project.isGitRepo
    console.log('[ProjectCard] isGitRepo:', isGit, 'for:', props.project.name)
    if (isGit === undefined) {
      isGit = await invoke<boolean>('check_is_git_repo', { path: props.project.path })
    }

    // 并行加载 branches 和 npm scripts
    await Promise.all([
      loadBranches(isGit),
      loadNpmScripts()
    ])

    // 保存到缓存
    cacheProjectData(props.project.id, {
      branches: branches.value,
      currentBranch: currentBranch.value,
      npmScripts: npmScripts.value,
      loaded: true
    })

    isDataLoaded.value = true
    markProjectLoaded(props.project.id) // 标记为已加载
    console.log('[ProjectCard] Loaded successfully for:', props.project.name)
  } catch (err) {
    console.error('[ProjectCard] Failed to load project data:', err)
  } finally {
    isLoading.value = false
  }
}

onMounted(async () => {
  // 加载项目数据
  await loadProjectData()

  // 检查是否有正在运行的脚本状态
  const savedState = getScriptState(props.project.id)
  if (savedState) {
    scriptOutput.value = savedState.output
    isRunningScript.value = savedState.isRunning
  }

  // 监听脚本开始事件
  await listen<[string, string]>('script-started', (event) => {
    if (event.payload[0] === props.project.id) {
      addScriptState(props.project.id, event.payload[1])
    }
  })

  // 监听脚本输出
  unlistenOutput = await listen<{ project_id: string; output: string; is_error: boolean }>('script-output', (event) => {
    if (event.payload.project_id === props.project.id) {
      if (event.payload.output === '__SCRIPT_COMPLETED__') {
        isRunningScript.value = false
        markScriptCompleted(props.project.id)
      } else {
        scriptOutput.value += event.payload.output + '\n'
        updateScriptOutput(props.project.id, event.payload.output)
      }
    }
  })

  // 监听自动刷新事件
  window.addEventListener('projects-auto-refresh', handleAutoRefresh)
})

onUnmounted(() => {
  if (unlistenOutput) {
    unlistenOutput()
  }
  window.removeEventListener('projects-auto-refresh', handleAutoRefresh)
})

// 处理自动刷新
const handleAutoRefresh = async () => {
  await loadProjectData()
}

const openInVscode = async () => {
  try {
    await invoke('open_in_vscode', { path: props.project.path })
  } catch (err) {
    console.error('Failed to open in VSCode:', err)
  }
}

const refreshProject = async () => {
  await loadProjectData()
}

const toggleBranches = () => {
  isBranchesCollapsed.value = !isBranchesCollapsed.value
}

const loadBranches = async (checkGit = true) => {
  // 直接使用传入的 checkGit 参数，不再额外检查 computed 属性
  if (!checkGit) {
    branches.value = []
    currentBranch.value = ''
    return
  }
  try {
    console.log('[ProjectCard] Loading branches for:', props.project.path)
    branches.value = await invoke('get_git_branches', { path: props.project.path })
    const current = branches.value.find(b => b.current)
    currentBranch.value = current?.name || ''
    console.log('[ProjectCard] Loaded branches:', branches.value.length, 'for:', props.project.name)
  } catch (err) {
    console.error('[ProjectCard] Failed to load branches:', err)
    branches.value = []
    currentBranch.value = ''
  }
}

const switchBranch = async (branch: string) => {
  try {
    await invoke('switch_git_branch', { path: props.project.path, branch })
    currentBranch.value = branch
    await loadBranches()
  } catch (err) {
    console.error('Failed to switch branch:', err)
    alert('切换分支失败: ' + err)
  }
}

const loadNpmScripts = async () => {
  try {
    npmScripts.value = await invoke('get_npm_scripts', { path: props.project.path })
  } catch (err) {
    console.error('Failed to load npm scripts:', err)
    npmScripts.value = []
  }
}

const runNpmScript = async (scriptName: string) => {
  isRunningScript.value = true
  scriptOutput.value = ''
  try {
    await invoke('run_npm_script', {
      projectId: props.project.id,
      path: props.project.path,
      scriptName
    })
  } catch (err) {
    isRunningScript.value = false
    scriptOutput.value = String(err)
    alert('脚本执行失败: ' + err)
  }
}

const runCustomScript = async () => {
  if (!customScript.value.trim()) return

  isRunningScript.value = true
  scriptOutput.value = ''
  try {
    await invoke('run_custom_script', {
      projectId: props.project.id,
      path: props.project.path,
      script: customScript.value
    })
    customScript.value = ''
  } catch (err) {
    isRunningScript.value = false
    scriptOutput.value = String(err)
    alert('脚本执行失败: ' + err)
  }
}

const killScript = async () => {
  try {
    await invoke('kill_script', { projectId: props.project.id })
    isRunningScript.value = false
    scriptOutput.value += '\n--- 脚本已终止 ---\n'
  } catch (err) {
    console.error('Failed to kill script:', err)
  }
}

const openSettings = () => {
  isSettingsVisible.value = true
}

// 打开输出 Modal
const openOutputModal = () => {
  isOutputModalVisible.value = true
}

// 保存产物命名配置
const saveProductConfig = async () => {
  try {
    await invoke('update_project_config', {
      id: props.project.id,
      productNameTemplate: productNameTemplate.value || props.project.name,
      addTimestamp: addTimestamp.value
    })
    message.success('配置已保存')
  } catch (err) {
    console.error('Failed to save config:', err)
    message.error('保存失败: ' + err)
  }
}

// 复制 dist 并打包 zip
const copyDistAndZip = async () => {
  try {
    const result = await invoke<string>('copy_dist_and_zip', {
      id: props.project.id,
      path: props.project.path
    })
    message.success('已生成: ' + result)
  } catch (err) {
    console.error('Failed to copy dist and zip:', err)
    message.error('操作失败: ' + err)
  }
}
</script>

<template>
  <div class="project-card expanded">
    <div class="card-main">
      <div class="card-left">
        <div class="logo">
          <span>📂</span>
        </div>
      </div>
      <div class="card-right">
        <div class="project-name-row">
          <h3 class="project-name">{{ project.name }}</h3>
          <span v-if="lastUpdatedText" class="last-updated">{{ lastUpdatedText }}</span>
        </div>
        <div class="actions">
          <button class="btn-open" @click="openInVscode">
            用 VSCode 打开
          </button>
          <button class="btn-remove" @click="emit('remove', project.id)">
            移除
          </button>
        </div>
      </div>
      <div class="card-actions">
        <button class="icon-btn" title="刷新" @click="refreshProject" :disabled="isLoading">
          <Icon icon="mdi:refresh" :class="{ 'spin': isLoading }" />
        </button>
        <button class="icon-btn" title="设置" @click="openSettings">
          <Icon icon="mdi:cog" />
        </button>
      </div>
    </div>

    <!-- Loading 状态 -->
    <div v-if="isLoading" class="loading-overlay">
      <Icon icon="mdi:loading" class="loading-spinner" />
      <span>加载中...</span>
    </div>

    <!-- 详细信息区域（始终显示） -->
    <div class="expanded-content">
      <!-- Git 分支 -->
      <div v-if="isGitRepo" class="section">
        <div class="section-header" @click="toggleBranches">
          <h4>Git 分支</h4>
          <span v-if="currentBranch" class="current-branch">({{ currentBranch }})</span>
          <span class="collapse-icon">{{ isBranchesCollapsed ? '▶' : '▼' }}</span>
        </div>
        <div v-if="!isBranchesCollapsed" class="branch-list">
          <button
            v-for="branch in branches"
            :key="branch.name"
            class="branch-btn"
            :class="{ current: branch.current }"
            @click="switchBranch(branch.name)"
          >
            {{ branch.name }}{{ branch.current ? ' ✓' : '' }}
          </button>
        </div>
      </div>
      <div v-else class="section">
        <p class="not-git">该目录不是 Git 仓库</p>
      </div>

      <!-- NPM 脚本 -->
      <div class="section">
        <h4>NPM 脚本</h4>
        <div v-if="npmScripts.length > 0" class="script-list">
          <button
            v-for="script in npmScripts"
            :key="script.name"
            class="script-btn"
            :disabled="isRunningScript"
            @click="runNpmScript(script.name)"
          >
            {{ script.name }}
          </button>
        </div>
        <p v-else class="no-scripts">未找到 package.json 或无可用脚本</p>
      </div>

      <!-- 自定义脚本 -->
      <div class="section">
        <h4>自定义脚本</h4>
        <div class="custom-script">
          <input
            v-model="customScript"
            type="text"
            placeholder="输入自定义脚本，如: npm install"
            :disabled="isRunningScript"
            @keyup.enter="runCustomScript"
          />
          <button
            class="run-btn"
            :disabled="isRunningScript || !customScript.trim()"
            @click="runCustomScript"
          >
            {{ isRunningScript ? '运行中...' : '执行' }}
          </button>
        </div>
      </div>

      <!-- 脚本输出和终止按钮 -->
      <div v-if="isRunningScript || scriptOutput" class="section">
        <div class="output-header">
          <h4>输出</h4>
          <div class="output-actions">
            <button class="output-btn" @click="openOutputModal">
              <Icon icon="mdi:console" />
              查看输出
            </button>
            <button v-if="isRunningScript" class="kill-btn" @click="killScript">
              <Icon icon="mdi:stop" />
              终止
            </button>
          </div>
        </div>
        <pre class="output">{{ scriptOutput }}</pre>
      </div>

      <!-- 产物命名配置 -->
      <div class="section">
        <h4>产物命名</h4>
        <div class="product-config">
          <div class="config-row">
            <span class="config-label">命名模板:</span>
            <input
              v-model="productNameTemplate"
              type="text"
              placeholder="输入命名模板"
              class="config-input"
            />
          </div>
          <div class="config-row">
            <span class="config-label">添加时间戳:</span>
            <Switch v-model:checked="addTimestamp" />
          </div>
          <div class="config-actions">
            <button class="save-btn" @click="saveProductConfig">
              保存配置
            </button>
            <button class="zip-btn" @click="copyDistAndZip">
              <Icon icon="mdi:folder-zip" />
              复制 dist 并打包
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 设置弹窗 -->
    <Modal
      v-model:open="isSettingsVisible"
      title="项目设置"
      :footer="null"
      :width="400"
    >
      <p>项目: {{ project.name }}</p>
      <p>路径: {{ project.path }}</p>
      <p style="color: #999; margin-top: 16px;">设置功能开发中...</p>
    </Modal>

    <!-- 输出 Modal -->
    <Modal
      v-model:open="isOutputModalVisible"
      title="脚本输出"
      :footer="null"
      :width="800"
    >
      <pre class="modal-output">{{ scriptOutput || '暂无输出' }}</pre>
    </Modal>
  </div>
</template>

<style scoped>
.project-card {
  background: #fff;
  border-radius: 8px;
  padding: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  transition: all 0.3s;
  position: relative;
}

.project-card:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.project-card.expanded {
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
}

.card-main {
  display: flex;
}

.card-left {
  margin-right: 16px;
}

.logo {
  width: 48px;
  height: 48px;
  background: #f0f0f0;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
}

.card-right {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.project-name {
  font-size: 16px;
  font-weight: 600;
  color: #333;
  margin: 0;
  word-break: break-all;
}

.project-name-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.last-updated {
  font-size: 11px;
  color: #999;
  font-weight: normal;
}

.actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.btn-open,
.btn-expand,
.btn-remove {
  padding: 6px 12px;
  border: none;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
  transition: background 0.2s;
}

.btn-open {
  background: #1890ff;
  color: #fff;
}

.btn-open:hover {
  background: #40a9ff;
}

.btn-expand {
  background: #52c41a;
  color: #fff;
}

.btn-expand:hover {
  background: #73d13d;
}

.btn-remove {
  background: #ff4d4f;
  color: #fff;
}

.btn-remove:hover {
  background: #ff7875;
}

.card-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
  position: absolute;
  top: 12px;
  right: 12px;
}

.icon-btn {
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 4px;
  background: #f0f0f0;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  color: #666;
  transition: all 0.2s;
}

.icon-btn:hover:not(:disabled) {
  background: #e0e0e0;
  color: #1890ff;
}

.icon-btn:disabled {
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

.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.9);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  z-index: 10;
  border-radius: 8px;
}

.loading-spinner {
  font-size: 32px;
  color: #1890ff;
  animation: spin 1s linear infinite;
}

.expanded-content {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid #eee;
}

.section {
  margin-bottom: 16px;
}

.section-header {
  display: flex;
  align-items: center;
  cursor: pointer;
  margin-bottom: 8px;
}

.section-header h4 {
  font-size: 14px;
  font-weight: 600;
  color: #333;
  margin: 0;
}

.current-branch {
  margin-left: 8px;
  color: #1890ff;
  font-weight: 500;
  font-size: 13px;
}

.collapse-icon {
  margin-left: auto;
  font-size: 10px;
  color: #999;
}

.not-git,
.no-scripts {
  color: #999;
  font-size: 13px;
}

.branch-list,
.script-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.branch-btn,
.script-btn {
  padding: 4px 10px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  background: #fff;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.branch-btn:hover,
.script-btn:hover:not(:disabled) {
  border-color: #1890ff;
  color: #1890ff;
}

.branch-btn.current {
  background: #1890ff;
  color: #fff;
  border-color: #1890ff;
}

.script-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.custom-script {
  display: flex;
  gap: 8px;
}

.custom-script input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  font-size: 13px;
}

.custom-script input:focus {
  outline: none;
  border-color: #1890ff;
}

.run-btn {
  padding: 8px 16px;
  background: #1890ff;
  color: #fff;
  border: none;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
}

.run-btn:hover:not(:disabled) {
  background: #40a9ff;
}

.run-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.output-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.output-header h4 {
  font-size: 14px;
  font-weight: 600;
  color: #333;
  margin: 0;
}

.output-actions {
  display: flex;
  gap: 8px;
}

.output-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 12px;
  background: #1890ff;
  color: #fff;
  border: none;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
}

.output-btn:hover {
  background: #40a9ff;
}

.kill-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 12px;
  background: #ff4d4f;
  color: #fff;
  border: none;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
}

.kill-btn:hover {
  background: #ff7875;
}

.output {
  background: #f5f5f5;
  padding: 12px;
  border-radius: 4px;
  font-size: 12px;
  max-height: 200px;
  overflow-y: auto;
  white-space: pre-wrap;
  word-break: break-all;
}

.product-config {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.config-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.config-label {
  font-size: 13px;
  color: #666;
  min-width: 80px;
}

.config-input {
  flex: 1;
  padding: 6px 10px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  font-size: 13px;
}

.config-input:focus {
  outline: none;
  border-color: #1890ff;
}

.config-actions {
  display: flex;
  gap: 8px;
  margin-top: 4px;
}

.save-btn {
  padding: 6px 16px;
  background: #52c41a;
  color: #fff;
  border: none;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
}

.save-btn:hover {
  background: #73d13d;
}

.zip-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 16px;
  background: #722ed1;
  color: #fff;
  border: none;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
}

.zip-btn:hover {
  background: #9254de;
}

.modal-output {
  background: #1e1e1e;
  color: #d4d4d4;
  padding: 16px;
  border-radius: 4px;
  font-size: 13px;
  max-height: 500px;
  overflow-y: auto;
  white-space: pre-wrap;
  word-break: break-all;
  font-family: 'Consolas', 'Monaco', monospace;
}
</style>