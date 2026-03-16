<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { Modal, Switch, message, Dropdown } from 'ant-design-vue'
import { Icon } from '@iconify/vue'
import type { Project } from '../types/project'
import { useScriptStore } from '../stores/scriptStore'
import { markProjectLoaded, isProjectLoaded, clearProjectLoadStatus } from '../stores/projectDataStore'
import { useProjectStore } from '../stores/projectStore'
import { useGroupStore } from '../stores/groupStore'
import { cacheProjectData, getCachedProjectData } from '../stores/cachedProjectData'

const props = defineProps<{
  project: Project
  groupColor?: string
}>()

const emit = defineEmits<{
  (e: 'remove', id: string): void
}>()

const { addScriptState, updateScriptOutput, markScriptCompleted, getScriptState } = useScriptStore()
const { state: projectState } = useProjectStore()
const { state: groupState, bindProjectToGroup, loadGroups } = useGroupStore()

// 固定展开，无折叠功能
const isBranchesCollapsed = ref(false)
const branches = ref<{ name: string; current: boolean }[]>([])
const currentBranch = ref('')
const npmScripts = ref<{ name: string; script: string }[]>([])
const customScript = ref('')
const scriptOutput = ref('')
const isRunningScript = ref(false)
const isSettingsVisible = ref(false)
const isZipping = ref(false)
const selectedGroupId = ref<string | null>(null)
const isLoading = ref(false)
const isDataLoaded = ref(false)

// 输出 Modal
const isOutputModalVisible = ref(false)

// 产物命名配置
const productNameTemplate = ref('')
const addTimestamp = ref(false)
const addBranch = ref(false)
const addEnv = ref(false)
const envName = ref('')
const addVersion = ref(false)
const versionName = ref('')

// 原始配置备份（用于比较是否有变更）
const originalConfig = ref({
  productNameTemplate: '',
  addTimestamp: false,
  addBranch: false,
  addEnv: false,
  envName: '',
  addVersion: false,
  versionName: ''
})

// 判断配置是否有变更
const hasProductConfigChanges = computed(() => {
  return productNameTemplate.value !== originalConfig.value.productNameTemplate ||
    addTimestamp.value !== originalConfig.value.addTimestamp ||
    addBranch.value !== originalConfig.value.addBranch ||
    addEnv.value !== originalConfig.value.addEnv ||
    envName.value !== originalConfig.value.envName ||
    addVersion.value !== originalConfig.value.addVersion ||
    versionName.value !== originalConfig.value.versionName
})

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

// 是否有分组背景
const hasGroupBackground = computed(() => !!props.groupColor)

// 分组背景样式
const groupBackgroundStyle = computed(() => {
  if (!props.groupColor) return {}
  return {
    '--group-color': props.groupColor
  }
})

let unlistenOutput: UnlistenFn | null = null

// 加载项目数据
const loadProjectData = async () => {
  // 初始化产物命名配置
  productNameTemplate.value = props.project.productNameTemplate || props.project.name
  addTimestamp.value = props.project.addTimestamp || false
  addBranch.value = props.project.addBranch || false
  addEnv.value = props.project.addEnv || false
  envName.value = props.project.envName || ''
  addVersion.value = props.project.addVersion || false
  versionName.value = props.project.versionName || ''

  // 保存原始配置
  originalConfig.value = {
    productNameTemplate: productNameTemplate.value,
    addTimestamp: addTimestamp.value,
    addBranch: addBranch.value,
    addEnv: addEnv.value,
    envName: envName.value,
    addVersion: addVersion.value,
    versionName: versionName.value
  }

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

const openInCursor = async () => {
  try {
    await invoke('open_in_cursor', { path: props.project.path })
  } catch (err) {
    console.error('Failed to open in Cursor:', err)
  }
}

const openInWebstorm = async () => {
  try {
    await invoke('open_in_webstorm', { path: props.project.path })
  } catch (err) {
    console.error('Failed to open in WebStorm:', err)
  }
}

const openInTrae = async () => {
  try {
    await invoke('open_in_trae', { path: props.project.path })
  } catch (err) {
    console.error('Failed to open in Trae:', err)
  }
}

const refreshProject = async () => {
  // 刷新前先清除加载状态，强制重新加载数据
  isDataLoaded.value = false
  clearProjectLoadStatus(props.project.id)
  await loadProjectData()
  // 更新项目的最后更新时间
  const now = Math.floor(Date.now() / 1000)
  props.project.lastUpdatedAt = now
  // 同时更新 store 中的数据
  const project = projectState.projects.find(p => p.id === props.project.id)
  if (project) {
    project.lastUpdatedAt = now
  }
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
    Modal.error({
      title: '切换分支失败',
      content: String(err),
      okText: '确定'
    })
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
    Modal.error({
      title: '脚本执行失败',
      content: String(err),
      okText: '确定'
    })
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
    Modal.error({
      title: '脚本执行失败',
      content: String(err),
      okText: '确定'
    })
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
  selectedGroupId.value = props.project.groupId || null
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
      addTimestamp: addTimestamp.value,
      addBranch: addBranch.value,
      addEnv: addEnv.value,
      envName: envName.value,
      addVersion: addVersion.value,
      versionName: versionName.value
    })
    // 更新 projectStore 中的项目数据
    const project = projectState.projects.find(p => p.id === props.project.id)
    if (project) {
      project.productNameTemplate = productNameTemplate.value || props.project.name
      project.addTimestamp = addTimestamp.value
      project.addBranch = addBranch.value
      project.addEnv = addEnv.value
      project.envName = envName.value
      project.addVersion = addVersion.value
      project.versionName = versionName.value
    }
    // 更新原始配置
    originalConfig.value = {
      productNameTemplate: productNameTemplate.value,
      addTimestamp: addTimestamp.value,
      addBranch: addBranch.value,
      addEnv: addEnv.value,
      envName: envName.value,
      addVersion: addVersion.value,
      versionName: versionName.value
    }
    message.success('配置已保存')
  } catch (err) {
    console.error('Failed to save config:', err)
    message.error('保存失败: ' + err)
  }
}

