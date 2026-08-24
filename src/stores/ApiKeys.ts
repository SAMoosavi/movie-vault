import { defineStore } from 'pinia'
import { ref } from 'vue'

export const DEFAULT_API_KEYS = ['e8f12113', '1e97e442']

export const useApiKeysStore = defineStore(
  'apiKeys',
  () => {
    const apiKeys = ref<string[]>([...DEFAULT_API_KEYS])

    function addKey(key: string): boolean {
      const trimmed = key.trim()
      if (!trimmed) return false
      if (apiKeys.value.includes(trimmed)) return false
      apiKeys.value.push(trimmed)
      return true
    }

    function removeKey(key: string): boolean {
      const index = apiKeys.value.indexOf(key)
      if (index === -1) return false
      apiKeys.value.splice(index, 1)
      return true
    }

    return { apiKeys, addKey, removeKey }
  },
  {
    persist: {
      storage: localStorage,
      pick: ['apiKeys'],
    },
  },
)
