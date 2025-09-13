<template>
  <div class="card from-primary/50 to-secondary/50 mb-8 bg-gradient-to-br p-0.5 shadow-xl">
    <div class="card bg-base-100 p-5">
      <!-- Header -->
      <div class="flex flex-wrap items-center gap-4">
        <label class="label flex gap-2">
          <span class="label-text font-semibold">Search IMDb of {{ movie.name }}</span>
          <span v-if="movie.year" class="opacity-70">({{ movie.year }})</span>
        </label>

        <template v-if="!is_editing">
          <!-- Rating -->
          <div class="flex items-center gap-1">
            <StarIcon
              v-for="i in 5"
              :key="i"
              class="text-warning h-5 w-5 cursor-pointer transition-colors"
              :class="{ 'fill-warning': movie.my_ranking >= i }"
              @click="$emit('set-ranking', i)"
            />
          </div>

          <!-- Watched toggle -->
          <button class="btn btn-circle btn-ghost hover:btn-primary" @click="$emit('toggle-watched')">
            <component :is="movie.watched ? EyeIcon : EyeOffIcon" class="h-5 w-5" />
          </button>

          <!-- Watchlist toggle -->
          <button class="btn btn-circle btn-ghost hover:btn-primary" @click="$emit('toggle-watch-list')">
            <component :is="movie.watch_list ? BookmarkCheckIcon : BookmarkIcon" class="h-5 w-5" />
          </button>
        </template>
      </div>

      <!-- Search Input -->
      <div class="join mt-4 w-full">
        <input
          v-model="movieName"
          type="text"
          placeholder="Enter movie name..."
          class="join-item input input-bordered w-full"
        />
        <button v-if="is_editing" class="join-item btn btn-primary" @click="$emit('cancel')">Cancel</button>
      </div>

      <!-- Results -->
      <div v-if="loading || searchItems.length" class="mt-8">
        <h3 class="mb-4 text-xl font-semibold">Search Results</h3>

        <div class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-8">
          <!-- Skeletons -->
          <template v-if="loading">
            <div
              v-for="i in 8"
              :key="i"
              class="card from-primary/50 to-secondary/50 h-60 bg-gradient-to-br p-0.5 shadow-xl"
            >
              <div class="card card-compact relative h-full w-full overflow-hidden shadow-lg">
                <div class="skeleton h-full w-full"></div>
                <div class="badge absolute top-2 right-2 h-6 w-10"></div>
                <div class="badge absolute bottom-2 left-1/2 h-6 w-16 -translate-x-1/2"></div>
              </div>
            </div>
          </template>

          <!-- Items -->
          <template v-else>
            <div
              v-for="item in searchItems"
              :key="item['#IMDB_ID']"
              class="card from-primary/50 to-secondary/50 cursor-pointer bg-gradient-to-br p-0.5 shadow-xl transition hover:scale-[1.02] hover:shadow-2xl"
              @click="selectMovie(item['#IMDB_ID'])"
            >
              <div class="card card-compact bg-base-100 h-full w-full overflow-hidden shadow-lg">
                <figure class="relative h-full">
                  <img
                    :src="
                      item['#IMG_POSTER'] !== 'N/A' ? item['#IMG_POSTER'] : 'https://placehold.co/300x450?text=No+Image'
                    "
                    :alt="item['#TITLE']"
                    class="h-full w-full object-cover"
                    loading="lazy"
                  />

                  <!-- Year -->
                  <div class="badge badge-primary absolute top-2 right-2 flex items-center gap-1 text-xs">
                    <CalendarIcon class="h-3 w-3" />
                    <span>{{ item['#YEAR'] }}</span>
                  </div>

                  <!-- Title -->
                  <div class="badge badge-secondary absolute bottom-2 left-1/2 -translate-x-1/2 text-xs text-nowrap">
                    {{ item['#TITLE'] }}
                  </div>
                </figure>
              </div>
            </div>
          </template>
        </div>
      </div>

      <!-- No Results -->
      <div v-else-if="movieName" class="py-12 text-center">
        <SearchX class="mx-auto mb-4 h-16 w-16 opacity-60" />
        <p class="text-base-content/70">No movies found for "{{ movieName }}"</p>
      </div>

      <!-- Empty State -->
      <div v-else class="py-16 text-center">
        <Search class="mx-auto mb-4 h-16 w-16 opacity-60" />
        <p class="text-base-content/70">Enter a movie name to search</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount } from 'vue'
import { fetch } from '@tauri-apps/plugin-http'
import { toast } from 'vue3-toastify'
import {
  SearchX,
  Search,
  EyeIcon,
  EyeOffIcon,
  StarIcon,
  BookmarkIcon,
  BookmarkCheckIcon,
  CalendarIcon,
} from 'lucide-vue-next'
import type { Media } from '../type'
import { update_media_imdb } from '../functions/invoker'
import type { MovieSearchResult, SearchedMovie } from './SearchMovie'

const props = defineProps<{ movie: Media; is_editing?: boolean }>()
const emit = defineEmits<{
  (e: 'cancel'): void
  (e: 'updated', id: number): void
  (e: 'set-ranking', rank: number): void
  (e: 'toggle-watched'): void
  (e: 'toggle-watch-list'): void
}>()

const movieName = ref(props.movie?.name ?? '')
const searchItems = ref<SearchedMovie[]>([])
const loading = ref(false)

let debounceTimer: number | undefined

async function performSearch(query: string) {
  const title = query.trim()
  if (!title) {
    searchItems.value = []
    return
  }
  loading.value = true
  try {
    const res = await fetch(`https://imdb.iamidiotareyoutoo.com/search?q=${encodeURIComponent(title)}`)
    const data: MovieSearchResult = await res.json()
    searchItems.value = data?.description ?? []
  } catch (err) {
    console.error(err)
    toast.error('Search failed')
    searchItems.value = []
  } finally {
    loading.value = false
  }
}

watch(movieName, (val) => {
  clearTimeout(debounceTimer)
  debounceTimer = window.setTimeout(() => performSearch(val), 350)
})

onMounted(() => performSearch(movieName.value))
onBeforeUnmount(() => clearTimeout(debounceTimer))

async function selectMovie(imdb_id: string) {
  try {
    const id = await update_media_imdb(props.movie.id, imdb_id)
    emit('updated', id)
  } catch (e: unknown) {
    toast.error(e instanceof Error ? e.message : 'Failed to set imdb')
  }
}
</script>
