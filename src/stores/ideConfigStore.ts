import { ref } from 'vue'

export interface IdeConfig {
  id: string
  name: string
  path: string
  enabled: boolean
}

const ideConfigs = ref<IdeConfig[]>([])

export function useIdeConfigStore() {
  return {
    ideConfigs
  }
}
