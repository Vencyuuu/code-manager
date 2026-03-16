<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useProjectStore } from './stores/projectStore'
import { useTheme } from './stores/themeStore'
import { Icon } from '@iconify/vue'

const router = useRouter()
const route = useRoute()
const { loadProjects, refreshAllProjects } = useProjectStore()
const { loadThemeConfig } = useTheme()

const APP_VERSION = 'v1.0.0'

const navigate = (path: string) => {
  router.push(path)
}

onMounted(async () => {
  // 加载主题配置
  loadThemeConfig()
  // 应用启动时加载项目数据并刷新
  await loadProjects()
  await refreshAllProjects()
})
</script>

<template>
  <div class="app-container" :class="{ 'dark-mode': false }">
    <aside class="sidebar">
      <div class="logo">
        <Icon icon="mdi:code-tags" class="logo-icon" />
        <div class="logo-text">
          <h2>代码管理器</h2>
          <span class="version">{{ APP_VERSION }}</span>
        </div>
      </div>
      <nav class="nav-menu">
        <a
          href="#"
          class="nav-item"
          :class="{ active: route.path === '/' }"
          @click.prevent="navigate('/')"
        >
          项目列表
        </a>
        <a
          href="#"
          class="nav-item"
          :class="{ active: route.path === '/settings' }"
          @click.prevent="navigate('/settings')"
        >
          设置
        </a>
      </nav>
    </aside>
    <main class="main-content">
      <header class="header">
        <h1>{{ route.path === '/' ? '项目列表' : '设置' }}</h1>
      </header>
      <div class="content">
        <router-view />
      </div>
    </main>
  </div>
</template>

<style>
/* 全局 CSS 变量 */
:root {
  --primary-color: #1890ff;
  --primary-hover: #40a9ff;
  --primary-color-hover: #40a9ff;
  --primary-color-focus: rgba(24, 144, 255, 0.2);
  --bg-color: #f5f5f5;
  --bg-card: #ffffff;
  --text-primary: #333333;
  --text-secondary: #666666;
  --border-color: #e0e0e0;
  --sidebar-bg: #1e1e1e;
  --sidebar-text: #ffffff;
  --sidebar-hover: #2d2d2d;
}

/* 夜间模式 */
[data-theme="dark"] {
  --primary-color: #1890ff;
  --primary-hover: #40a9ff;
  --primary-color-hover: #40a9ff;
  --primary-color-focus: rgba(24, 144, 255, 0.3);
  --bg-color: #141414;
  --bg-card: #1f1f1f;
  --text-primary: #ffffff;
  --text-secondary: #aaa;
  --border-color: #303030;
  --sidebar-bg: #000000;
  --sidebar-text: #ffffff;
  --sidebar-hover: #1a1a1a;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  background: var(--bg-color);
  color: var(--text-primary);
}

/* Ant Design Vue Modal 暗色模式适配 */
html[data-theme="dark"] .ant-modal-content {
  background: var(--bg-card) !important;
  border: 1px solid var(--border-color);
}

html[data-theme="dark"] .ant-modal-header {
  background: var(--bg-card) !important;
  border-bottom: 1px solid var(--border-color);
}

html[data-theme="dark"] .ant-modal-title {
  color: var(--text-primary) !important;
}

html[data-theme="dark"] .ant-modal-close {
  color: var(--text-secondary) !important;
}

html[data-theme="dark"] .ant-modal-close:hover {
  color: var(--text-primary) !important;
}

html[data-theme="dark"] .ant-modal-body {
  background: var(--bg-card) !important;
  color: var(--text-primary) !important;
}

html[data-theme="dark"] .ant-modal-footer {
  background: var(--bg-card) !important;
  border-top: 1px solid var(--border-color);
}

html[data-theme="dark"] .ant-modal-mask {
  background: rgba(0, 0, 0, 0.45) !important;
}

html[data-theme="dark"] .ant-modal-confirm-body .ant-modal-confirm-title {
  color: var(--text-primary) !important;
}

html[data-theme="dark"] .ant-modal-confirm-body .ant-modal-confirm-content {
  color: var(--text-primary) !important;
}

