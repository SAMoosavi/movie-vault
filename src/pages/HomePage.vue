<template>
  <main class="container mx-auto px-4 py-6">
    <FilterMovies />

    <!-- Tag Manager -->
    <div class="mb-3 rounded-xl border-2 p-3">
      <div class="flex items-center mb-2 gap-3">
        <TagIcon class="h-6 w-6" />
        <h3 class="text-xl">tag manager</h3>
      </div>
      <div class="flex items-center">
        <!-- Add Tag -->
        <div class="flex basis-1/3 gap-3">
          <input class="input" v-model="tag.name" />
          <button class="btn" @click="addTag">add tag</button>
        </div>

        <!-- Show and remove tag -->
        <div class="flex flex-wrap gap-1">
          <span
            v-for="tag in tags"
            :key="tag.id"
            class="badge badge-outline badge-sm cursor-pointer"
            @click="removeTag(tag.id)"
          >
            {{ tag.name }}
          </span>
        </div>
      </div>
    </div>

    <LoadingView v-if="loading" />
    <!-- Movie Grid -->
    <div v-else>
      <ResultsInfo :totalMovies="videos_metadata.length" :numberOfSearchedMovies="videos_metadata.length" />

      <NotFoundMovies v-if="videos_metadata.length === 0" />
      <div v-else class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
        <MovieCard v-for="movie in videos_metadata" :key="movie.id" :movie="movie" />
      </div>
    </div>
  </main>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { toast } from 'vue3-toastify'
import FilterMovies from '../component/FilterMovies.vue'
import LoadingView from '../component/LoadingView.vue'
import ResultsInfo from '../component/ResultsInfo.vue'
import NotFoundMovies from '../component/NotFoundMovies.vue'
import MovieCard from '../component/MovieCard.vue'
import { useVideosStore } from '../stores/Videos'
import { storeToRefs } from 'pinia'
import { useFiltersStore } from '../stores/Filters'
import type { Tag } from '../type'
import { get_tags, insert_tag, remove_tag } from '../functions/invoker'
import { TagIcon } from 'lucide-vue-next'

const loading = ref(true)

const videos = useVideosStore()
const { videos_metadata } = storeToRefs(videos)

onMounted(async () => {
  await getTags()

  try {
    await videos.updata()

    toast.success('Data loaded successfully!')
  } catch (e) {
    console.error('Data fetching error:', e)
    toast.error(`Failed to load data: ${e instanceof Error ? e.message : 'Unknown error'}`)
  } finally {
    loading.value = false
  }
})

const filtersStore = useFiltersStore()

const { filters } = storeToRefs(filtersStore)

// Watch and emit on change
watch(filters, () => search(), { deep: true })

async function search() {
  loading.value = true
  videos
    .updata()
    .then(() => {})
    .catch((e) => toast.error(e))
    .finally(() => (loading.value = false))
}

const tags = ref<Tag[]>([])
const tag = ref<Tag>({ id: 0, name: '' })

async function getTags() {
  tags.value = await get_tags()
}

async function addTag() {
  await insert_tag(tag.value)
  tag.value = { id: 0, name: '' }
  await getTags()
}

async function removeTag(tag_id: number) {
  await remove_tag(tag_id)
  await getTags()
}
</script>
