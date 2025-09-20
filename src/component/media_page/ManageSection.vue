<template>
  <div class="card from-primary/40 to-secondary/40 mb-8 bg-gradient-to-br p-0.5 shadow-xl">
    <div class="card bg-base-100 sticky top-8 overflow-hidden shadow-lg">
      <div class="card-body p-6">
        <h2 class="mb-4 flex items-center gap-2 text-2xl font-bold">
          <SettingsIcon class="text-primary h-6 w-6" />
          Manage
        </h2>

        <div class="space-y-3">
          <button class="btn btn-outline w-full justify-between" @click="$emit('toggle-editing')">
            <span>{{ isEditing ? 'Cancel edit IMDB' : 'Edit IMDB' }}</span>
            <component :is="isEditing ? XCircleIcon : PencilIcon" class="h-5 w-5" />
          </button>

          <button class="btn btn-outline w-full justify-between" @click="toggleWatchList">
            <span>{{ media.watch_list ? 'Remove from Watchlist' : 'Add to Watchlist' }}</span>
            <component :is="media.watch_list ? BookmarkMinusIcon : BookmarkPlusIcon" class="h-5 w-5" />
          </button>

          <button class="btn btn-outline w-full justify-between" @click="toggleWatched">
            <span>{{ media.watched ? 'Watched' : 'Not watched' }}</span>
            <component :is="media.watched ? EyeIcon : EyeOffIcon" class="h-5 w-5" />
          </button>

          <div class="divider my-4"></div>

          <div class="card card-xs border">
            <div class="card-body flex flex-row items-center justify-between px-4">
              <div class="text-lg font-bold">Your ranking</div>

              <div class="flex items-center gap-1">
                <StarIcon
                  v-for="i in 5"
                  :key="i"
                  class="text-warning h-5 w-5 transform cursor-pointer transition-colors duration-200 hover:scale-110"
                  :class="{ 'fill-warning': media.my_ranking >= i }"
                  @click="setRanking(i)"
                />
              </div>
            </div>
          </div>

          <template v-if="media.imdb">
            <div class="divider my-4"></div>

            <div class="card from-warning to-warning/40 mb-8 bg-gradient-to-br p-0.5 shadow-xl">
              <div class="card bg-base-100 p-4 text-center">
                <h3 class="mb-2 flex items-center justify-center gap-2 font-semibold">
                  <StarIcon class="text-warning h-5 w-5" />
                  IMDb Rating
                </h3>
                <div class="py-3">
                  <div class="text-warning mb-1 text-4xl font-bold">
                    {{ media.imdb?.imdb_rating || 'N/A' }}
                  </div>
                  <div class="text-base-content/70 text-sm">{{ media.imdb?.imdb_votes || '0' }} votes</div>
                </div>
              </div>
            </div>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  BookmarkMinusIcon,
  BookmarkPlusIcon,
  EyeIcon,
  EyeOffIcon,
  PencilIcon,
  SettingsIcon,
  StarIcon,
  XCircleIcon,
} from 'lucide-vue-next'
import type { Media } from '../../type'
import { update_media_my_ranking, update_media_watch_list, update_media_watched } from '../../functions/invoker'

const props = defineProps<{ media: Media; isEditing: boolean }>()
const emit = defineEmits<{
  (e: 'fetch-media'): void
  (e: 'toggle-editing'): void
}>()

function fetchMedia() {
  emit('fetch-media')
}

async function toggleWatched() {
  await update_media_watched(props.media.id, !props.media.watched)
  fetchMedia()
}
async function toggleWatchList() {
  await update_media_watch_list(props.media.id, !props.media.watch_list)
  fetchMedia()
}

async function setRanking(rank: number) {
  await update_media_my_ranking(props.media.id, rank)
  fetchMedia()
}
</script>
