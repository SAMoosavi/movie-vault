<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

import type { VideoMetaData } from './type'

import { Calendar, Filter, Hash, Star, User, Search, FolderPlus, Sun, Moon, AlignJustify } from 'lucide-vue-next'
import { toast } from 'vue3-toastify'

const NA = 'N/A'

const loading = ref(true)
const countries = ref<string[]>([])
const genres = ref<string[]>([])
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
      invoke<string[]>('get_genres_app'),
      invoke<string[]>('get_countries_app'),
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

async function add_dir() {
  try {
    const selectedDir = await open({
      multiple: false,
      directory: true,
    })

    if (!selectedDir) {
      toast.info('No directory selected')
      return
    }

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

const filters = ref({ type: 'all', minRating: 0, country: 'all', genre: 'all' })
const searchTerm = ref('')

function resetFilters() {
  searchTerm.value = ''
  filters.value = { type: 'all', minRating: 0, country: 'all', genre: 'all' }
}
</script>

<template>
  <div class="navbar bg-base-100 sticky top-0 z-10 shadow-lg">
    <!-- App Name -->
    <div class="navbar-start">
      <div class="dropdown">
        <div tabindex="0" role="button" class="btn btn-ghost lg:hidden">
          <AlignJustify class="h-5 w-5" />
        </div>
        <ul tabindex="0" class="menu menu-sm dropdown-content bg-base-100 rounded-box z-10 mt-3 w-52 p-2 shadow">
          <li><a>Home</a></li>
          <li><a>Movies</a></li>
          <li><a>TV Shows</a></li>
          <li><a>Watchlist</a></li>
        </ul>
      </div>
      <span class="from-primary to-secondary ml-2 bg-gradient-to-r bg-clip-text text-xl font-bold text-transparent">
        Movie Vault
      </span>
    </div>

    <!-- Desktop Menu -->
    <div class="navbar-center hidden lg:flex">
      <ul class="menu menu-horizontal px-1">
        <li><a class="hover:bg-base-200 rounded-lg">Home</a></li>
        <li><a class="hover:bg-base-200 rounded-lg">Movies</a></li>
        <li><a class="hover:bg-base-200 rounded-lg">TV Shows</a></li>
        <li><a class="hover:bg-base-200 rounded-lg">Watchlist</a></li>
      </ul>
    </div>

    <!-- Search and Actions -->
    <div class="navbar-end flex items-center gap-2">
      <!-- Theme Toggle -->
      <label class="swap swap-rotate">
        <input type="checkbox" class="theme-controller" value="synthwave" />
        <Sun class="swap-off h-5 w-5" />
        <Moon class="swap-on h-5 w-5" />
      </label>

      <!-- Search Box -->
      <div class="form-control">
        <label class="input input-bordered input-sm w-full max-w-xs">
          <Search class="h-4" />
          <input type="search" placeholder="Search movie name..." />
        </label>
      </div>

      <!-- Add Folder Button -->
      <button class="btn btn-primary btn-sm" @click="add_dir">
        <FolderPlus class="h-4 w-4" />
        Add Folder
      </button>
    </div>
  </div>

  <div class="container mx-auto px-4 py-6">
    <div class="card from-primary/50 to-secondary/50 mb-8 bg-gradient-to-br p-0.5">
      <div class="card bg-base-200 p-6">
        <div class="mb-4 flex items-center">
          <Filter class="text-primary mr-2 h-5 w-5" />
          <h2 class="text-base-content text-lg font-semibold">Filters</h2>
        </div>

        <div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-5">
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
              <option value="all">All countries</option>
              <option v-for="country in countries" :key="country" :value="country">
                {{ country }}
              </option>
            </select>
          </div>

          <!-- Genre -->
          <div class="form-control">
            <label class="label">
              <span class="label-text">Genre</span>
            </label>
            <select class="select select-bordered" v-model="filters.genre">
              <option value="all">All Genres</option>
              <option v-for="genre in genres" :key="genre" :value="genre">
                {{ genre }}
              </option>
            </select>
          </div>

          <!-- Reset Button -->
          <div class="flex items-end">
            <button type="reset" class="btn btn-primary btn-block transition-transform" @click="resetFilters">
              Reset Filters
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Movie Grid -->
    <div v-if="loading">
      <div class="mb-6 flex items-center justify-between">
        <div class="skeleton h-8 w-32"></div>
        <div class="skeleton h-4 w-48"></div>
      </div>

      <div class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
        <div v-for="i in 8" :key="i" class="card shadow-md">
          <div class="skeleton h-96 w-full"></div>

          <div class="card-body p-4">
            <div class="skeleton mb-2 h-6 rounded"></div>

            <div class="mb-3 flex items-center text-sm">
              <div class="skeleton mr-1 h-4 w-4 rounded-full"></div>
              <div class="skeleton mr-2 h-3 w-16 rounded"></div>
              <div class="skeleton mx-1 h-3 w-3 rounded-full"></div>
              <div class="skeleton h-3 w-12 rounded"></div>
            </div>

            <div class="mb-3 flex flex-wrap gap-1">
              <div class="skeleton h-4 w-12 rounded"></div>
              <div class="skeleton h-4 w-16 rounded"></div>
              <div class="skeleton h-4 w-14 rounded"></div>
            </div>

            <div class="flex items-center text-sm">
              <div class="skeleton mr-1 h-4 w-4 rounded-full"></div>
              <div class="skeleton h-3 w-32 rounded"></div>
            </div>
          </div>
        </div>
      </div>
    </div>
    <main v-else>
      <!-- Results Info -->
      <div class="mb-6 flex items-center justify-between">
        <h2 class="text-xl font-bold">
          {{ videos_metadata.length }} {{ videos_metadata.length === 1 ? 'Result' : 'Results' }}
        </h2>
        <div class="text-base-content/50 text-sm">
          Showing {{ videos_metadata.length }} of {{ videos_metadata.length }} media items
        </div>
      </div>

      <div v-if="videos_metadata.length === 0" class="py-12 text-center">
        <Hash class="text-error/80 mx-auto mb-4 h-16 w-16" />
        <h3 class="mb-2 text-xl font-semibold">No movies found</h3>
        <p class="text-base-content">Try adjusting your filters or search terms</p>
      </div>
      <div v-else class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
        <div
          v-for="(movie, i) in videos_metadata"
          :key="i"
          class="card from-primary/50 to-secondary/50 mb-8 bg-gradient-to-br p-0.5 transition-all duration-200 hover:scale-[1.02]"
        >
          <div class="card bg-base-100">
            <figure class="relative">
              <img
                :src="movie.imdb_metadata?.poster"
                :alt="movie.imdb_metadata?.title || movie.name"
                class="h-96 w-full object-cover"
              />

              <div
                v-if="(movie.imdb_metadata?.imdb_rating || NA) !== NA"
                class="badge badge-neutral absolute bottom-3 left-3 flex items-center"
              >
                <Star class="text-warning fill-warning mr-1 h-4 w-4" />
                <span class="font-semibold">{{ movie.imdb_metadata?.imdb_rating }}</span>
              </div>
            </figure>

            <div class="card-body p-4">
              <h3 class="card-title truncate text-lg">
                {{ movie.imdb_metadata?.title || movie.name }}
              </h3>

              <div class="text-base-content/70 mb-2 flex items-center text-sm">
                <Calendar class="mr-1 h-4 w-4" />
                <span>{{ movie.imdb_metadata?.year || movie.year }}</span>
                <span class="mx-2">•</span>
                <span class="capitalize">{{ movie.imdb_metadata?.type }}</span>
              </div>

              <div class="mb-3 flex flex-wrap gap-1">
                <span v-for="genre in movie.imdb_metadata?.genre" :key="genre" class="badge badge-outline badge-sm">
                  {{ genre }}
                </span>
              </div>

              <div class="text-base-content/80 flex items-center text-sm">
                <User class="mr-1 h-4 w-4" />
                <div class="truncate">
                  {{ movie.imdb_metadata?.actors?.slice(0, 2).join(', ') }}
                  <span v-if="(movie.imdb_metadata?.actors?.length || 0) > 2">, ...</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>
