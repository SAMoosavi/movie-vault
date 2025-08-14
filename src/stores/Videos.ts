import { filter_medias } from '../functions/invoker'
import type { Media } from '../type'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { useFiltersStore } from './Filters'

export const useVideosStore = defineStore('videos', () => {
  const filterStore = useFiltersStore()

  const videos_metadata = ref<Media[]>([])

  async function updata() {
    videos_metadata.value = await filter_medias(filterStore.filters)
  }

  const number_of_videos = computed(() => videos_metadata.value.length)

  return { videos_metadata, updata, number_of_videos }
})
