<template>
  <!-- Main Container -->
  <div class="container mx-auto min-h-screen">
    <!-- Back Button -->
    <button @click="goBack" class="btn btn-ghost my-6">
      <ArrowLeft class="mr-2 h-5 w-5" />
      Back
    </button>
    <!-- Loading Skeletons -->
    <template v-if="!media">
      <MediaHeaderSkeleton />
      <FilesSectionSkeleton />
    </template>

    <!-- Media Content -->
    <template v-else>
      <!-- Media Header or Edit/Search -->
      <MediaHeader
        v-if="media.imdb && !isEditing"
        :media="media"
        @edit="startEditing"
        @toggle-watched="toggleWatched"
        @set-ranking="setRanking"
        @toggle-watch-list="toggleWatchList"
      />
      <SearchMediaImdb
        v-else
        :media="media"
        :is_editing="isEditing"
        @toggle-watched="toggleWatched"
        @set-ranking="setRanking"
        @cancel="cancelEditing"
        @toggle-watch-list="toggleWatchList"
        @updated="updated"
      />

      <!-- Files Section -->
      <FilesSection :media="media" @set-watched-episode="setWatchedEpisode" @set-watched-season="setWatchedSeason" />
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

import MediaHeader from '../component/media_page/MediaHeader.vue'
import SearchMediaImdb from '../component/media_page/SearchMediaImdb.vue'
import MediaHeaderSkeleton from '../component/media_page/MediaHeaderSkeleton.vue'
import FilesSectionSkeleton from '../component/media_page/FilesSectionSkeleton.vue'
import FilesSection from '../component/media_page/FilesSection.vue'

// --- State ---
const route = useRoute()
const router = useRouter()
const media = ref<Media | null>(null)
const isEditing = ref(false)

// --- Navigation ---
function goBack() {
  router.back()
}

// Fetch media data by ID (safer error handling)
async function fetchMedia(id: number = 0) {
  if (id !== 0) {
    // navigate to a new id, reset edit mode afterwards
    await router.push({ name: route.name, params: { id } })
    isEditing.value = false
    return
  }

  try {
    const data = await get_media_by_id(Number(route.params.id))
    media.value = data
  } catch (error) {
    toast.error(typeof error === 'string' ? error : error instanceof Error ? error.message : 'Failed to fetch media')
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

// --- Media actions ---
async function toggleWatched() {
  if (media.value) {
    await update_media_watched(media.value.id, !media.value.watched)
    fetchMedia()
  }
}
async function toggleWatchList() {
  if (media.value) {
    await update_media_watch_list(media.value.id, !media.value.watch_list)
    fetchMedia()
  }
}

async function setWatchedEpisode(episodeId: number, newState: boolean) {
  await update_episode_watched(episodeId, newState)
  fetchMedia()
}

async function setWatchedSeason(seasonId: number, newState: boolean) {
  await update_season_watched(seasonId, newState)
  fetchMedia()
}

async function setRanking(rank: number) {
  if (media.value) {
    await update_media_my_ranking(media.value.id, rank)
    fetchMedia()
  }
}

async function updated(id: number) {
  await router.push({ name: 'media_page', params: { id } })
  fetchMedia()
  cancelEditing()
}

onMounted(() => {
  fetchMedia()
})
</script>
