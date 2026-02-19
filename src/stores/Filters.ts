import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { FilterValues } from '@/type'
import { info } from '@tauri-apps/plugin-log'

const defaultFilters: FilterValues = {
  name: '',
  type: 'all',
  country: [],
  genre: [],
  people: [],
  minRating: 0,
  existImdb: null,
  existMultiFile: null,
  watched: null,
  sortBy: 'name',
  sortDirection: 'asc',
  watchList: null,
  tags: [],
}

export const useFiltersStore = defineStore('filters', () => {
  const filters = ref<FilterValues>(structuredClone(defaultFilters))

  function resetFilters() {
    info('Resetting filters to default values.')
    filters.value = structuredClone(defaultFilters)
    info('Filters have been reset successfully.')
  }

  return { filters, resetFilters }
})
