<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { Input, Switch, Modal, message } from 'ant-design-vue'
import { Icon } from '@iconify/vue'
import { useTheme } from '../stores/themeStore'
import { useErrorLogStore } from '../stores/errorLogStore'
import { useIdeConfigStore, type IdeConfig } from '../stores/ideConfigStore'

const { themeConfig, setPrimaryColor, setDarkMode } = useTheme()
const { errorLogs, refreshErrorLogs, clearErrorLogs } = useErrorLogStore()
const { ideConfigs } = useIdeConfigStore()

// 配置项
const distOutputPath = ref('')
const autoRefreshEnabled = ref(false)
const autoRefreshInterval = ref(60)

// IDE 配置列表
const availableIdes = [
  { id: 'vscode', name: 'VSCode', icon: 'mdi:microsoft-visual-studio' },
  { id: 'cursor', name: 'Cursor', icon: 'mdi:cursor-default' },
  { id: 'webstorm', name: 'WebStorm', icon: 'mdi:jetbrains' },
  { id: 'trae', name: 'Trae', icon: 'mdi:robot' },
  { id: 'wechat', name: '微信开发者工具', icon: 'mdi:wechat' }
]

// 主题色
const primaryColor = ref('#1890ff')
const isDarkMode = ref(false)

// 功能介绍弹窗
const isFeatureModalVisible = ref(false)

// 功能点介绍数据
const features = [
  {
    title: '项目列表',
    icon: 'mdi:folder-multiple',
    description: '展示所有已添加的项目，支持拖拽添加新项目，可搜索项目名称。拖拽添加时自动绑定到当前选中的分组。',
  },
  {
    title: 'IDE 打开',
    icon: 'mdi:microsoft-visual-studio',
    description: '支持多种 IDE 打开项目：VSCode、Cursor、WebStorm、Trae、微信开发者工具。仅显示已配置的 IDE。',
  },
  {
    title: 'Git 分支管理',
    icon: 'mdi:git',
    description: '显示当前 Git 仓库的分支列表，支持切换分支。',
  },
  {
    title: 'NPM 脚本',
    icon: 'mdi:npm',
    description: '自动读取并展示 package.json 中的脚本，支持直接运行。',
  },
  {
    title: '自定义脚本',
    icon: 'mdi:console',
    description: '可输入自定义命令执行，如 npm install、yarn add 等。所有命令后台执行，不弹出终端窗口。',
  },
  {
    title: '产物打包',
    icon: 'mdi:folder-zip',
    description: '一键复制 dist 目录并打包成 zip 文件，支持自定义命名规则。',
  },
  {
    title: '项目分组',
    icon: 'mdi:folder-group',
    description: '支持创建分组并绑定项目，方便按分类管理项目。可批量选择项目并绑定到指定分组。',
  },
  {
    title: '项目备注',
    icon: 'mdi:note-text',
    description: '可为每个项目添加备注信息，方便记录项目相关说明。',
  },
  {
    title: '错误日志',
    icon: 'mdi:alert-circle',
    description: '自动记录应用运行中的错误信息，便于问题排查。保留最近 24 小时的错误日志。',
  },
  {
    title: '自动刷新',
    icon: 'mdi:refresh-auto',
    description: '可设置自动刷新间隔，自动更新项目的 Git 分支和 NPM 脚本信息。',
  },
  {
    title: '主题定制',
    icon: 'mdi:palette',
    description: '支持自定义主题颜色和夜间模式，提供多种预设颜色。',
  },
]

// 版本更新数据
const versionHistory = [
  {
    version: '1.0.2',
    date: '2026-03-17',
    isLatest: true,
    features: [
      '内存泄漏修复 - 修复多处事件监听器未清理导致的内存泄漏',
      '脚本状态清理 - 脚本执行完成后正确清理后端进程状态',
    ],
  },
  {
    version: '1.0.1',
    date: '2026-03-17',
    isLatest: false,
    features: [
      'IDE 配置优化 - 手动配置 IDE 路径，仅显示已配置的 IDE',
      '新增微信开发者工具支持',
      '项目备注功能 - 可为项目添加备注信息',
      '批量导入优化 - 自动创建分组时可选择是否绑定到该分组',
      '拖拽添加项目自动绑定到当前选中的分组',
      '终端窗口完全隐藏 - 所有操作后台执行不再弹出终端',
      '启动优化 - 首次启动不再自动刷新所有项目',
      '左侧菜单优化 - 添加图标，渐变效果优化',
      '错误日志模块 - 存储和查看应用错误日志',
      '即时保存 - 设置和项目配置修改即时生效',
      '修复分组问题 - 拖拽添加项目时正确绑定到选中的分组',
    ],
  },
  {
    version: '1.0.0',
    date: '2026-03-16',
    isLatest: false,
    features: [
      '项目列表 - 支持拖拽添加项目、搜索筛选',
      '多 IDE 支持 - VSCode、Cursor、WebStorm、Trae',
      'Git 分支管理 - 查看分支、切换分支',
      'NPM 脚本 - 自动读取并运行脚本',
      '自定义脚本 - 执行自定义命令',
      '产物打包 - 一键复制 dist 并打包 zip',
      '项目分组 - 创建分组管理项目',
      '自动刷新 - 自动更新项目信息',
      '主题定制 - 自定义主题颜色和夜间模式',
      '暗色模式完整适配',
      'Apple 风格滚动条',
    ],
  },
]

