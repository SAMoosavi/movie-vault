import { filter_medias } from '../functions/invoker'
import type { Media } from '../type'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { useFiltersStore } from './Filters'

export const useVideosStore = defineStore('videos', () => {
  const filtersStore = useFiltersStore()
  const videos = ref<Media[]>([])

  async function reload() {
    videos.value = await filter_medias(filtersStore.filters)
  }

  const count = computed(() => videos.value.length)

  return {
    videos,
    reload,
    count,
  }
})
