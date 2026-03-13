<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { Input, Switch, message } from 'ant-design-vue'
import { Icon } from '@iconify/vue'

// 配置项
const distOutputPath = ref('')
const autoRefreshEnabled = ref(false)
const autoRefreshInterval = ref(60)

// 定时器
let refreshTimer: number | null = null

// 加载配置
const loadConfig = async () => {
  try {
    const config = await invoke<{ distOutputPath: string; autoRefreshEnabled: boolean; autoRefreshInterval: number }>('get_config')
    distOutputPath.value = config.distOutputPath || ''
    autoRefreshEnabled.value = config.autoRefreshEnabled || false
    autoRefreshInterval.value = config.autoRefreshInterval || 60
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
    message.success('配置已保存')
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
    <h2 class="page-title">设置</h2>

    <div class="settings-section">
      <h3>产物输出</h3>
      <div class="setting-item">
        <span class="setting-label">压缩包默认输出路径:</span>
        <div class="setting-input-group">
          <Input
            v-model:value="distOutputPath"
            placeholder="选择输出目录"
            class="path-input"
            readonly
          />
          <button class="browse-btn" @click="selectOutputPath">
            <Icon icon="mdi:folder-open" />
            浏览
          </button>
        </div>
        <p class="setting-hint">复制 dist 并打包时，压缩包将保存到此目录，默认为项目目录下的 output 文件夹</p>
      </div>
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

    <div class="settings-actions">
      <button class="save-btn" @click="saveConfig">
        <Icon icon="mdi:content-save" />
        保存配置
      </button>
    </div>
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
  margin-bottom: 24px;
  color: #333;
}

.settings-section {
  background: #fff;
  border-radius: 8px;
  padding: 20px;
  margin-bottom: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
}

.settings-section h3 {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 16px;
  color: #333;
}

.setting-item {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}

.setting-label {
  font-size: 14px;
  color: #666;
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
  background: #1890ff;
  color: #fff;
  border: none;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
  white-space: nowrap;
}

.browse-btn:hover {
  background: #40a9ff;
}

.interval-input {
  width: 100px;
}

.setting-hint {
  font-size: 12px;
  color: #999;
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
  background: #52c41a;
  color: #fff;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  cursor: pointer;
}

.save-btn:hover {
  background: #73d13d;
}
</style>