// 版本更新弹窗
const isVersionModalVisible = ref(false)

const openVersionModal = () => {
  isVersionModalVisible.value = true
}

const openFeatureModal = () => {
  isFeatureModalVisible.value = true
}

// 预设主题色
const presetColors = [
  '#1890ff', // 蓝色
  '#52c41a', // 绿色
  '#faad14', // 黄色
  '#fa541c', // 橙色
  '#f5222d', // 红色
  '#722ed1', // 紫色
  '#13c2c2', // 青色
  '#eb2f96', // 粉色
]

// 错误日志相关
const isErrorLogModalVisible = ref(false)

const openErrorLogModal = () => {
  refreshErrorLogs()
  isErrorLogModalVisible.value = true
}

const handleClearErrorLogs = () => {
  clearErrorLogs()
  refreshErrorLogs()
  message.success('错误日志已清除')
}

const formatErrorTime = (timestamp: number) => {
  const date = new Date(timestamp)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  const minutes = Math.floor(diff / 60000)
  const hours = Math.floor(diff / 3600000)

  if (minutes < 1) return '刚刚'
  if (minutes < 60) return `${minutes}分钟前`
  if (hours < 24) return `${hours}小时前`
  return date.toLocaleString('zh-CN')
}

// 定时器
let refreshTimer: number | null = null

// 保存配置（自动保存）
const saveConfig = async () => {
  try {
    await invoke('save_config', {
      distOutputPath: distOutputPath.value,
      autoRefreshEnabled: autoRefreshEnabled.value,
      autoRefreshInterval: String(autoRefreshInterval.value)
    })
    // 保存主题配置
    setPrimaryColor(primaryColor.value)
    setDarkMode(isDarkMode.value)
    // 更新定时器
    setupAutoRefresh()
  } catch (err) {
    console.error('Failed to save config:', err)
    message.error('保存失败: ' + err)
  }
}

// 加载配置
const loadConfig = async () => {
  try {
    const config = await invoke<{ distOutputPath: string; autoRefreshEnabled: boolean; autoRefreshInterval: number }>('get_config')
    distOutputPath.value = config.distOutputPath || ''
    autoRefreshEnabled.value = config.autoRefreshEnabled || false
    autoRefreshInterval.value = config.autoRefreshInterval || 60
    // 加载主题配置
    primaryColor.value = themeConfig.primaryColor
    isDarkMode.value = themeConfig.isDarkMode
  } catch (err) {
    console.error('Failed to load config:', err)
  }
}

// 加载 IDE 配置
const loadIdeConfigs = async () => {
  try {
    const configs = await invoke<IdeConfig[]>('get_ide_configs')
    ideConfigs.value = configs
  } catch (err) {
    console.error('Failed to load IDE configs:', err)
  }
}

// 保存单个 IDE 配置
const saveIdeConfig = async (ide: typeof availableIdes[0], path: string, enabled: boolean) => {
  try {
    await invoke('save_ide_config', {
      id: ide.id,
      name: ide.name,
      path: path,
      enabled: enabled
    })
    // 更新本地状态
    const existing = ideConfigs.value.find(c => c.id === ide.id)
    if (existing) {
      existing.path = path
      existing.enabled = enabled
    } else {
      ideConfigs.value.push({
        id: ide.id,
        name: ide.name,
        path: path,
        enabled: enabled
      })
    }
  } catch (err) {
    console.error('Failed to save IDE config:', err)
    message.error('保存失败: ' + err)
  }
}

