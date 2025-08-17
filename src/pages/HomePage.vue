<template>
  <main class="container mx-auto px-4 py-6">
    <FilterMovies />

    <LoadingView v-if="loading" />
    <!-- Movie Grid -->
    <div v-else>
      <ResultsInfo :totalMovies="videos_metadata.length" :numberOfSearchedMovies="videos_metadata.length" />

      <NotFoundMovies v-if="videos_metadata.length === 0" />
      <div v-else class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
        <MovieCard v-for="movie in videos_metadata" :key="movie.id" :movie="movie" />
      </div>
    </div>
  </main>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { toast } from 'vue3-toastify'
import FilterMovies from '../component/FilterMovies.vue'
import LoadingView from '../component/LoadingView.vue'
import ResultsInfo from '../component/ResultsInfo.vue'
import NotFoundMovies from '../component/NotFoundMovies.vue'
import MovieCard from '../component/MovieCard.vue'
import { useVideosStore } from '../stores/Videos'
import { storeToRefs } from 'pinia'
import { useFiltersStore } from '../stores/Filters'

const loading = ref(true)

const videos = useVideosStore()
const { videos_metadata } = storeToRefs(videos)

onMounted(async () => {
  try {
    await videos.reload_media()
  } catch (e) {
    console.error('Data fetching error:', e)
    toast.error(`Failed to load data: ${e instanceof Error ? e.message : 'Unknown error'}`)
  } finally {
    loading.value = false
  }
})

const filtersStore = useFiltersStore()

const { filters } = storeToRefs(filtersStore)

// Watch and emit on change
watch(filters, () => search(), { deep: true })

async function search() {
  loading.value = true
  videos
    .reload_media()
    .then(() => {})
    .catch((e) => toast.error(e))
    .finally(() => (loading.value = false))
}
</script>
