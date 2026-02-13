<template>
  <!-- Main Container -->
  <main class="container mx-auto px-4 py-6" ref="scrollContainer">
    <!-- Movie filter controls -->
    <FilterMedias />

    <!-- Loading indicator -->
    <LoadingView v-if="isLoading" />

    <!-- Movie results grid -->
    <div v-else>
      <ResultsInfo v-model="isShowCard" />

      <!-- No movies found message -->
      <NotFoundMedias v-if="medias.length === 0" />

      <!-- Display movie cards -->
      <template v-else>
        <main v-if="isShowCard" class="flex flex-wrap justify-center gap-2">
          <MediaCard v-for="media in medias" :key="media.id" :media="media" />
        </main>
        <TransitionGroup name="list" tag="ul" v-else class="list bg-base-100 rounded-box shadow-md">
          <MediaList v-for="media in medias" :key="media.id" :media="media" @update:media="updateMedia" />
        </TransitionGroup>
      </template>
    </div>
    <!-- Infinite scroll loading indicator -->
    <div ref="loadMoreRef"></div>
    <div v-if="isFetchingMore" class="py-4 text-center">
      Loading more movies<span class="loading loading-dots loading-sm ml-1"></span>
    </div>
  </main>
</template>

<script setup lang="ts">
// --- Vue & toast ---
import { onMounted, ref, watch, onBeforeUnmount } from 'vue'

// --- Components ---
import FilterMedias from '@/component/home_page/FilterMedias.vue'
import LoadingView from '@/component/home_page/LoadingView.vue'
import ResultsInfo from '@/component/home_page/ResultsInfo.vue'
import NotFoundMedias from '@/component/home_page/NotFoundMedias.vue'
import MediaCard from '@/component/home_page/MediaCard.vue'
import MediaList from '@/component/home_page/MediaList.vue'

// --- Stores ---
import { useMediasStore } from '@/stores/medias'
import { useFiltersStore } from '@/stores/Filters'
import { storeToRefs } from 'pinia'
import { handleFrontendError } from '@/functions/errorHandling'

// --- Types ---
import type { Media } from '../type'

// --- State ---
const isLoading = ref(true)
const isFetchingMore = ref(false)
const mediasStore = useMediasStore()
const { medias } = storeToRefs(mediasStore)
const filtersStore = useFiltersStore()
const { filters } = storeToRefs(filtersStore)
const isShowCard = ref(false)

const loadMoreRef = ref(null)
let observer: null | IntersectionObserver = null

// --- Lifecycle: initial load ---
onMounted(async () => {
  await fetchMovies()

  observer = new IntersectionObserver((entries) => {
    if (entries && entries[0]?.isIntersecting) {
      handleScroll()
    }
  })

  if (loadMoreRef.value) {
    observer.observe(loadMoreRef.value)
  }
})

onBeforeUnmount(() => {
  if (observer && loadMoreRef.value) {
    observer.unobserve(loadMoreRef.value)
  }
})

watch(filters, async () => await fetchMovies(), { deep: true })

async function fetchMovies() {
  isLoading.value = true
  try {
    await mediasStore.reload()
  } catch (error) {
    handleFrontendError('home.fetch_movies', error, 'Failed to reload movies')
  } finally {
    isLoading.value = false
  }
}

// --- Infinite scroll handler ---
async function handleScroll() {
  const bottomOfWindow = window.innerHeight + window.scrollY >= document.body.offsetHeight - 100

  if (bottomOfWindow && !isFetchingMore.value && !isLoading.value) {
    isFetchingMore.value = true
    try {
      await mediasStore.get_next_page()
      await new Promise((r) => setTimeout(r, 400))
    } catch (error) {
      handleFrontendError('home.load_more_movies', error, 'Failed to load more movies')
    } finally {
      isFetchingMore.value = false
    }
  }
}

function updateMedia(updatedMedia: Media) {
  // Find and update the media in the store
  const index = medias.value.findIndex((m) => m.id === updatedMedia.id)

  if (index !== -1) {
    if (
      (filters.value.watched != null && filters.value.watched != updatedMedia.watched) ||
      (filters.value.watchList != null && filters.value.watchList != updatedMedia.watch_list)
    )
      medias.value.splice(index, 1)
    else medias.value[index] = updatedMedia
  }
}
</script>

<style lang="css" scoped>
.list-enter-active,
.list-leave-active {
  transition: all 0.5s ease;
}
.list-enter-from,
.list-leave-to {
  opacity: 0;
  transform: translateY(-30px);
}
</style>