// 选择 IDE 路径
const selectIdePath = async (ide: typeof availableIdes[0]) => {
  try {
    const selected = await open({
      directory: false,
      multiple: false,
      title: `选择 ${ide.name} 可执行文件`
    })
    if (selected) {
      await saveIdeConfig(ide, selected as string, true)
      message.success(`${ide.name} 路径已配置`)
    }
  } catch (err) {
    console.error('Failed to select IDE path:', err)
  }
}

// 监听配置变化，自动保存
watch([distOutputPath, autoRefreshEnabled, autoRefreshInterval, primaryColor, isDarkMode], () => {
  saveConfig()
}, { deep: true })

// 选择目录
const selectOutputPath = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择压缩包输出目录'
    })
    if (selected) {
      distOutputPath.value = selected as string
    }
  } catch (err) {
    console.error('Failed to select directory:', err)
  }
}

// 设置自动刷新定时器
const setupAutoRefresh = () => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = null
  }

  if (autoRefreshEnabled.value && autoRefreshInterval.value > 0) {
    const intervalMs = autoRefreshInterval.value * 60 * 1000
    refreshTimer = window.setInterval(async () => {
      try {
        // 仅刷新项目元数据，不触发项目卡片的重新加载
        await invoke('refresh_all_projects')
      } catch (err) {
        console.error('Auto refresh failed:', err)
      }
    }, intervalMs)
  }
}

// 选择主题色
const selectColor = (color: string) => {
  primaryColor.value = color
}

// 切换夜间模式
const toggleDarkMode = (checked: boolean | string | number) => {
  isDarkMode.value = !!checked
}

onMounted(async () => {
  await loadConfig()
  await loadIdeConfigs()
  setupAutoRefresh()
})

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
  }
})
</script>

