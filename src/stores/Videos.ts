import { get_all_video_metadata, search_videos } from '../functions/invoker'
import type { FilterValues, VideoMetaData } from '../type'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

export const useVideosStore = defineStore('videos', () => {
  const videos_metadata = ref<VideoMetaData[]>([])

  async function updata() {
    videos_metadata.value = await get_all_video_metadata()
  }

  async function search(filters: FilterValues) {
    videos_metadata.value = await search_videos(filters)
  }

  const number_of_videos = computed(() => videos_metadata.value.length)

  return { videos_metadata, updata, number_of_videos, search }
})