// 绑定项目到分组
const handleBindGroup = async () => {
  try {
    await bindProjectToGroup(props.project.id, selectedGroupId.value)
    // 更新本地项目数据
    const project = projectState.projects.find(p => p.id === props.project.id)
    if (project) {
      project.groupId = selectedGroupId.value || undefined
    }
    message.success('分组已更新')
    // 刷新分组列表
    await loadGroups()
  } catch (err) {
    message.error('绑定失败: ' + err)
  }
}

// 复制 dist 并打包 zip
const copyDistAndZip = async () => {
  isZipping.value = true
  try {
    const result = await invoke<string>('copy_dist_and_zip', {
      id: props.project.id,
      path: props.project.path
    })
    message.success('已生成: ' + result)
  } catch (err) {
    console.error('Failed to copy dist and zip:', err)
    message.error('操作失败: ' + err)
  } finally {
    isZipping.value = false
  }
}
</script>

<template>
  <div class="project-card expanded" :class="{ 'has-group-bg': hasGroupBackground }" :style="groupBackgroundStyle">
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
          <Dropdown :trigger="['click']">
            <button class="btn-open">
              <Icon icon="mdi:folder-eye" />
              打开方式
              <span class="dropdown-arrow">▼</span>
            </button>
            <template #overlay>
              <div class="ide-dropdown">
                <div class="ide-option" @click="openInVscode">
                  <Icon icon="mdi:microsoft-visual-studio" class="ide-icon vscode" />
                  <span>VSCode</span>
                </div>
                <div class="ide-option" @click="openInCursor">
                  <Icon icon="mdi:cursor-default" class="ide-icon cursor" />
                  <span>Cursor</span>
                </div>
                <div class="ide-option" @click="openInWebstorm">
                  <Icon icon="mdi:jetbrains" class="ide-icon webstorm" />
                  <span>WebStorm</span>
                </div>
                <div class="ide-option" @click="openInTrae">
                  <Icon icon="mdi:robot" class="ide-icon trae" />
                  <span>Trae</span>
                </div>
              </div>
            </template>
          </Dropdown>
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
          <div class="config-row">
            <span class="config-label">添加分支名:</span>
            <Switch v-model:checked="addBranch" />
            <input
              v-if="addBranch"
              v-model="currentBranch"
              type="text"
              placeholder="当前分支名"
              class="config-input-small"
              disabled
            />
          </div>
          <div class="config-row">
            <span class="config-label">添加环境名:</span>
            <Switch v-model:checked="addEnv" />
            <input
              v-if="addEnv"
              v-model="envName"
              type="text"
              placeholder="如: dev, test, prod"
              class="config-input-small"
            />
          </div>
          <div class="config-row">
            <span class="config-label">添加版本号:</span>
            <Switch v-model:checked="addVersion" />
            <input
              v-if="addVersion"
              v-model="versionName"
              type="text"
              placeholder="如: v1.0.0"
              class="config-input-small"
            />
          </div>
          <div class="config-actions">
            <button class="save-btn" @click="saveProductConfig" :disabled="!hasProductConfigChanges">
              保存配置
            </button>
            <button class="zip-btn" @click="copyDistAndZip" :disabled="isZipping">
              <Icon icon="mdi:folder-zip" :class="{ 'spin': isZipping }" />
              {{ isZipping ? '打包中...' : '复制 dist 并打包' }}
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
      <div class="settings-modal-content">
        <div class="settings-info">
          <p><strong>项目:</strong> {{ project.name }}</p>
          <p><strong>路径:</strong> {{ project.path }}</p>
        </div>
        <div class="settings-group-section">
          <label>分组</label>
          <div class="group-select-wrapper">
            <select v-model="selectedGroupId" class="group-select">
              <option :value="null">无</option>
              <option v-for="group in groupState.groups" :key="group.id" :value="group.id">
                {{ group.name }}
              </option>
            </select>
            <button class="bind-group-btn" @click="handleBindGroup">绑定</button>
          </div>
        </div>
        <div class="settings-actions">
          <button class="btn-remove-in-modal" @click="emit('remove', project.id); isSettingsVisible = false">
            <Icon icon="mdi:delete" />
            移除此项目
          </button>
        </div>
      </div>
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
  background: var(--bg-card);
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

