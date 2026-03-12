import { reactive } from 'vue'

export interface ScriptState {
  projectId: string
  output: string
  isRunning: boolean
  scriptName: string
}

// 全局脚本状态
const state = reactive({
  scripts: {} as Record<string, ScriptState>
})

// 添加脚本状态
export const addScriptState = (projectId: string, scriptName: string) => {
  state.scripts[projectId] = {
    projectId,
    output: '',
    isRunning: true,
    scriptName
  }
}

// 更新脚本输出
export const updateScriptOutput = (projectId: string, output: string) => {
  if (state.scripts[projectId]) {
    state.scripts[projectId].output += output + '\n'
  }
}

// 标记脚本完成
export const markScriptCompleted = (projectId: string) => {
  if (state.scripts[projectId]) {
    state.scripts[projectId].isRunning = false
  }
}

// 获取脚本状态
export const getScriptState = (projectId: string): ScriptState | undefined => {
  return state.scripts[projectId]
}

// 移除脚本状态
export const removeScriptState = (projectId: string) => {
  delete state.scripts[projectId]
}

export const useScriptStore = () => {
  return {
    state,
    addScriptState,
    updateScriptOutput,
    markScriptCompleted,
    getScriptState,
    removeScriptState
  }
}