<template>
  <div class="settings-container">
    <div class="settings-header">
      <h2 class="page-title">
        <Icon icon="mdi:code-tags" class="title-icon" />
        设置
      </h2>
      <div class="header-actions">
        <button class="feature-btn" @click="openErrorLogModal">
          <Icon icon="mdi:alert-circle" />
          错误日志
        </button>
        <button class="feature-btn" @click="openVersionModal">
          <Icon icon="mdi:history" />
          版本更新
        </button>
        <button class="feature-btn" @click="openFeatureModal">
          <Icon icon="mdi:information-outline" />
          功能介绍
        </button>
      </div>
    </div>

    <div class="settings-section">
      <h3>主题设置</h3>
      <div class="setting-item">
        <span class="setting-label">主题颜色:</span>
        <div class="color-presets">
          <button
            v-for="color in presetColors"
            :key="color"
            class="color-preset"
            :class="{ active: primaryColor === color }"
            :style="{ background: color }"
            @click="selectColor(color)"
          />
          <input type="color" v-model="primaryColor" class="color-input" />
        </div>
      </div>
      <div class="setting-item">
        <span class="setting-label">夜间模式:</span>
        <Switch v-model:checked="isDarkMode" @change="toggleDarkMode" />
      </div>
    </div>

    <div class="settings-section">
      <h3>产物输出</h3>
      <div class="setting-item">
        <span class="setting-label">压缩包默认输出路径:</span>
        <div class="setting-input-group">
          <Input
            v-model:value="distOutputPath"
            placeholder="选择输出目录，留空则默认使用项目目录下的 output 文件夹"
            class="path-input"
          />
          <button class="browse-btn" @click="selectOutputPath">
            <Icon icon="mdi:folder-open" />
            浏览
          </button>
        </div>
      </div>
      <p class="setting-hint">复制 dist 并打包时，压缩包将保存到此目录，默认为项目目录下的 output 文件夹</p>
    </div>

    <div class="settings-section">
      <h3>自动刷新</h3>
      <div class="setting-item">
        <span class="setting-label">启用自动刷新:</span>
        <Switch v-model:checked="autoRefreshEnabled" />
      </div>
      <div class="setting-item" v-if="autoRefreshEnabled">
        <span class="setting-label">刷新间隔 (分钟):</span>
        <Input
          v-model:value="autoRefreshInterval"
          type="number"
          :min="1"
          :max="1440"
          class="interval-input"
        />
      </div>
      <p class="setting-hint">自动刷新所有项目的 Git 分支和 NPM 脚本信息，默认为 60 分钟</p>
    </div>

    <div class="settings-section">
      <h3>IDE 配置</h3>
      <p class="setting-hint">配置 IDE 可执行文件路径后，项目卡片中才会显示对应的打开方式</p>
      <div class="ide-config-list">
        <div v-for="ide in availableIdes" :key="ide.id" class="ide-config-item">
          <div class="ide-config-left">
            <Icon :icon="ide.icon" class="ide-icon" :class="ide.id" />
            <span class="ide-name">{{ ide.name }}</span>
          </div>
          <div class="ide-config-right">
            <Input
              :value="ideConfigs.find(c => c.id === ide.id)?.path || ''"
              :placeholder="`选择 ${ide.name} 可执行文件`"
              class="ide-path-input"
              readonly
            />
            <button class="browse-btn" @click="selectIdePath(ide)">
              <Icon icon="mdi:folder-open" />
              浏览
            </button>
          </div>
        </div>
      </div>
    </div>

    <div class="settings-section">
      <h3>关于</h3>
      <div class="about-info">
        <div class="about-logo">
          <Icon icon="mdi:code-tags" />
        </div>
        <div class="about-content">
          <h4>代码管理器</h4>
          <p class="version">版本 1.0.1</p>
          <p class="description">一款专为开发者设计的项目管理工具，支持多 IDE 打开、Git 分支管理、NPM 脚本运行等功能。</p>
        </div>
      </div>
    </div>

    <!-- 功能介绍弹窗 -->
    <Modal
      v-model:open="isFeatureModalVisible"
      title="功能介绍"
      :footer="null"
      :width="700"
      class="feature-modal"
    >
      <div class="feature-list">
        <div v-for="feature in features" :key="feature.title" class="feature-item">
          <div class="feature-icon">
            <Icon :icon="feature.icon" />
          </div>
          <div class="feature-content">
            <h4>{{ feature.title }}</h4>
            <p>{{ feature.description }}</p>
          </div>
        </div>
      </div>
    </Modal>

    <!-- 版本更新弹窗 -->
    <Modal
      v-model:open="isVersionModalVisible"
      title="版本更新"
      :footer="null"
      :width="600"
      class="version-modal"
    >
      <div class="version-timeline">
        <div v-for="item in versionHistory" :key="item.version" class="version-item">
          <div class="version-header">
            <span class="version-tag" :class="{ latest: item.isLatest }">v{{ item.version }}</span>
            <span class="version-date">{{ item.date }}</span>
          </div>
          <ul class="version-features">
            <li v-for="(feature, index) in item.features" :key="index">{{ feature }}</li>
          </ul>
        </div>
      </div>
    </Modal>

    <!-- 错误日志弹窗 -->
    <Modal
      v-model:open="isErrorLogModalVisible"
      title="错误日志"
      :footer="null"
      :width="700"
      class="error-log-modal"
    >
      <div class="error-log-content">
        <div class="error-log-actions" v-if="errorLogs.length > 0">
          <button class="clear-logs-btn" @click="handleClearErrorLogs">
            <Icon icon="mdi:delete-sweep" />
            清除所有日志
          </button>
        </div>
        <div v-if="errorLogs.length === 0" class="no-error-logs">
          <Icon icon="mdi:check-circle" class="no-error-icon" />
          <span>暂无错误日志</span>
        </div>
        <div v-else class="error-log-list">
          <div v-for="log in errorLogs" :key="log.id" class="error-log-item">
            <div class="error-log-header">
              <span class="error-log-source">{{ log.source }}</span>
              <span class="error-log-time">{{ formatErrorTime(log.timestamp) }}</span>
            </div>
            <div class="error-log-message">{{ log.message }}</div>
          </div>
        </div>
      </div>
    </Modal>
  </div>
</template>

<style scoped>
.settings-container {
  padding: 24px;
  max-width: 800px;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  color: var(--text-primary);
  display: flex;
  align-items: center;
  gap: 8px;
}

.title-icon {
  font-size: 28px;
  color: var(--primary-color);
}

.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.feature-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  color: var(--text-primary);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.feature-btn:hover {
  border-color: var(--primary-color);
  color: var(--primary-color);
}

.settings-section {
  background: var(--bg-card);
  border-radius: 8px;
  padding: 20px;
  margin-bottom: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
}

.settings-section h3 {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 16px;
  color: var(--text-primary);
}

.setting-item {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}

.setting-label {
  font-size: 14px;
  color: var(--text-secondary);
  min-width: 140px;
}

.setting-input-group {
  display: flex;
  gap: 8px;
  flex: 1;
}

.path-input {
  flex: 1;
}

.browse-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  background: var(--primary-color);
  color: #fff;
  border: none;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
  white-space: nowrap;
}

.browse-btn:hover {
  background: var(--primary-hover);
}

.interval-input {
  width: 100px;
}

