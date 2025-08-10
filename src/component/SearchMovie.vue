<template>
  <div class="card from-primary/50 to-secondary/50 mb-8 bg-gradient-to-br p-0.5 shadow-xl">
    <div class="card bg-base-100 p-5">
      <div class="form-control">
        <label class="label">
          <span class="label-text font-semibold">Search Imdb of {{ movie.name }}</span>
          <span v-if="movie.year"> ({{ movie.year }})</span>
        </label>
        <div class="join mt-3 w-full">
          <input
            v-model="movieName"
            type="text"
            placeholder="Enter movie name..."
            class="join-item input input-bordered w-full"
          />
          <button v-if="change" class="join-item btn btn-primary" @click="$emit('cancel')">cancel change</button>
        </div>
      </div>

      <!-- Search Results -->
      <div v-if="loading" class="mt-6">
        <h3 class="mb-4 text-xl font-semibold">Search Results</h3>
        <div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
          <div
            v-for="i in 6"
            :key="i"
            class="card from-primary/50 to-secondary/50 mb-8 bg-gradient-to-br p-0.5 shadow-xl"
          >
            <div class="card card-compact bg-base-100 cursor-pointer shadow-lg transition-shadow hover:shadow-xl">
              <figure class="bg-base-200 h-48 rounded-t-2xl">
                <div class="bg-base-300 skeleton h-full w-full rounded-t-2xl"></div>
              </figure>
              <div class="card-body">
                <div class="bg-base-300 skeleton mb-2 h-6 w-3/4 rounded"></div>
                <div class="bg-base-300 skeleton h-4 w-1/2 rounded"></div>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div v-else-if="searchItems.length > 0" class="mt-6">
        <h3 class="mb-4 text-xl font-semibold">Search Results</h3>
        <div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
          <div
            v-for="item in searchItems"
            :key="item['#IMDB_ID']"
            class="card from-primary/50 to-secondary/50 mb-8 bg-gradient-to-br p-0.5 shadow-xl transition-all duration-200 hover:scale-105"
          >
            <div
              class="card card-compact bg-base-100 h-full cursor-pointer shadow-lg transition-shadow hover:shadow-xl"
              @click="selectMovie(item['#IMDB_ID'])"
            >
              <figure class="h-48">
                <img
                  :src="
                    item['#IMG_POSTER'] !== 'N/A' ? item['#IMG_POSTER'] : 'https://placehold.co/300x450?text=No+Image'
                  "
                  :alt="item['#TITLE']"
                  class="h-full w-full object-cover"
                />
              </figure>
              <div class="card-body">
                <h4 class="card-title text-lg">{{ item['#TITLE'] }}</h4>
                <p class="text-sm opacity-70">{{ item['#YEAR'] }}</p>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- No Results -->
      <div v-else-if="searchItems.length === 0 && movieName" class="py-8 text-center">
        <SearchX class="mx-auto mb-4 h-16 w-16" />
        <p class="text-base-content/70">No movies found for "{{ movieName }}"</p>
      </div>

      <!-- Empty State -->
      <div v-else class="py-12 text-center">
        <Search class="mx-auto mb-4 h-16 w-16" />
        <p class="text-base-content/70">Enter a movie name to search for information</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import type { VideoMetaData } from '../type'
import { update_video_imdb } from '../functions/invoker'
import { toast } from 'vue3-toastify'
import { fetch } from '@tauri-apps/plugin-http'
import { SearchX, Search } from 'lucide-vue-next'
import type { MovieSearchResult, SearchedMovie } from './SearchMovie'

const props = defineProps<{ movie: VideoMetaData; change: boolean }>()
const emit = defineEmits(['cancel', 'updated'])

const movieName = ref(props.movie.name)

const searchItems = ref<SearchedMovie[]>([])
const loading = ref(false)

async function searchMovies(title: string) {
  if (title.length == 0) {
    loading.value = false
    searchItems.value = []
    return
  }

  loading.value = true
  const response = await fetch(`https://imdb.iamidiotareyoutoo.com/search?q=${encodeURIComponent(title)}`)
  const result: MovieSearchResult = await response.json()
  searchItems.value = result.description
  loading.value = false
}

watch(movieName, searchMovies)

onMounted(() => searchMovies(movieName.value))

async function selectMovie(imdb_id: string) {
  try {
    await update_video_imdb(props.movie.id, imdb_id)
    emit('updated')
  } catch (e: unknown) {
    const message = e instanceof Error ? e.message : 'Failed to set imdb'
    toast.error(message)
  } finally {
    emit('cancel')
  }
}
</script>
