<template>
  <!-- Main Container -->
  <main class="container mx-auto px-4 py-6">
    <!-- Movie filter controls -->
    <FilterMovies />

    <!-- Loading indicator -->
    <LoadingView v-if="isLoading" />

    <!-- Movie results grid -->
    <div v-else>
      <ResultsInfo :totalMovies="medias.length" :numberOfSearchedMovies="medias.length" />

      <!-- No movies found message -->
      <NotFoundMovies v-if="medias.length === 0" />

      <!-- Display movie cards in a flex -->
      <div v-else class="flex flex-wrap justify-center gap-2">
        <MediaCard v-for="media in medias" :key="media.id" :media="media" />
      </div>
    </div>
  </main>
</template>

<script setup lang="ts">
// --- Vue & toast ---
import { onMounted, ref, watch } from 'vue'
import { toast } from 'vue3-toastify'

// --- Components ---
import FilterMovies from '../component/FilterMovies.vue'
import LoadingView from '../component/LoadingView.vue'
import ResultsInfo from '../component/ResultsInfo.vue'
import NotFoundMovies from '../component/NotFoundMovies.vue'
import MediaCard from '../component/MediaCard.vue'

// --- Stores ---
import { useMediasStore } from '../stores/medias'
import { useFiltersStore } from '../stores/Filters'
import { storeToRefs } from 'pinia'

// --- State ---
const isLoading = ref(true)
const mediasStore = useMediasStore()
const { medias } = storeToRefs(mediasStore)
const filtersStore = useFiltersStore()
const { filters } = storeToRefs(filtersStore)

// --- Lifecycle: initial load ---
onMounted(async () => await fetchMovies())

watch(filters, async () => await fetchMovies(), { deep: true })

async function fetchMovies() {
  isLoading.value = true
  try {
    await mediasStore.reload()
  } catch (error) {
    toast.error(`Failed to reload movies: ${error}`)
  } finally {
    isLoading.value = false
  }
}
</script>