.setting-hint {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 8px;
}

/* IDE 配置样式 */
.ide-config-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-top: 12px;
}

.ide-config-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px;
  background: var(--bg-color);
  border-radius: 6px;
}

.ide-config-left {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 120px;
}

.ide-icon {
  font-size: 24px;
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

.ide-icon.wechat {
  color: #07c160;
}

.ide-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.ide-config-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  max-width: 400px;
}

.ide-path-input {
  flex: 1;
}

/* 关于卡片 */
.about-info {
  display: flex;
  gap: 16px;
  align-items: flex-start;
}

.about-logo {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 60px;
  height: 60px;
  background: linear-gradient(135deg, var(--primary-color) 0%, #722ed1 100%);
  border-radius: 12px;
  flex-shrink: 0;
}

.about-logo .anticon {
  font-size: 32px;
  color: #fff;
}

.about-content {
  flex: 1;
}

.about-content h4 {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 4px 0;
}

.about-content .version {
  font-size: 14px;
  color: var(--primary-color);
  font-weight: 500;
  margin: 0 0 8px 0;
}

.about-content .description {
  font-size: 13px;
  color: var(--text-secondary);
  margin: 0;
  line-height: 1.5;
}

/* 主题色选择器 */
.color-presets {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: wrap;
}

.color-preset {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  transition: all 0.2s;
}

.color-preset:hover {
  transform: scale(1.1);
}

.color-preset.active {
  border-color: var(--text-primary);
  box-shadow: 0 0 0 2px var(--bg-card), 0 0 0 4px var(--primary-color);
}

.color-input {
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 50%;
  cursor: pointer;
  padding: 0;
  overflow: hidden;
}

/* 功能介绍弹窗 */
.feature-list {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  max-height: 500px;
  overflow-y: auto;
}

.feature-item {
  display: flex;
  gap: 12px;
  padding: 12px;
  background: var(--bg-color);
  border-radius: 8px;
  transition: all 0.2s;
}

.feature-item:hover {
  background: var(--border-color);
}

.feature-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  background: var(--primary-color);
  border-radius: 8px;
  flex-shrink: 0;
  font-size: 20px;
  color: #fff;
}

.feature-content {
  flex: 1;
  min-width: 0;
}

.feature-content h4 {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 4px 0;
}

.feature-content p {
  font-size: 12px;
  color: var(--text-secondary);
  margin: 0;
  line-height: 1.4;
}

/* 头部按钮组 */
.header-actions {
  display: flex;
  gap: 8px;
}

/* 版本更新弹窗 */
.version-timeline {
  max-height: 500px;
  overflow-y: auto;
}

.version-item {
  padding: 16px;
  background: var(--bg-color);
  border-radius: 8px;
  margin-bottom: 12px;
}

.version-item:last-child {
  margin-bottom: 0;
}

.version-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.version-tag {
  display: inline-block;
  padding: 4px 12px;
  background: var(--primary-color);
  color: #fff;
  border-radius: 12px;
  font-size: 14px;
  font-weight: 600;
}

.version-tag.latest {
  background: linear-gradient(135deg, var(--primary-color) 0%, #722ed1 100%);
}

.version-date {
  font-size: 13px;
  color: var(--text-secondary);
}

.version-features {
  margin: 0;
  padding-left: 20px;
}

.version-features li {
  font-size: 13px;
  color: var(--text-primary);
  line-height: 1.8;
}

/* 错误日志样式 */
.error-log-content {
  max-height: 500px;
  overflow-y: auto;
}

.error-log-actions {
  display: flex;
  justify-content: flex-end;
  margin-bottom: 16px;
}

.clear-logs-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: #ff4d4f;
  color: #fff;
  border: none;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
  transition: background 0.2s;
}

.clear-logs-btn:hover {
  background: #ff7875;
}

.no-error-logs {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 0;
  color: var(--text-secondary);
}

.no-error-icon {
  font-size: 48px;
  color: #52c41a;
  margin-bottom: 12px;
}

.error-log-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.error-log-item {
  padding: 12px;
  background: var(--bg-color);
  border-radius: 6px;
  border-left: 3px solid #ff4d4f;
}

.error-log-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.error-log-source {
  font-size: 12px;
  font-weight: 600;
  color: var(--primary-color);
}

.error-log-time {
  font-size: 12px;
  color: var(--text-secondary);
}

.error-log-message {
  font-size: 13px;
  color: var(--text-primary);
  word-break: break-all;
  line-height: 1.5;
}
</style>
