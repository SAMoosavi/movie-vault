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
      <MovieHeader
        v-if="movie.imdb && !change"
        :movie="movie"
        @edit="() => (change = true)"
        @toggle-watched="toggle_media_watched"
        @set-ranking="set_ranking"
      />
      <SearchMovie v-else :movie="movie" :change="change" @cancel="() => (change = false)" @updated="get_movie" />

      <FilesSection
        :movie="movie"
        @set-watched-episode="set_watched_episode"
        @set-watched-season="set_watched_season"
      />
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import type { Media } from '../type'
import { useRoute } from 'vue-router'
import {
  get_media_by_id,
  update_episode_watched,
  update_media_my_ranking,
  update_media_watched,
  update_season_watched,
} from '../functions/invoker'
import { ArrowLeft, Hash } from 'lucide-vue-next'
import MovieHeader from '../component/MovieHeader.vue'
import SearchMovie from '../component/SearchMovie.vue'
import FilesSection from '../component/FilesSection.vue'
import MovieHeaderSkeleton from '../component/MovieHeaderSkeleton.vue'
import FilesSectionSkeleton from '../component/FilesSectionSkeleton.vue'
import { toast } from 'vue3-toastify'

const route = useRoute()

const movie = ref<Media>()
const change = ref(false)
const notFound = ref(false)

async function get_movie() {
  get_media_by_id(+route.params.id)
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

async function toggle_media_watched() {
  if (movie.value) await update_media_watched(movie.value.id, !movie.value.watched)
}

async function set_watched_episode(episode_id: number, new_state:boolean) {
  console.log(episode_id);

  await update_episode_watched(episode_id, new_state)
}

async function set_watched_season(season_id: number, new_state:boolean) {
  console.log(season_id);

  await update_season_watched(season_id, new_state)
}

async function set_ranking(rank: number) {
  if (movie.value) await update_media_my_ranking(movie.value.id, rank)
}
</script>
