import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { FilterValues } from '../type'

const defaultFilters: FilterValues = {
  name: '',
  type: 'all',
  country: [],
  genre: [],
  actor: [],
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
    filters.value = structuredClone(defaultFilters)
  }

  return { filters, resetFilters }
})