.project-card.has-group-bg {
  background: linear-gradient(var(--bg-card), var(--bg-card)) padding-box,
              linear-gradient(135deg, var(--group-color, var(--primary-color)), rgba(255, 255, 255, 0.5)) border-box !important;
  border: 3px solid transparent;
  border-radius: 8px;
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
  background: var(--bg-color);
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
  color: var(--text-primary);
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
  color: var(--text-secondary);
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
  background: var(--primary-color);
  color: #fff;
}

.btn-open:hover {
  background: var(--primary-hover);
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

.dropdown-arrow {
  font-size: 10px;
  margin-left: 4px;
}

.ide-dropdown {
  background: var(--bg-card);
  border-radius: 8px;
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.12);
  padding: 4px 0;
  min-width: 140px;
}

.ide-option {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  cursor: pointer;
  transition: background 0.2s;
  color: var(--text-primary);
}

.ide-option:hover {
  background: var(--bg-color);
}

.ide-icon {
  font-size: 18px;
}

.ide-icon.vscode {
  color: #007acc;
}

.ide-icon.cursor {
  color: #3b82f6;
}

.ide-icon.webstorm {
  color: #087cfa;
}

.ide-icon.trae {
  color: #ff6b35;
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
  background: var(--bg-color);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  color: var(--text-secondary);
  transition: all 0.2s;
}

.icon-btn:hover:not(:disabled) {
  background: var(--border-color);
  color: var(--primary-color);
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
  background: var(--bg-card);
  opacity: 0.95;
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
  color: var(--primary-color);
  animation: spin 1s linear infinite;
}

.expanded-content {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid var(--border-color);
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
  color: var(--text-primary);
  margin: 0;
}

.current-branch {
  margin-left: 8px;
  color: var(--primary-color);
  font-weight: 500;
  font-size: 13px;
}

.collapse-icon {
  margin-left: auto;
  font-size: 10px;
  color: var(--text-secondary);
}

.not-git,
.no-scripts {
  color: var(--text-secondary);
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
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: var(--bg-card);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.branch-btn:hover,
.script-btn:hover:not(:disabled) {
  border-color: var(--primary-color);
  color: var(--primary-color);
}

.branch-btn.current {
  background: var(--primary-color);
  color: #fff;
  border-color: var(--primary-color);
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
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-size: 13px;
  background: var(--bg-card);
  color: var(--text-primary);
}

.custom-script input:focus {
  outline: none;
  border-color: #1890ff;
}

.run-btn {
  padding: 8px 16px;
  background: var(--primary-color);
  color: #fff;
  border: none;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
}

.run-btn:hover:not(:disabled) {
  background: var(--primary-hover);
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
  color: var(--text-primary);
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
  background: var(--primary-color);
  color: #fff;
  border: none;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
}

.output-btn:hover {
  background: var(--primary-hover);
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
  background: var(--bg-color);
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
  color: var(--text-secondary);
  min-width: 80px;
}

.config-input {
  flex: 1;
  padding: 6px 10px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-size: 13px;
  background: var(--bg-card);
  color: var(--text-primary);
}

.config-input:focus {
  outline: none;
  border-color: #1890ff;
}

.config-input-small {
  padding: 4px 8px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-size: 12px;
  width: 120px;
  margin-left: 8px;
  background: var(--bg-card);
  color: var(--text-primary);
}

.config-input-small:focus {
  outline: none;
  border-color: #1890ff;
}

.config-input-small:disabled {
  background: var(--bg-color);
  color: var(--text-secondary);
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

.save-btn:hover:not(:disabled) {
  background: #73d13d;
}

.save-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
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

.settings-modal-content {
  padding: 8px 0;
}

.settings-info {
  margin-bottom: 20px;
}

.settings-info p {
  margin-bottom: 8px;
  word-break: break-all;
}

.settings-group-section {
  margin-bottom: 16px;
}

.settings-group-section label {
  display: block;
  margin-bottom: 8px;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.group-select-wrapper {
  display: flex;
  gap: 8px;
}

.group-select {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-size: 14px;
  background: var(--bg-card);
  color: var(--text-primary);
}

.bind-group-btn {
  padding: 8px 16px;
  background: var(--primary-color);
  color: #fff;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  cursor: pointer;
}

.bind-group-btn:hover {
  background: var(--primary-hover);
}

.settings-actions {
  border-top: 1px solid var(--border-color);
  padding-top: 16px;
}

.btn-remove-in-modal {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  background: #ff4d4f;
  color: #fff;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  cursor: pointer;
  transition: background 0.2s;
}

.btn-remove-in-modal:hover {
  background: #ff7875;
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