import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { FilterValues } from '../type'

export const useFiltersStore = defineStore('filters', () => {
  // Initial state constant
  const defaultFilters: FilterValues = {
    type: 'all',
    minRating: 0,
    country: 0,
    genre: 0,
    name: '',
  }

  const filters = ref<FilterValues>({ ...defaultFilters })

  // Reset to defaults
  function resetFilters() {
    filters.value = { ...defaultFilters }
  }

  return { filters, resetFilters }
})
