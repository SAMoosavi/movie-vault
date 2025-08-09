<template>
  <div class="card from-primary/50 to-secondary/50 mb-8 bg-gradient-to-br p-0.5">
    <div class="card bg-base-200 p-6">
      <div class="mb-4 flex items-center">
        <Filter class="text-primary mr-2 h-5 w-5" />
        <h2 class="text-base-content text-lg font-semibold">Filters</h2>
      </div>

      <div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-5">
        <!-- Search name -->
        <div class="form-control">
          <label class="label">
            <span class="label-text">Movie name</span>
          </label>
          <label class="input input-bordered w-full max-w-xs">
            <Search class="h-4" />
            <input v-model="filters.name" type="search" placeholder="Search movie name..." />
          </label>
        </div>

        <!-- Type -->
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

        <!-- Min Rating -->
        <div class="form-control">
          <label class="label">
            <span class="label-text">Min Rating</span>
          </label>
          <select class="select select-bordered w-full pr-8" v-model.number="filters.minRating">
            <option value="0">Any Rating</option>
            <option value="7">7+ Stars</option>
            <option value="8">8+ Stars</option>
            <option value="9">9+ Stars</option>
          </select>
        </div>

        <!-- Country -->
        <div class="form-control">
          <label class="label">
            <span class="label-text">Country</span>
          </label>
          <select class="select select-bordered w-full pr-8" v-model="filters.country">
            <option :value="0">All countries</option>
            <option v-for="country in countries" :key="country[0]" :value="country[0]">
              {{ country[1] }}
            </option>
          </select>
        </div>

        <!-- Genre -->
        <div class="form-control">
          <label class="label">
            <span class="label-text">Genre</span>
          </label>
          <select class="select select-bordered" v-model="filters.genre">
            <option :value="0">All Genres</option>
            <option v-for="genre in genres" :key="genre[0]" :value="genre[0]">
              {{ genre[1] }}
            </option>
          </select>
        </div>

        <!-- Reset Button -->
        <div class="flex items-end">
          <button
            type="reset"
            class="btn btn-primary btn-block transition-transform"
            @click="filtersStore.resetFilters()"
          >
            Reset Filters
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Filter, Search } from 'lucide-vue-next'
import { useFiltersStore } from '../stores/Filters'
import { storeToRefs } from 'pinia'
import { onMounted, ref } from 'vue'
import { get_countries, get_genres } from '../functions/invoker'

const filtersStore = useFiltersStore()
const { filters } = storeToRefs(filtersStore)

const countries = ref<[number, string][]>([])
const genres = ref<[number, string][]>([])

onMounted(async () => {
  try {
    // Fetch all data in parallel for better performance
    const [genresData, countriesData] = await Promise.all([get_genres(), get_countries()])

    genres.value = genresData
    countries.value = countriesData
  } catch (e) {
    console.error('Data fetching error:', e)
  }
})
</script>