html[data-theme="dark"] .ant-modal-confirm .ant-modal-confirm-btns {
  background: var(--bg-card) !important;
  border-top: 1px solid var(--border-color);
}

html[data-theme="dark"] .ant-modal-confirm-btns .ant-btn {
  background: var(--bg-card) !important;
  border-color: var(--border-color) !important;
  color: var(--text-primary) !important;
}

html[data-theme="dark"] .ant-modal-confirm-btns .ant-btn-primary {
  background: var(--primary-color) !important;
  border-color: var(--primary-color) !important;
  color: #fff !important;
}

html[data-theme="dark"] .ant-modal-wrap {
  background: rgba(0, 0, 0, 0.45);
}

/* Ant Design Vue 其他组件暗色模式 */
html[data-theme="dark"] .ant-select-dropdown {
  background: var(--bg-card) !important;
  border: 1px solid var(--border-color);
}

html[data-theme="dark"] .ant-select-item {
  color: var(--text-primary) !important;
}

html[data-theme="dark"] .ant-select-item:hover {
  background: var(--bg-color) !important;
}

html[data-theme="dark"] .ant-select-item-option-selected {
  background: var(--primary-color) !important;
}

html[data-theme="dark"] .ant-dropdown-menu {
  background: var(--bg-card) !important;
  border: 1px solid var(--border-color);
}

html[data-theme="dark"] .ant-dropdown-menu-item {
  color: var(--text-primary) !important;
}

/* Input 暗色模式 - 使用更强选择器 */
html[data-theme="dark"] .ant-input,
html[data-theme="dark"] .ant-input-affix-wrapper,
html[data-theme="dark"] .ant-input-affix-wrapper input,
html[data-theme="dark"] .ant-input-search .ant-input {
  background: var(--bg-card) !important;
  border-color: var(--border-color) !important;
  color: var(--text-primary) !important;
}

html[data-theme="dark"] .ant-input:hover,
html[data-theme="dark"] .ant-input-affix-wrapper:hover,
html[data-theme="dark"] .ant-input-search .ant-input:hover {
  border-color: var(--primary-color) !important;
}

html[data-theme="dark"] .ant-input:focus,
html[data-theme="dark"] .ant-input-affix-wrapper:focus,
html[data-theme="dark"] .ant-input-affix-wrapper-focused,
html[data-theme="dark"] .ant-input-search .ant-input:focus {
  border-color: var(--primary-color) !important;
  box-shadow: 0 0 0 2px var(--primary-color-focus) !important;
}

/* 亮色模式输入框 focus 效果也使用主题色 */
.ant-input:focus,
.ant-input-affix-wrapper:focus,
.ant-input-affix-wrapper-focused,
.ant-input-search .ant-input:focus {
  border-color: var(--primary-color) !important;
  box-shadow: 0 0 0 2px var(--primary-color-focus) !important;
}

html[data-theme="dark"] .ant-input::placeholder {
  color: var(--text-secondary) !important;
}

html[data-theme="dark"] .ant-input-prefix,
html[data-theme="dark"] .ant-input-suffix,
html[data-theme="dark"] .ant-input-clear-icon,
html[data-theme="dark"] .ant-input-search .ant-input-search-icon {
  color: var(--text-secondary) !important;
}

/* Switch 暗色模式 */
html[data-theme="dark"] .ant-switch {
  background: rgba(255, 255, 255, 0.3) !important;
}

html[data-theme="dark"] .ant-switch-checked {
  background: var(--primary-color) !important;
}

/* Message 暗色模式 */
html[data-theme="dark"] .ant-message-notice-content {
  background: var(--bg-card) !important;
  color: var(--text-primary) !important;
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.12);
}

/* Checkbox 暗色模式 */
html[data-theme="dark"] .ant-checkbox-wrapper {
  color: var(--text-primary) !important;
}

html[data-theme="dark"] .ant-checkbox-inner {
  background: var(--bg-card) !important;
  border-color: var(--border-color) !important;
}

html[data-theme="dark"] .ant-checkbox-checked .ant-checkbox-inner {
  background: var(--primary-color) !important;
  border-color: var(--primary-color) !important;
}

