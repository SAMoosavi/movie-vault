import { filter_medias } from '../functions/invoker'
import type { Media } from '../type'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { useFiltersStore } from './Filters'

export const useMediasStore = defineStore('medias', () => {
  const filtersStore = useFiltersStore()
  const medias = ref<Media[]>([])

  async function reload() {
    medias.value = await filter_medias(filtersStore.filters)
  }

  const count = computed(() => medias.value.length)

  return {
    medias,
    reload,
    count,
  }
})
