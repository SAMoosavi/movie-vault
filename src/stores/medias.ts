import { filter_medias } from '@/functions/invoker'
import type { Media } from '../type'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { useFiltersStore } from './Filters'
import { info, error } from '@tauri-apps/plugin-log'

export const useMediasStore = defineStore('medias', () => {
  const filtersStore = useFiltersStore()
  const medias = ref<Media[]>([])
  const page = ref(0)

  async function get_next_page() {
    try {
      page.value++
      const newMedias = await get_data()
      medias.value = medias.value.concat(newMedias)
      await info(`Loaded page ${page.value} with ${newMedias.length} new media items.`)
    } catch (err) {
      await error(`Failed to load next page: ${err}`)
    }
  }

  async function get_data() {
    try {
      return await filter_medias(filtersStore.filters, page.value)
    } catch (err) {
      await error(`Failed to get data: ${err}`)
      throw err
    }
  }

  async function reload() {
    try {
      page.value = 0
      medias.value = await get_data()
      await info('Media list reloaded successfully.')
    } catch (err) {
      await error(`Failed to reload media list: ${err}`)
    }
  }

  const count = computed(() => medias.value.length)

  return {
    medias,
    reload,
    count,
    get_next_page,
  }
})
