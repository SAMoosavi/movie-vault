<template>
  <div class="container mx-auto min-h-screen">
    <!-- Back Button -->
    <button @click="$router.back()" class="btn btn-ghost my-6">
      <ArrowLeft class="mr-2 h-5 w-5" />
      Back
    </button>

    <template v-if="notFound">
      <div class="py-12 text-center">
        <Hash class="text-error/80 mx-auto mb-4 h-16 w-16" />
        <h3 class="mb-2 text-xl font-semibold">No movie found</h3>
      </div>
    </template>
    <template v-else-if="!movie">
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
import { ref, onMounted, onUnmounted } from 'vue'
import type { VideoMetaData } from '../type'
import { useRoute } from 'vue-router'
import { get_video_by_id } from '../functions/invoker'
import { ArrowLeft, Hash } from 'lucide-vue-next'
import MovieHeader from '../component/MovieHeader.vue'
import SearchMovie from '../component/SearchMovie.vue'
import FilesSection from '../component/FilesSection.vue'
import MovieHeaderSkeleton from '../component/MovieHeaderSkeleton.vue'
import FilesSectionSkeleton from '../component/FilesSectionSkeleton.vue'
import { toast } from 'vue3-toastify'

const route = useRoute()

const movie = ref<VideoMetaData>()
const change = ref(false)
const notFound = ref(false)

async function get_movie() {
  get_video_by_id(+route.params.id)
    .then((data) => (movie.value = data))
    .catch((e) => {
      toast.error(e)
      notFound.value = true
    })
}

let interval = 0
onMounted(() => {
  interval = setInterval(get_movie, 1000)
})
onUnmounted(() => clearInterval(interval))
</script>
