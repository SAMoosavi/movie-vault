<template>
  <div class="container mx-auto min-h-screen">
    <!-- Back Button -->
    <button @click="$router.back()" class="btn btn-ghost my-6">
      <ArrowLeft class="mr-2 h-5 w-5" />
      Back to Movies
    </button>

    <template v-if="!movie">
      <MovieHeaderSkeleton />
      <FilesSectionSkeleton />
    </template>
    <template v-else>
      <MovieHeader v-if="movie.imdb_metadata && !change" :movie="movie" @edit="() => (change = true)" />
      <SearchMovie v-else :movie="movie" :change="change" @cancel="() => (change = false)" @updated="get_movie" />

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
import MovieHeaderSkeleton from '../component/MovieHeaderSkeleton.vue'
import FilesSectionSkeleton from '../component/FilesSectionSkeleton.vue'

const route = useRoute()

const movie = ref<VideoMetaData>()
const change = ref(false)

async function get_movie() {
  movie.value = await get_video_by_id(+route.params.id)
}

onMounted(get_movie)
</script>
