<template>
  <main class="container mx-auto px-4 py-6">
    <FilterMovies :countries="countries" :genres="genres" @search="search" />

    <LoadingView v-if="loading" />
    <!-- Movie Grid -->
    <div v-else>
      <ResultsInfo :totalMovies="videos_metadata.length" :numberOfSearchedMovies="videos_metadata.length" />

      <NotFoundMovies v-if="videos_metadata.length === 0" />
      <div v-else class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
        <MovieCard v-for="(movie, i) in videos_metadata" :key="i" :movie="movie" />
      </div>
    </div>
  </main>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import type { FilterValues } from '../type'
import { toast } from 'vue3-toastify'
import FilterMovies from '../component/FilterMovies.vue'
import LoadingView from '../component/LoadingView.vue'
import ResultsInfo from '../component/ResultsInfo.vue'
import NotFoundMovies from '../component/NotFoundMovies.vue'
import MovieCard from '../component/MovieCard.vue'
import { create_table, get_countries, get_genres, sync_app } from '../functions/invoker'
import { useVideosStore } from '../stores/Videos'
import { storeToRefs } from 'pinia'
import { useDirsStore } from '../stores/Dirs'

const loading = ref(true)
const countries = ref<[number, string][]>([])
const genres = ref<[number, string][]>([])

const videos = useVideosStore()
const { videos_metadata } = storeToRefs(videos);

const { dir_path } = storeToRefs(useDirsStore())

onMounted(async () => {
  try {
    // Initialize database
    await create_table()

    // Sync files with better error handling
    const syncPromises = dir_path.value.map(sync_app)
    await Promise.all(syncPromises)

    toast.success('Database initialized and files synced successfully!')
  } catch (e) {
    console.error('Initialization error:', e)
    toast.error(`Failed to initialize: ${e instanceof Error ? e.message : 'Unknown error'}`)
  }

  try {
    // Fetch all data in parallel for better performance
    const [genresData, countriesData] = await Promise.all([
      get_genres(),
      get_countries(),
      videos.updata(),
    ])

    genres.value = genresData
    countries.value = countriesData

    toast.success('Data loaded successfully!')
  } catch (e) {
    console.error('Data fetching error:', e)
    toast.error(`Failed to load data: ${e instanceof Error ? e.message : 'Unknown error'}`)
  } finally {
    loading.value = false
  }
})

async function search(filters: FilterValues) {
  loading.value = true
  videos.search(filters)
    .then(() => { })
    .catch((e) => toast.error(e))
    .finally(() => (loading.value = false))
}
</script>
