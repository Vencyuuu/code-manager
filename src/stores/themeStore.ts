import { reactive } from 'vue'

// 全局主题配置
const themeConfig = reactive({
  primaryColor: '#1890ff',
  isDarkMode: false
})

// 从 localStorage 加载主题配置
export const loadThemeConfig = () => {
  const saved = localStorage.getItem('themeConfig')
  if (saved) {
    try {
      const parsed = JSON.parse(saved)
      themeConfig.primaryColor = parsed.primaryColor || '#1890ff'
      themeConfig.isDarkMode = parsed.isDarkMode || false
    } catch (e) {
      console.error('Failed to parse theme config:', e)
    }
  }
  applyTheme()
}

// 保存主题配置到 localStorage
export const saveThemeConfig = () => {
  localStorage.setItem('themeConfig', JSON.stringify({
    primaryColor: themeConfig.primaryColor,
    isDarkMode: themeConfig.isDarkMode
  }))
  applyTheme()
}

// 应用主题到 CSS 变量
export const applyTheme = () => {
  const root = document.documentElement

  // 设置主色调
  root.style.setProperty('--primary-color', themeConfig.primaryColor)
  root.style.setProperty('--primary-hover', adjustColor(themeConfig.primaryColor, -10))

  // 夜间模式
  if (themeConfig.isDarkMode) {
    root.setAttribute('data-theme', 'dark')
  } else {
    root.setAttribute('data-theme', 'light')
  }

  // 更新 Ant Design Vue 主题
  updateAntdTheme()
}

// 更新 Ant Design Vue 主题
const updateAntdTheme = () => {
  // 尝试获取 Ant Design Vue 的配置并更新
  const style = document.createElement('style')
  style.textContent = `
    .ant-btn-primary {
      --ant-primary-color: ${themeConfig.primaryColor} !important;
      background-color: ${themeConfig.primaryColor} !important;
      border-color: ${themeConfig.primaryColor} !important;
    }
    .ant-btn-primary:hover, .ant-btn-primary:focus {
      background-color: ${adjustColor(themeConfig.primaryColor, 10)} !important;
      border-color: ${adjustColor(themeConfig.primaryColor, 10)} !important;
    }
    .ant-switch-checked {
      background-color: ${themeConfig.primaryColor} !important;
    }
    .ant-input:focus, .ant-input:hover, .ant-input-focused {
      border-color: ${themeConfig.primaryColor} !important;
      box-shadow: 0 0 0 2px ${themeConfig.primaryColor}20 !important;
    }
    .ant-select-focused .ant-select-selector {
      border-color: ${themeConfig.primaryColor} !important;
      box-shadow: 0 0 0 2px ${themeConfig.primaryColor}20 !important;
    }
    .ant-checkbox-checked .ant-checkbox-inner {
      background-color: ${themeConfig.primaryColor} !important;
      border-color: ${themeConfig.primaryColor} !important;
    }
    .ant-dropdown-menu-item:hover {
      background-color: ${themeConfig.primaryColor}10 !important;
    }
  `

  // 移除旧的样式
  const oldStyle = document.getElementById('antd-theme-style')
  if (oldStyle) {
    oldStyle.remove()
  }

  style.id = 'antd-theme-style'
  document.head.appendChild(style)
}

// 调整颜色亮度
const adjustColor = (color: string, amount: number): string => {
  const hex = color.replace('#', '')
  const r = Math.max(0, Math.min(255, parseInt(hex.substr(0, 2), 16) + amount))
  const g = Math.max(0, Math.min(255, parseInt(hex.substr(2, 2), 16) + amount))
  const b = Math.max(0, Math.min(255, parseInt(hex.substr(4, 2), 16) + amount))
  return `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`
}

// 设置主色调
export const setPrimaryColor = (color: string) => {
  themeConfig.primaryColor = color
  saveThemeConfig()
}

// 设置夜间模式
export const setDarkMode = (isDark: boolean) => {
  themeConfig.isDarkMode = isDark
  saveThemeConfig()
}

export const useTheme = () => {
  return {
    themeConfig,
    loadThemeConfig,
    saveThemeConfig,
    setPrimaryColor,
    setDarkMode
  }
}
