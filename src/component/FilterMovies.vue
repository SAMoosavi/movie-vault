<template>
  <!-- Modern Card Container with Enhanced Gradient -->
  <div class="card from-primary/40 to-secondary/40 mb-8 bg-gradient-to-br p-0.5 shadow-xl">
    <div class="card from-base-100 to-base-200 bg-gradient-to-br p-6">
      <!-- Enhanced Header Section -->
      <div class="mb-6 flex flex-col items-start justify-between gap-4 sm:flex-row sm:items-center">
        <div class="flex items-center">
          <div class="bg-primary/10 mr-3 rounded-lg p-2">
            <Filter class="text-primary h-5 w-5" />
          </div>
          <div>
            <h2 class="text-base-content text-xl font-bold">Movie Filters</h2>
            <p class="text-base-content/60 text-sm">Customize your movie discovery experience</p>
          </div>
        </div>
        <!-- Enhanced Reset Button -->
        <button
          type="reset"
          class="btn btn-outline btn-primary btn-sm flex items-center gap-2"
          @click="filtersStore.resetFilters()"
        >
          <RefreshCcw class="h-4 w-4" />
          Reset All
        </button>
      </div>

      <div class="flex flex-col gap-5">
        <!-- Enhanced Filters Grid -->
        <div class="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
          <!-- Movie Name Filter with Enhanced Styling -->
          <div class="form-control w-full">
            <label class="label">
              <span class="label-text font-medium">Movie name</span>
            </label>
            <div class="relative">
              <Search class="text-primary absolute top-1/2 left-3 z-10 h-6 w-6 -translate-y-1/2 transform" />
              <input
                v-model="filters.name"
                type="search"
                placeholder="Search movie name..."
                class="input input-bordered w-full pl-10 transition-all"
              />
            </div>
          </div>

          <!-- Country Filter with Enhanced Styling -->
          <div class="form-control w-full">
            <label class="label">
              <span class="label-text font-medium">Country</span>
            </label>
            <AutocompleteSelect
              @selected-items="(v) => (filters.country = v as number[])"
              :items="countries"
              class="transition-all"
            />
          </div>

          <!-- Genre Filter with Enhanced Styling -->
          <div class="form-control w-full">
            <label class="label">
              <span class="label-text font-medium">Genre</span>
            </label>
            <AutocompleteSelect
              @selected-items="(v) => (filters.genre = v as number[])"
              :items="genres"
              class="transition-all"
            />
          </div>

          <!-- Actor Filter with Enhanced Styling -->
          <div class="form-control w-full">
            <label class="label">
              <span class="label-text font-medium">Person</span>
            </label>
            <AutocompleteSelect
              @selected-items="(v) => (filters.people = v as string[])"
              :items="people"
              class="transition-all"
            />
          </div>

          <!-- Tag Filter with Enhanced Styling -->
          <div class="form-control w-full">
            <label class="label">
              <span class="label-text font-medium">Tag</span>
            </label>
            <AutocompleteSelect
              @selected-items="(v) => (filters.tags = v as number[])"
              :items="tags"
              class="transition-all"
            />
          </div>

          <!-- Enhanced Minimum Rating Filter using Range Slider -->
          <div class="form-control w-full">
            <label class="label">
              <span class="label-text flex w-full justify-between font-medium">
                <span>Min Rating</span>
              </span>
            </label>

            <div class="w-full">
              <input
                type="range"
                :min="0"
                :max="10"
                v-model.number.lazy="filters.minRating"
                class="range range-primary w-full"
                :step="1"
              />

              <div class="mt-2 flex w-full justify-between px-2.5 text-xs">
                <span>0</span>
                <span>2</span>
                <span>4</span>
                <span>6</span>
                <span>8</span>
                <span>10</span>
              </div>
            </div>
          </div>

          <!-- Enhanced Sort By Filter -->
          <div class="form-control w-full">
            <label class="label">
              <span class="label-text font-medium">Sort By</span>
            </label>

            <div class="join w-full">
              <select class="join-item select select-bordered w-full transition-all" v-model="filters.sortBy">
                <option v-for="opt in sortByOptions" :key="opt.value" :value="opt.value">
                  {{ opt.label }}
                </option>
              </select>
              <button class="btn join-item btn-outline btn-primary">
                <ArrowDownNarrowWide
                  v-if="filters.sortDirection === 'asc'"
                  @click="() => (filters.sortDirection = 'desc')"
                  class="h-6 w-6"
                />
                <ArrowDownWideNarrow v-else @click="() => (filters.sortDirection = 'asc')" class="h-6 w-6" />
              </button>
            </div>
          </div>
        </div>

        <!-- Enhanced Toggle Filters Grid -->
        <div class="grid grid-cols-2 gap-6 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5">
          <!-- Enhanced Has IMDb Filter using Toggle -->
          <div class="form-control w-full">
            <label class="label">
              <span class="label-text font-medium">Has IMDb</span>
            </label>
            <div class="filter">
              <input
                class="btn filter-reset btn-error"
                type="radio"
                @click="filters.existImdb = null"
                name="has_imdb"
                aria-label="All"
                :checked="filters.existImdb === null"
              />
              <input
                class="btn checked:btn-primary transition-all duration-150"
                type="radio"
                @click="filters.existImdb = true"
                name="has_imdb"
                aria-label="Yes"
              />
              <input
                class="btn checked:btn-primary transition-all duration-150"
                type="radio"
                @click="filters.existImdb = false"
                name="has_imdb"
                aria-label="No"
              />
            </div>
          </div>

          <!-- Enhanced Multiple Files Filter using Toggle -->
          <div class="form-control w-full">
            <label class="label">
              <span class="label-text font-medium">Multiple Files</span>
            </label>
            <div class="filter">
              <input
                class="btn filter-reset btn-error"
                type="radio"
                @click="filters.existMultiFile = null"
                name="multi_file"
                aria-label="All"
                :checked="filters.existMultiFile === null"
              />
              <input
                class="btn checked:btn-primary transition-all duration-150"
                type="radio"
                @click="filters.existMultiFile = true"
                name="multi_file"
                aria-label="Yes"
              />
              <input
                class="btn checked:btn-primary transition-all duration-150"
                type="radio"
                @click="filters.existMultiFile = false"
                name="multi_file"
                aria-label="No"
              />
            </div>
          </div>

          <!-- Enhanced Watched Filter using Toggle -->
          <div class="form-control w-full">
            <label class="label">
              <span class="label-text font-medium">Watched</span>
            </label>
            <div class="filter">
              <input
                class="btn filter-reset btn-error"
                type="radio"
                @click="filters.watched = null"
                name="watched"
                aria-label="All"
                :checked="filters.watched === null"
              />
              <input
                class="btn checked:btn-primary transition-all duration-150"
                type="radio"
                @click="filters.watched = true"
                name="watched"
                aria-label="Yes"
              />
              <input
                class="btn checked:btn-primary transition-all duration-150"
                type="radio"
                @click="filters.watched = false"
                name="watched"
                aria-label="No"
              />
            </div>
          </div>

          <!-- Enhanced Watch List Filter using Toggle -->
          <div class="form-control w-full">
            <label class="label">
              <span class="label-text font-medium">Watch List</span>
            </label>
            <div class="filter">
              <input
                class="btn filter-reset btn-error"
                type="radio"
                @click="filters.watchList = null"
                name="watch_list"
                aria-label="All"
                :checked="filters.watchList === null"
              />
              <input
                class="btn checked:btn-primary transition-all duration-150"
                type="radio"
                @click="filters.watchList = true"
                name="watch_list"
                aria-label="Yes"
              />
              <input
                class="btn checked:btn-primary transition-all duration-150"
                type="radio"
                @click="filters.watchList = false"
                name="watch_list"
                aria-label="No"
              />
            </div>
          </div>

          <!-- Type Filter with Enhanced Styling -->
          <div class="form-control col-span-1 w-full md:col-span-2 lg:col-span-1">
            <label class="label">
              <span class="label-text font-medium">Type</span>
            </label>
            <div class="filter">
              <input
                class="btn filter-reset btn-error"
                type="radio"
                @click="filters.type = 'all'"
                name="watch_list"
                aria-label="All"
                :checked="filters.type === null"
              />
              <input
                class="btn checked:btn-primary transition-all duration-150"
                type="radio"
                @click="filters.type = 'movie'"
                name="watch_list"
                aria-label="Movies"
              />
              <input
                class="btn checked:btn-primary transition-all duration-150"
                type="radio"
                @click="filters.type = 'series'"
                name="watch_list"
                aria-label="Series"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// --- Icons ---
