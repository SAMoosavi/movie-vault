<template>
  <li class="list-row flex cursor-pointer items-center" @click="got_to_media_page">
    <div>
      <div class="w-12 overflow-hidden rounded-2xl">
        <img :src="media.imdb?.poster || 'default-image.jpg'" alt="Movie Poster" />
      </div>
    </div>
    <div class="w-fit xl:w-96">
      <div class="text-lg font-bold sm:text-2xl">
        <span>
          {{ media.imdb?.title || media.name }}
        </span>
        <span class="opacity-30"> ({{ media.imdb?.year || media.year }}) </span>
      </div>
    </div>
    <div class="hidden max-w-fit overflow-hidden text-nowrap xl:block">
      <div class="text-sm">
        <span class="text-primary font-bold">Genre: </span>
        <span class="opacity-50">
          {{ media.imdb?.genres?.join(', ') }}
        </span>
      </div>

      <div class="text-sm">
        <span class="text-primary font-bold">Cast: </span>
        <span class="opacity-50">
          {{ media.imdb?.actors?.map((a) => a.name).join(', ') }}
        </span>
      </div>
    </div>

    <div class="ml-auto hidden sm:block">
      <div class="flex items-center justify-end gap-2">
        {{ media.imdb?.imdb_rating }}
        <StarIcon class="material-icons mr-1 text-yellow-500" />
      </div>
      <div class="flex items-center justify-end gap-2">
        {{ media.tags?.map((a) => a.name).join(', ') }}
        <TagsIcon class="text-primary size-6" />
      </div>
    </div>
    <div class="hidden sm:block">
      <div>
        <BookmarkIcon class="text-accent size-6" :class="media.watch_list && 'fill-accent'" />
      </div>
      <component :is="media.watched ? EyeIcon : EyeClosedIcon" class="text-accent size-6" />
    </div>
  </li>
</template>

<script setup lang="ts">
import { BookmarkIcon, EyeClosedIcon, EyeIcon, StarIcon, TagsIcon } from 'lucide-vue-next'
import type { Media } from '../../type'
import { useRouter } from 'vue-router'

const props = defineProps<{ media: Media }>()
const router = useRouter()

function got_to_media_page() {
  router.push({ name: 'media_page', params: { id: props.media.id.toString() } })
}
</script>
