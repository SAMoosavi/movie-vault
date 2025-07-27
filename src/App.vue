<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

import type { VideoMetaData } from "./type"

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
    const syncPromises = dir_path.value.map(dir =>
      invoke('sync_app_files', { root: dir, apiKey: '4c602a26' })
    )

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
      invoke<string[]>('get_countries_app')
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

const filters = ref({ type: 'all', minRating: 0, country: 'all', genre: 'all' });
const searchTerm = ref('');

function resetFilters() {
  searchTerm.value = '';
  filters.value = { type: 'all', minRating: 0, country: 'all', genre: 'all' };
}


</script>

<template>

  <div class="navbar bg-base-100 shadow-lg sticky top-0 z-10">
    <!-- App Name -->
    <div class="navbar-start">
      <div class="dropdown">
        <div tabindex="0" role="button" class="btn btn-ghost lg:hidden">
          <AlignJustify class="w-5 h-5" />
        </div>
        <ul tabindex="0" class="menu menu-sm dropdown-content mt-3 z-10 p-2 shadow bg-base-100 rounded-box w-52">
          <li><a>Home</a></li>
          <li><a>Movies</a></li>
          <li><a>TV Shows</a></li>
          <li><a>Watchlist</a></li>
        </ul>
      </div>
      <span class="ml-2 text-xl font-bold bg-gradient-to-r from-primary to-secondary bg-clip-text text-transparent">
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
        <Sun class="swap-off w-5 h-5" />
        <Moon class="swap-on w-5 h-5" />
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
        <FolderPlus class="w-4 h-4" />
        Add Folder
      </button>
    </div>
  </div>

  <div class="container mx-auto px-4 py-6">

    <div class="p-0.5 card bg-gradient-to-br from-primary/50 to-secondary/50 mb-8">
      <div class="card bg-base-200 p-6">

        <div class="flex items-center mb-4">
          <Filter class="w-5 h-5 mr-2 text-primary" />
          <h2 class="text-lg font-semibold text-base-content">Filters</h2>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
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
            <select class="select select-bordered pr-8 w-full" v-model.number="filters.minRating">
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
            <select class="select select-bordered pr-8 w-full" v-model="filters.country">
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
      <div class="flex justify-between items-center mb-6">
        <div class="skeleton h-8 w-32"></div>
        <div class="skeleton h-4 w-48"></div>
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
        <div v-for="i in 8" :key="i" class="card shadow-md">
          <div class="w-full h-96 skeleton"></div>

          <div class="card-body p-4">
            <div class="h-6 skeleton rounded mb-2"></div>

            <div class="flex items-center text-sm mb-3">
              <div class="w-4 h-4 skeleton rounded-full mr-1"></div>
              <div class="h-3 w-16 skeleton rounded mr-2"></div>
              <div class="h-3 w-3 skeleton rounded-full mx-1"></div>
              <div class="h-3 w-12 skeleton rounded"></div>
            </div>

            <div class="flex flex-wrap gap-1 mb-3">
              <div class="h-4 w-12 skeleton rounded"></div>
              <div class="h-4 w-16 skeleton rounded"></div>
              <div class="h-4 w-14 skeleton rounded"></div>
            </div>

            <div class="flex items-center text-sm">
              <div class="w-4 h-4 skeleton rounded-full mr-1"></div>
              <div class="h-3 w-32 skeleton rounded"></div>
            </div>
          </div>
        </div>
      </div>
    </div>
    <main v-else>
      <!-- Results Info -->
      <div class="flex justify-between items-center mb-6">
        <h2 class="text-xl font-bold">
          {{ videos_metadata.length }} {{ videos_metadata.length === 1 ? 'Result' : 'Results' }}
        </h2>
        <div class="text-sm text-base-content/50">
          Showing {{ videos_metadata.length }} of {{ videos_metadata.length }} media items
        </div>
      </div>

      <div v-if="videos_metadata.length === 0" class="text-center py-12">
        <Hash class="w-16 h-16 mx-auto text-error/80 mb-4" />
        <h3 class="text-xl font-semibold mb-2">No movies found</h3>
        <p class="text-base-content">Try adjusting your filters or search terms</p>
      </div>
      <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
        <div v-for="(movie, i) in videos_metadata" :key="i"
          class="p-0.5 card bg-gradient-to-br from-primary/50 to-secondary/50 mb-8 transition-all duration-200 hover:scale-[1.02]">
          <div class="card bg-base-100">
            <figure class="relative">
              <img :src="movie.imdb_metadata?.poster" :alt="movie.imdb_metadata?.title || movie.name"
                class="w-full h-96 object-cover" />

              <div v-if="(movie.imdb_metadata?.imdb_rating || NA) !== NA"
                class="absolute bottom-3 left-3 badge badge-neutral flex items-center">
                <Star class="w-4 h-4 text-warning mr-1 fill-warning" />
                <span class="font-semibold">{{ movie.imdb_metadata?.imdb_rating }}</span>
              </div>
            </figure>

            <div class="card-body p-4">
              <h3 class="card-title text-lg truncate">
                {{ movie.imdb_metadata?.title || movie.name }}
              </h3>

              <div class="flex items-center text-sm text-base-content/70 mb-2">
                <Calendar class="w-4 h-4 mr-1" />
                <span>{{ movie.imdb_metadata?.year || movie.year }}</span>
                <span class="mx-2">•</span>
                <span class="capitalize">{{ movie.imdb_metadata?.type }}</span>
              </div>

              <div class="flex flex-wrap gap-1 mb-3">
                <span v-for="genre in movie.imdb_metadata?.genre" :key="genre" class="badge badge-outline badge-sm">
                  {{ genre }}
                </span>
              </div>

              <div class="flex items-center text-sm text-base-content/80">
                <User class="w-4 h-4 mr-1" />
                <div class="truncate">
                  {{ movie.imdb_metadata?.actors?.slice(0, 2).join(', ') }}<span
                    v-if="(movie.imdb_metadata?.actors?.length || 0) > 2">, ...</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>
