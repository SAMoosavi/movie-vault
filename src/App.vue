<template>
  <AppNavbar @add-dir="addDir" />

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
import { onMounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { FilterValues, VideoMetaData } from './type'
import { toast } from 'vue3-toastify'
import AppNavbar from './component/AppNavbar.vue'
import FilterMovies from './component/FilterMovies.vue'
import LoadingView from './component/LoadingView.vue'
import ResultsInfo from './component/ResultsInfo.vue'
import NotFoundMovies from './component/NotFoundMovies.vue'
import MovieCard from './component/MovieCard.vue'

const loading = ref(true)
const countries = ref<[number, string][]>([])
const genres = ref<[number, string][]>([])
const videos_metadata = ref<VideoMetaData[]>([])
const dir_path = ref<string[]>(['/run/media/sam/film/marvel']) // default for test

onMounted(async () => {
  try {
    // Initialize database
    await invoke('create_table_app')

    // Sync files with better error handling
    const syncPromises = dir_path.value.map((dir) => invoke('sync_app_files', { root: dir, apiKey: '4c602a26' }))

    await Promise.all(syncPromises)

    toast.success('Database initialized and files synced successfully!')
  } catch (e) {
    console.error('Initialization error:', e)
    toast.error(`Failed to initialize: ${e instanceof Error ? e.message : 'Unknown error'}`)
  }

  try {
    // Fetch all data in parallel for better performance
    const [videos, genresData, countriesData] = await Promise.all([
      invoke<VideoMetaData[]>('get_all_video_metadata_app'),
      invoke<[number, string][]>('get_genres_app'),
      invoke<[number, string][]>('get_countries_app'),
    ])

    videos_metadata.value = videos
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

async function addDir(selectedDir: string) {
  try {
    // Check if directory already exists
    if (dir_path.value.includes(selectedDir)) {
      toast.warning('Directory already added')
      return
    }

    toast.info('Adding directory and syncing files...')

    // Add directory to list
    dir_path.value.push(selectedDir)

    // Sync files in the new directory
    await invoke('sync_app_files', { root: selectedDir, apiKey: '4c602a26' })

    // Refresh video metadata
    const prev_number = videos_metadata.value.length
    videos_metadata.value = await invoke<VideoMetaData[]>('get_all_video_metadata_app')

    toast.success(`Successfully added directory with ${videos_metadata.value.length - prev_number} items!`)
  } catch (error) {
    // Remove the directory if sync failed
    if (dir_path.value.length > 0) {
      dir_path.value.pop()
    }

    console.error('Error adding directory:', error)
    toast.error(`Failed to add directory: ${error instanceof Error ? error.message : 'Unknown error'}`)
  }
}

const filterName = ref<string>('')

watch(filterName, (v) => console.log(v))

function search(filters: FilterValues) {
  console.table(filters)
}
</script>
