<template>
  <!-- Main Container -->
  <div class="container mx-auto min-h-screen">
    <!-- Back Button -->
    <button @click="goBack" class="btn btn-ghost my-6">
      <ArrowLeft class="mr-2 h-5 w-5" />
      Back
    </button>
    <!-- Loading Skeletons -->
    <template v-if="!movie">
      <MovieHeaderSkeleton />
      <FilesSectionSkeleton />
    </template>

    <!-- Movie Content -->
    <template v-else>
      <!-- Movie Header or Edit/Search -->
      <MovieHeader
        v-if="movie.imdb && !isEditing"
        :media="movie"
        @edit="startEditing"
        @toggle-watched="toggleWatched"
        @set-ranking="setRanking"
        @toggle-watch-list="toggleWatchList"
      />
      <SearchMovie
        v-else
        :media="movie"
        :is_editing="isEditing"
        @toggle-watched="toggleWatched"
        @set-ranking="setRanking"
        @cancel="cancelEditing"
        @toggle-watch-list="toggleWatchList"
        @updated="updated"
      />

      <!-- Files Section -->
      <FilesSection :movie="movie" @set-watched-episode="setWatchedEpisode" @set-watched-season="setWatchedSeason" />
    </template>
  </div>
</template>

<script setup lang="ts">
// --- External ---
import { ref, onMounted } from 'vue'
import { toast } from 'vue3-toastify'

// --- Routing & types ---
import { useRouter, useRoute } from 'vue-router'
import type { Media } from '../type'

// --- Functions & components ---
import {
  get_media_by_id,
  update_episode_watched,
  update_media_my_ranking,
  update_media_watch_list,
  update_media_watched,
  update_season_watched,
} from '../functions/invoker'
import { ArrowLeft } from 'lucide-vue-next'
import MovieHeader from '../component/MovieHeader.vue'
import SearchMovie from '../component/SearchMovie.vue'
import FilesSection from '../component/FilesSection.vue'
import MovieHeaderSkeleton from '../component/MovieHeaderSkeleton.vue'
import FilesSectionSkeleton from '../component/FilesSectionSkeleton.vue'

// --- State ---
const route = useRoute()
const router = useRouter()
const movie = ref<Media | null>(null)
const isEditing = ref(false)

// --- Navigation ---
function goBack() {
  router.back()
}

// Fetch movie data by ID (safer error handling)
async function fetchMovie(id: number = 0) {
  if (id !== 0) {
    // navigate to a new id, reset edit mode afterwards
    await router.push({ name: route.name, params: { id } })
    isEditing.value = false
    return
  }

  try {
    const data = await get_media_by_id(Number(route.params.id))
    movie.value = data
  } catch (error) {
    toast.error(typeof error === 'string' ? error : error instanceof Error ? error.message : 'Failed to fetch movie')
    goBack()
  }
}

// --- Edit mode handlers ---
function startEditing() {
  isEditing.value = true
}

function cancelEditing() {
  isEditing.value = false
}

// --- Movie actions ---
async function toggleWatched() {
  if (movie.value) {
    await update_media_watched(movie.value.id, !movie.value.watched)
    fetchMovie()
  }
}
async function toggleWatchList() {
  if (movie.value) {
    await update_media_watch_list(movie.value.id, !movie.value.watch_list)
    fetchMovie()
  }
}

async function setWatchedEpisode(episodeId: number, newState: boolean) {
  await update_episode_watched(episodeId, newState)
  fetchMovie()
}

async function setWatchedSeason(seasonId: number, newState: boolean) {
  await update_season_watched(seasonId, newState)
  fetchMovie()
}

async function setRanking(rank: number) {
  if (movie.value) {
    await update_media_my_ranking(movie.value.id, rank)
    fetchMovie()
  }
}

async function updated(id: number) {
  await router.push({ name: 'movie_page', params: { id } })
  fetchMovie()
  cancelEditing()
}

onMounted(() => {
  fetchMovie()
})
</script>
