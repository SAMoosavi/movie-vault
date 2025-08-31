<template>
  <!-- Main Container -->
  <main class="container mx-auto px-4 py-6">
    <!-- Movie filter controls -->
    <FilterMovies />

    <!-- Loading indicator -->
    <LoadingView v-if="isLoading" />

    <!-- Movie results grid -->
    <div v-else>
      <ResultsInfo :totalMovies="movies.length" :numberOfSearchedMovies="movies.length" />

      <!-- No movies found message -->
      <NotFoundMovies v-if="movies.length === 0" />

      <!-- Display movie cards in a grid -->
      <div v-else class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
        <MovieCard v-for="movie in movies" :key="movie.id" :movie="movie" />
      </div>
    </div>
  </main>
</template>

<script setup lang="ts">
// --- Vue & toast ---
import { onMounted, ref, watch } from 'vue'
import { toast } from 'vue3-toastify'

// --- Components ---
import FilterMovies from '../component/FilterMovies.vue'
import LoadingView from '../component/LoadingView.vue'
import ResultsInfo from '../component/ResultsInfo.vue'
import NotFoundMovies from '../component/NotFoundMovies.vue'
import MovieCard from '../component/MovieCard.vue'

// --- Stores ---
import { useVideosStore } from '../stores/Videos'
import { useFiltersStore } from '../stores/Filters'
import { storeToRefs } from 'pinia'

// --- State ---
const isLoading = ref(true)
const videosStore = useVideosStore()
const { videos: movies } = storeToRefs(videosStore)
const filtersStore = useFiltersStore()
const { filters } = storeToRefs(filtersStore)

// --- Lifecycle: initial load ---
onMounted(async () => {
  try {
    await videosStore.reload()
  } catch (error) {
    console.error('Error loading movies:', error)
    toast.error(`Failed to load movies: ${error instanceof Error ? error.message : 'Unknown error'}`)
  } finally {
    isLoading.value = false
  }
})

watch(filters, () => fetchMovies(), { deep: true })

async function fetchMovies() {
  isLoading.value = true
  try {
    await videosStore.reload()
  } catch (error) {
    toast.error(`Failed to reload movies: ${error}`)
  } finally {
    isLoading.value = false
  }
}
</script>
