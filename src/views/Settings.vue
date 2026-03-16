<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { Input, Switch, Modal, message } from 'ant-design-vue'
import { Icon } from '@iconify/vue'
import { useTheme } from '../stores/themeStore'

const { themeConfig, setPrimaryColor, setDarkMode } = useTheme()

// 配置项
const distOutputPath = ref('')
const autoRefreshEnabled = ref(false)
const autoRefreshInterval = ref(60)

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
    description: '展示所有已添加的项目，支持拖拽添加新项目，可搜索项目名称。',
  },
  {
    title: 'IDE 打开',
    icon: 'mdi:microsoft-visual-studio',
    description: '支持多种 IDE 打开项目：VSCode、Cursor、WebStorm、Trae。',
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
    description: '可输入自定义命令执行，如 npm install、yarn add 等。',
  },
  {
    title: '产物打包',
    icon: 'mdi:folder-zip',
    description: '一键复制 dist 目录并打包成 zip 文件，支持自定义命名规则。',
  },
  {
    title: '项目分组',
    icon: 'mdi:folder-group',
    description: '支持创建分组并绑定项目，方便按分类管理项目。',
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
  {
    title: '多 IDE 支持',
    icon: 'mdi:application-cog',
    description: '支持配置不同的 IDE 打开方式，满足不同开发习惯。',
  },
]

// 版本更新数据
const versionHistory = [
  {
    version: '1.0.0',
    date: '2026-03-16',
    isLatest: true,
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

// 原始配置数据（用于比较是否有变更）
const originalConfig = ref({
  distOutputPath: '',
  autoRefreshEnabled: false,
  autoRefreshInterval: 60,
  primaryColor: '#1890ff',
  isDarkMode: false
})

// 判断配置是否有变更
const hasChanges = computed(() => {
  return distOutputPath.value !== originalConfig.value.distOutputPath ||
    autoRefreshEnabled.value !== originalConfig.value.autoRefreshEnabled ||
    autoRefreshInterval.value !== originalConfig.value.autoRefreshInterval ||
    primaryColor.value !== originalConfig.value.primaryColor ||
    isDarkMode.value !== originalConfig.value.isDarkMode
})

// 定时器
let refreshTimer: number | null = null

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
    // 保存原始配置
    originalConfig.value = {
      distOutputPath: distOutputPath.value,
      autoRefreshEnabled: autoRefreshEnabled.value,
      autoRefreshInterval: autoRefreshInterval.value,
      primaryColor: primaryColor.value,
      isDarkMode: isDarkMode.value
    }
  } catch (err) {
    console.error('Failed to load config:', err)
  }
}

// 保存配置
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
    message.success('配置已保存')
    // 更新原始配置
    originalConfig.value = {
      distOutputPath: distOutputPath.value,
      autoRefreshEnabled: autoRefreshEnabled.value,
      autoRefreshInterval: autoRefreshInterval.value,
      primaryColor: primaryColor.value,
      isDarkMode: isDarkMode.value
    }
    // 更新定时器
    setupAutoRefresh()
  } catch (err) {
    console.error('Failed to save config:', err)
    message.error('保存失败: ' + err)
  }
}

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
        await invoke('refresh_all_projects')
        // 触发自定义事件通知项目列表刷新
        window.dispatchEvent(new CustomEvent('projects-auto-refresh'))
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
      <h3>关于</h3>
      <div class="about-info">
        <div class="about-logo">
          <Icon icon="mdi:code-tags" />
        </div>
        <div class="about-content">
          <h4>代码管理器</h4>
          <p class="version">版本 1.0.0</p>
          <p class="description">一款专为开发者设计的项目管理工具，支持多 IDE 打开、Git 分支管理、NPM 脚本运行等功能。</p>
        </div>
      </div>
    </div>

    <div class="settings-actions">
      <button class="save-btn" @click="saveConfig" :disabled="!hasChanges">
        <Icon icon="mdi:content-save" />
        保存配置
      </button>
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

.settings-actions {
  display: flex;
  justify-content: flex-end;
}

.save-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 24px;
  background: var(--primary-color);
  color: #fff;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  cursor: pointer;
}

.save-btn:hover:not(:disabled) {
  background: var(--primary-hover);
}

.save-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
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
</style>
