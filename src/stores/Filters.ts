import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { FilterValues } from '../type'

export const useFiltersStore = defineStore('filters', () => {
  const defaultFilters: FilterValues = {
    name: '',
    type: 'all',
    country: [],
    genre: [],
    actor: [],
    minRating: null,
    existImdb: null,
    existMultiFile: null,
    watched: null,
    sortBy: 'name',
    sortDirection: 'asc',
  }

  const filters = ref<FilterValues>({ ...defaultFilters })

  // Reset to defaults
  function resetFilters() {
    filters.value = { ...defaultFilters }
  }

  return { filters, resetFilters }
})
