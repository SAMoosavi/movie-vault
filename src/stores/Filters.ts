import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { FilterValues } from '../type'

export const useFiltersStore = defineStore('filters', () => {
  const defaultFilters: FilterValues = {
    type: 'all',
    minRating: null,
    country: null,
    genre: null,
    name: '',
    existImdb: null,
    existMultiFile: null,
    actor: null,
    showed: null
  }

  const filters = ref<FilterValues>({ ...defaultFilters })

  // Reset to defaults
  function resetFilters() {
    filters.value = { ...defaultFilters }
  }

  return { filters, resetFilters }
})
