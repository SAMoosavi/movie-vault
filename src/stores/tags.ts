import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import type { Tag } from '@/type'
import { get_tags } from '@/functions/invoker'
import { info } from '@tauri-apps/plugin-log'
import { logFrontendError } from '@/functions/errorHandling'

export const useTagsStore = defineStore('tags', () => {
  const tags = ref<Tag[]>([])

  async function reload() {
    try {
      tags.value = await get_tags()
      await info(`Tag list reloaded successfully with ${tags.value.length} tags.`)
    } catch (error) {
      logFrontendError('store.tags.reload', error)
      throw error
    }
  }

  const count = computed(() => tags.value.length)

  return {
    tags,
    count,
    reload,
  }
})
