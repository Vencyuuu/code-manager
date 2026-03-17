import { ref } from 'vue'

// 错误日志类型
export interface ErrorLog {
  id: string
  message: string
  timestamp: number
  source: string
}

const ERROR_LOG_KEY = 'app_error_logs'
const MAX_LOG_AGE = 24 * 60 * 60 * 1000 // 1天 = 24 * 60 * 60 * 1000 毫秒

// 获取所有错误日志
export function getErrorLogs(): ErrorLog[] {
  try {
    const logs = localStorage.getItem(ERROR_LOG_KEY)
    if (!logs) return []

    const parsed: ErrorLog[] = JSON.parse(logs)
    const now = Date.now()

    // 过滤掉超过1天的日志
    const validLogs = parsed.filter(log => now - log.timestamp < MAX_LOG_AGE)

    // 如果有过期日志，更新存储
    if (validLogs.length !== parsed.length) {
      localStorage.setItem(ERROR_LOG_KEY, JSON.stringify(validLogs))
    }

    return validLogs.sort((a, b) => b.timestamp - a.timestamp) // 按时间倒序
  } catch {
    return []
  }
}

// 添加错误日志
export function addErrorLog(message: string, source: string): void {
  try {
    const logs = getErrorLogs()

    const newLog: ErrorLog = {
      id: `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      message: message.slice(0, 200), // 限制长度，最多200字符
      timestamp: Date.now(),
      source
    }

    logs.unshift(newLog)

    // 保留最多100条日志
    const trimmedLogs = logs.slice(0, 100)
    localStorage.setItem(ERROR_LOG_KEY, JSON.stringify(trimmedLogs))
  } catch (e) {
    console.error('Failed to add error log:', e)
  }
}

// 清除所有错误日志
export function clearErrorLogs(): void {
  localStorage.removeItem(ERROR_LOG_KEY)
}

// 创建响应式的错误日志
const errorLogs = ref<ErrorLog[]>(getErrorLogs())

// 刷新错误日志
export function refreshErrorLogs(): void {
  errorLogs.value = getErrorLogs()
}

// 监听storage变化
if (typeof window !== 'undefined') {
  window.addEventListener('storage', (e) => {
    if (e.key === ERROR_LOG_KEY) {
      refreshErrorLogs()
    }
  })
}

export function useErrorLogStore() {
  return {
    errorLogs,
    refreshErrorLogs,
    clearErrorLogs,
    addErrorLog
  }
}
