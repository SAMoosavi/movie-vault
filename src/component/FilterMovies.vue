<template>
  <!-- Card Container -->
  <div class="card from-primary/50 to-secondary/50 mb-8 bg-gradient-to-br p-0.5">
    <div class="card bg-base-200 p-6">
      <!-- Header Section -->
      <div class="mb-6 flex items-center justify-between">
        <div class="flex items-center">
          <Filter class="text-primary mr-2 h-5 w-5" />
          <h2 class="text-base-content text-lg font-semibold">Advanced Filters</h2>
        </div>
        <!-- Reset Button -->
        <button type="reset" class="btn btn-ghost btn-sm" @click="filtersStore.resetFilters()">
          <RefreshCcw class="mr-1 h-4 w-4" />
          Reset Filters
        </button>
      </div>

      <!-- Filters Grid -->
      <div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-5">
        <!-- Movie Name Filter -->
        <div class="form-control">
          <label class="label">
            <span class="label-text">Movie name</span>
          </label>
          <label class="input input-bordered w-full max-w-xs">
            <Search class="h-4" />
            <input v-model="filters.name" type="search" placeholder="Search movie name..." />
          </label>
        </div>

        <!-- Type Filter -->
        <div class="form-control">
          <label class="label">
            <span class="label-text">Type</span>
          </label>
          <select class="select select-bordered" v-model="filters.type">
            <option value="all">All Types</option>
            <option value="movie">Movies</option>
            <option value="series">Series</option>
          </select>
        </div>

        <!-- Minimum Rating Filter -->
        <div class="form-control">
          <label class="label">
            <span class="label-text">Min Rating</span>
          </label>
          <select class="select select-bordered w-full pr-8" v-model="filters.minRating">
            <option :value="null">Any Rating</option>
            <option v-for="i in 9" :key="i" :value="i">{{ i }}+ Stars</option>
          </select>
        </div>

        <!-- Country Filter -->
        <div class="form-control">
          <label class="label">
            <span class="label-text">Country</span>
          </label>
          <AutocompleteSelect v-model="filters.country" :items="countries" />
        </div>

        <!-- Genre Filter -->
        <div class="form-control">
          <label class="label">
            <span class="label-text">Genre</span>
          </label>
          <AutocompleteSelect v-model="filters.genre" :items="genres" />
        </div>

        <!-- Actor Filter -->
        <div class="form-control">
          <label class="label"><span class="label-text">Actor</span></label>
          <AutocompleteSelect v-model="filters.actor" :items="actors" />
        </div>

        <!-- Tag Filter -->
        <div class="form-control">
          <label class="label"><span class="label-text">Tag</span></label>
          <AutocompleteSelect v-model="filters.tags" :items="tags" />
        </div>

        <!-- Has IMDb Filter -->
        <div class="form-control">
          <label class="label"><span class="label-text">Has IMDb</span></label>
          <select v-model="filters.existImdb" class="select select-bordered w-full pr-8">
            <option v-for="opt in boolOptions" :key="opt.label" :value="opt.value">
              {{ opt.label }}
            </option>
          </select>
        </div>

        <!-- Multiple Files Filter -->
        <div class="form-control">
          <label class="label"><span class="label-text">Multiple Files</span></label>
          <select v-model="filters.existMultiFile" class="select select-bordered w-full pr-8">
            <option v-for="opt in boolOptions" :key="opt.label" :value="opt.value">
              {{ opt.label }}
            </option>
          </select>
        </div>

        <!-- Watched Filter -->
        <div class="form-control">
          <label class="label"><span class="label-text">Watched</span></label>
          <select v-model="filters.watched" class="select select-bordered w-full pr-8">
            <option v-for="opt in boolOptions" :key="opt.label" :value="opt.value">
              {{ opt.label }}
            </option>
          </select>
        </div>

        <!-- Watch List Filter -->
        <div class="form-control">
          <label class="label"><span class="label-text">Watch List</span></label>
          <select v-model="filters.watchList" class="select select-bordered w-full pr-8">
            <option v-for="opt in boolOptions" :key="opt.label" :value="opt.value">
              {{ opt.label }}
            </option>
          </select>
        </div>

        <!-- Sort By Filter -->
        <div class="form-control">
          <label class="label">
            <span class="label-text">Sort By</span>
          </label>
          <select class="select select-bordered" v-model="filters.sortBy">
            <option v-for="opt in sortByOptions" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </option>
          </select>
        </div>

        <!-- Sort Direction Filter -->
        <div class="form-control">
          <label class="label">
            <span class="label-text">Sort Type</span>
          </label>
          <select class="select select-bordered" v-model="filters.sortDirection">
            <option v-for="opt in sortTypeOptions" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </option>
          </select>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// --- Icons ---
import { Filter, RefreshCcw, Search } from 'lucide-vue-next'

// --- Stores & helpers ---
import { useFiltersStore } from '../stores/Filters'
import { storeToRefs } from 'pinia'
import { onMounted, ref } from 'vue'
import { get_actors, get_countries, get_genres, get_tags } from '../functions/invoker'

// --- Components & types ---
import AutocompleteSelect from './AutocompleteSelect.vue'
import type { NumericalString } from '../type'

// --- State / lifecycle ---
const filtersStore = useFiltersStore()
const { filters } = storeToRefs(filtersStore)

const countries = ref<NumericalString[]>([])
const genres = ref<NumericalString[]>([])
const actors = ref<NumericalString[]>([])
const tags = ref<NumericalString[]>([])

onMounted(async () => {
  try {
    const [genresData, countriesData, actorsData, tagsData] = await Promise.all([
      get_genres(),
      get_countries(),
      get_actors(),
      get_tags(),
    ])
    genres.value = genresData
    countries.value = countriesData
    actors.value = actorsData
    tags.value = tagsData.map((tag) => [tag.id, tag.name])
  } catch (e) {
    console.error('Data fetching error:', e)
  }
})

// --- Boolean select options ---
const boolOptions = [
  { label: 'Any', value: null },
  { label: 'Yes', value: true },
  { label: 'No', value: false },
]

// --- Sort options ---
const sortByOptions = [
  { label: 'Name', value: 'name' },
  { label: 'Rating', value: 'imdb' },
  { label: 'Year', value: 'year' },
]

// --- Sort direction options ---
const sortTypeOptions = [
  { label: 'Descending', value: 'desc' },
  { label: 'Ascending', value: 'asc' },
]
</script>
