<template>
  <div class="card from-primary/50 to-secondary/50 mb-8 bg-gradient-to-br p-0.5 shadow-xl">
    <div class="card bg-base-100 p-5">
      <!-- Header -->
      <div class="flex flex-wrap items-center gap-4">
        <label class="label flex gap-2">
          <span class="label-text font-semibold">Add Media Manually</span>
        </label>
      </div>

      <!-- IMDb ID Input -->
      <div class="join mt-4 w-full">
        <input
          v-model="imdbId"
          type="text"
          placeholder="Enter IMDb ID (e.g., tt0111161)"
          class="join-item input input-bordered w-full"
          @keyup.enter="addMedia"
        />
        <button class="btn btn-primary join-item" @click="addMedia" :disabled="loading">
          <PlusIcon v-if="!loading" class="h-4 w-4" />
          <span v-if="loading" class="loading loading-spinner loading-sm"></span>
          Add
        </button>
      </div>

      <!-- Search Section -->
      <div class="divider">OR</div>

      <!-- Search Input -->
      <div class="join mt-4 w-full">
        <input
          v-model="mediaName"
          type="text"
          placeholder="Search media name..."
          class="join-item input input-bordered w-full"
        />
      </div>

      <!-- Results -->
      <div v-if="loadingSearch || searchItems.length" class="mt-8">
        <h3 class="mb-4 text-xl font-semibold">Search Results</h3>

        <div class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
          <!-- Skeletons -->
          <template v-if="loadingSearch">
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
              :key="item.id"
              class="card from-primary/50 to-secondary/50 cursor-pointer bg-gradient-to-br p-0.5 shadow-xl transition hover:scale-[1.02] hover:shadow-2xl"
              @click="selectMedia(item.id)"
            >
              <div class="card card-compact bg-base-100 h-full w-full overflow-hidden shadow-lg">
                <figure class="relative h-full">
                  <img
                    :src="
                      item.primaryImage  ? item.primaryImage.url : 'https://placehold.co/300x450?text=No+Image'
                    "
                    :alt="item.primaryTitle || item.originalTitle"
                    class="h-full w-full object-cover"
                    loading="lazy"
                  />

                  <!-- Year -->
                  <div class="badge badge-primary absolute top-2 right-2 flex items-center gap-1 text-xs">
                    <CalendarIcon class="h-3 w-3" />
                    <span>{{ item.startYear }}</span>
                  </div>

                  <!-- Title -->
                  <div
                    class="bg-secondary text-secondary-content card absolute bottom-2 left-1/2 max-w-3/4 -translate-x-1/2 p-1 text-center text-xs text-wrap"
                  >
                    {{ item.primaryTitle || item.originalTitle }}
                  </div>

                  <div class="badge badge-primary absolute top-2 left-2 flex items-center gap-1 text-xs">
                    <StarIcon class="h-3 w-3" />
                    <span>{{ item.rating?.aggregateRating }}</span>
                  </div>
                </figure>
              </div>
            </div>
          </template>
        </div>
      </div>

      <!-- No Results -->
      <div v-else-if="mediaName" class="py-12 text-center">
        <SearchX class="mx-auto mb-4 h-16 w-16 opacity-60" />
        <p class="text-base-content/70">No medias found for "{{ mediaName }}"</p>
      </div>

      <!-- Empty State -->
      <div v-else class="py-16 text-center">
        <Search class="mx-auto mb-4 h-16 w-16 opacity-60" />
        <p class="text-base-content/70">Enter a media name to search or IMDb ID to add directly</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onBeforeUnmount } from 'vue'
import { fetch } from '@tauri-apps/plugin-http'
import { toast } from 'vue3-toastify'
import { SearchX, Search, CalendarIcon, PlusIcon, StarIcon } from 'lucide-vue-next'
import { create_media_from_imdb } from '../functions/invoker'
import type { MediaSearchResult, SearchedMedia } from './media_page/SearchMediaImdb'
import { useRouter } from 'vue-router'

const imdbId = ref('')
const mediaName = ref('')
const searchItems = ref<SearchedMedia[]>([])
const loading = ref(false)
const loadingSearch = ref(false)
const router = useRouter()

let debounceTimer: number | undefined

async function performSearch(query: string) {
  const title = query.trim()
  if (!title) {
    searchItems.value = []
    return
  }
  loadingSearch.value = true
  try {
    const res = await fetch(`https://api.imdbapi.dev/search/titles?query=${encodeURIComponent(title)}`)
    const data: MediaSearchResult = await res.json()
    searchItems.value = data?.titles ?? []
  } catch (err) {
    console.error(err)
    toast.error('Search failed')
    searchItems.value = []
  } finally {
    loadingSearch.value = false
  }
}

watch(mediaName, (val) => {
  clearTimeout(debounceTimer)
  debounceTimer = window.setTimeout(() => performSearch(val), 350)
})

onBeforeUnmount(() => clearTimeout(debounceTimer))

async function addMedia() {
  const id = imdbId.value.trim()
  if (!id) {
    toast.error('Please enter an IMDb ID')
    return
  }

  loading.value = true
  try {
    const media_id = await create_media_from_imdb(id)
    toast.success('Media added successfully!')
    imdbId.value = ''
    mediaName.value = ''
    await router.push({name:'media_page', params:{id:media_id}})
  } catch (e: unknown) {
    toast.error(e instanceof Error ? e.message : 'Failed to add media')
    console.error(e)
  } finally {
    loading.value = false
  }
}

async function selectMedia(imdb_id: string) {
  imdbId.value = imdb_id
  await addMedia()
}
</script>
