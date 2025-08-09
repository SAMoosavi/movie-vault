import { search_videos } from '../functions/invoker'
import type { VideoMetaData } from '../type'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { useFiltersStore } from './Filters'

export const useVideosStore = defineStore('videos', () => {
  const filterStore = useFiltersStore()

  const videos_metadata = ref<VideoMetaData[]>([])

  async function updata() {
    videos_metadata.value = await search_videos(filterStore.filters)
  }

  const number_of_videos = computed(() => videos_metadata.value.length)

  return { videos_metadata, updata, number_of_videos }
})
