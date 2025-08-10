<template>
  <div class="container min-h-screen mx-auto">
    <!-- Back Button -->
    <button @click="$router.back()" class="btn btn-ghost my-6">
      <ArrowLeft class="mr-2 h-5 w-5" />
      Back to Movies
    </button>

    <div v-if="!movie" class="flex h-64 items-center justify-center">
      <div class="loading loading-spinner loading-lg"></div>
    </div>
    <template v-else>
      <MovieHeader v-if="movie.imdb_metadata && !change" :movie="movie" @edit="() => change = true" />
      <SearchMovie v-else :movie="movie" :change="change" @cancel="() => change = false" @updated="get_movie" />

      <FilesSection :movie="movie" />
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import type { VideoMetaData } from '../type'
import { useRoute } from 'vue-router'
import { get_video_by_id } from '../functions/invoker'
import { ArrowLeft } from 'lucide-vue-next'

import MovieHeader from '../component/MovieHeader.vue'
import SearchMovie from '../component/SearchMovie.vue'
import FilesSection from '../component/FilesSection.vue'


const movie = ref<VideoMetaData>()

const route = useRoute()

async function get_movie() {
  movie.value = await get_video_by_id(+route.params.id)
}

onMounted(get_movie)

const change = ref(false)
</script>