import { ArrowDownNarrowWide, ArrowDownWideNarrow, Filter, RefreshCcw, Search } from 'lucide-vue-next'

// --- Stores & helpers ---
import { useFiltersStore } from '../stores/Filters'
import { storeToRefs } from 'pinia'
import { onMounted, ref } from 'vue'
import { get_people, get_countries, get_genres, get_tags } from '../functions/invoker'

// --- Components & types ---
import AutocompleteSelect from './AutocompleteSelect.vue'
import type { NumericalString } from '../type'

// --- State / lifecycle ---
const filtersStore = useFiltersStore()
const { filters } = storeToRefs(filtersStore)

const countries = ref<NumericalString[]>([])
const genres = ref<NumericalString[]>([])
const people = ref<NumericalString[]>([])
const tags = ref<NumericalString[]>([])

onMounted(async () => {
  try {
    const [genresData, countriesData, peopleData, tagsData] = await Promise.all([
      get_genres(),
      get_countries(),
      get_people(),
      get_tags(),
    ])
    genres.value = genresData
    countries.value = countriesData
    people.value = peopleData
    tags.value = tagsData.map((tag) => [tag.id, tag.name])
  } catch (e) {
    console.error('Data fetching error:', e)
  }
})

// --- Sort options ---
const sortByOptions = [
  { label: 'Name', value: 'name' },
  { label: 'Rating', value: 'imdb' },
  { label: 'Year', value: 'year' },
]
</script>
