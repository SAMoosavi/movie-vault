import { filter_medias } from '../functions/invoker'
import type { Media } from '../type'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { useFiltersStore } from './Filters'

export const useMediasStore = defineStore('medias', () => {
  const filtersStore = useFiltersStore()
  const medias = ref<Media[]>([])
  const page = ref(0)

  async function get_next_page() {
    page.value++
    medias.value = medias.value.concat(await get_data())
  }

  async function get_data() {
    return await filter_medias(filtersStore.filters, page.value)
  }


  async function reload() {
    page.value = 0
    medias.value = await get_data()
  }

  const count = computed(() => medias.value.length)

  return {
    medias,
    reload,
    count,
    get_next_page
  }
})