/* Dropdown 暗色模式 */
html[data-theme="dark"] .ant-dropdown-menu {
  background: var(--bg-card) !important;
  border: 1px solid var(--border-color);
}

html[data-theme="dark"] .ant-dropdown-menu-item {
  color: var(--text-primary) !important;
}

html[data-theme="dark"] .ant-dropdown-menu-item:hover {
  background: var(--bg-color) !important;
}

html[data-theme="dark"] .ant-dropdown-menu-item-divider {
  background: var(--border-color) !important;
}

/* Select 暗色模式 */
html[data-theme="dark"] .ant-select:not(.ant-select-disabled) .ant-select-selector {
  background: var(--bg-card) !important;
  border-color: var(--border-color) !important;
  color: var(--text-primary) !important;
}

html[data-theme="dark"] .ant-select:not(.ant-select-disabled):hover .ant-select-selector {
  border-color: var(--primary-color) !important;
}

html[data-theme="dark"] .ant-select-focused:not(.ant-select-disabled) .ant-select-selector {
  border-color: var(--primary-color) !important;
  box-shadow: 0 0 0 2px rgba(24, 144, 255, 0.2) !important;
}

/* 原生表单元素暗色模式 */
html[data-theme="dark"] select {
  background: var(--bg-card) !important;
  border-color: var(--border-color) !important;
  color: var(--text-primary) !important;
}

/* Modal 内部通用元素暗色模式 */
html[data-theme="dark"] .ant-modal-body label {
  color: var(--text-primary) !important;
}

html[data-theme="dark"] .ant-modal-body p {
  color: var(--text-primary) !important;
}

html[data-theme="dark"] .ant-modal-body strong {
  color: var(--text-primary) !important;
}

html[data-theme="dark"] .ant-modal-body .ant-btn {
  background: var(--bg-card) !important;
  border-color: var(--border-color) !important;
  color: var(--text-primary) !important;
}

html[data-theme="dark"] .ant-modal-body .ant-btn:hover {
  border-color: var(--primary-color) !important;
  color: var(--primary-color) !important;
}

html[data-theme="dark"] .ant-modal-body .ant-btn-primary {
  background: var(--primary-color) !important;
  border-color: var(--primary-color) !important;
  color: #fff !important;
}

html[data-theme="dark"] .ant-modal-body .ant-btn-primary:hover {
  background: var(--primary-hover) !important;
  border-color: var(--primary-hover) !important;
}
</style>

<style scoped>
.app-container {
  display: flex;
  height: 100vh;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  background: var(--bg-color);
}

.sidebar {
  width: 220px;
  background: linear-gradient(135deg, var(--primary-color) 0%, var(--sidebar-bg) 100%);
  color: #fff;
  display: flex;
  flex-direction: column;
}

[data-theme="dark"] .sidebar {
  background: var(--sidebar-bg);
}

.logo {
  padding: 20px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.2);
  display: flex;
  align-items: center;
  gap: 10px;
}

[data-theme="dark"] .logo {
  border-bottom-color: rgba(255, 255, 255, 0.1);
}

.logo-icon {
  font-size: 28px;
  color: #fff;
}

.logo-text {
  display: flex;
  flex-direction: column;
}

.logo h2 {
  font-size: 16px;
  font-weight: 600;
  line-height: 1.2;
}

.logo .version {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.7);
  margin-top: 2px;
}

.nav-menu {
  padding: 10px 0;
}

.nav-item {
  display: block;
  padding: 12px 20px;
  color: rgba(255, 255, 255, 0.8);
  text-decoration: none;
  transition: all 0.2s;
}

.nav-item:hover,
.nav-item.active {
  background: rgba(255, 255, 255, 0.15);
  color: #fff;
}

.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--bg-color);
}

.header {
  padding: 20px 30px;
  background: var(--bg-card);
  border-bottom: 1px solid var(--border-color);
}

.header h1 {
  font-size: 20px;
  font-weight: 600;
  color: var(--text-primary);
}

.content {
  flex: 1;
  padding: 30px;
  overflow-y: auto;
}
</style>
