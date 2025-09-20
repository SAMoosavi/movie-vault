<template>
  <!-- Files Section Card -->
  <div class="card from-primary/40 to-secondary/40 mb-8 bg-gradient-to-br p-0.5 shadow-xl">
    <div class="card bg-base-100 shadow-lg">
      <div class="card-body">
        <!-- Title Section -->
        <h2 class="card-title mb-4 flex items-center gap-2">
          <FileText class="text-primary h-6 w-6" />
          <span>Available Files</span>
          <div class="badge badge-secondary">{{ fileCount }} files</div>
        </h2>

        <div class="border-base-200 rounded-lg border shadow-sm">
          <!-- If Media Files Exist -->
          <template v-if="hasMediaFiles">
            <FileRow v-for="file in media.files" :key="file.path" :file="file" @reload="fetchMedia" />
          </template>

          <!-- If Series (Seasons) Exist -->
          <template v-else>
            <div v-for="season in media.seasons" :key="season.id" class="border-base-300 border-t">
              <!-- Season Accordion -->
              <div class="collapse-arrow bg-base-200 border-primary/30 collapse rounded-none border">
                <input type="checkbox" />
                <div class="collapse-title flex items-center justify-between font-semibold">
                  <span>Season {{ season.number }}</span>
                  <button
                    class="z-10 flex cursor-pointer items-center gap-2"
                    @click="setWatchedSeason(season.id, !season.watched)"
                  >
                    <div v-if="season.watched" class="badge badge-lg badge-success gap-1">
                      <Eye class="h-4 w-4" />
                      <span>Watched</span>
                    </div>
                    <div v-else class="badge badge-lg badge-outline gap-1">
                      <EyeOff class="h-4 w-4" />
                      <span>Not Watched</span>
                    </div>
                  </button>
                </div>
                <div class="collapse-content bg-base-100 p-0">
                  <!-- Episodes List -->
                  <div v-for="episode in season.episodes" :key="episode.id" class="border-base-300 border-t">
                    <!-- Episode Accordion -->
                    <div class="collapse-arrow border-secondary/30 collapse rounded-none border-t">
                      <input type="checkbox" />
                      <div class="collapse-title flex items-center justify-between text-sm font-medium">
                        <span>Episode {{ episode.number }}</span>
                        <button
                          class="z-10 flex cursor-pointer items-center gap-2"
                          @click="setWatchedEpisode(episode.id, !episode.watched)"
                        >
                          <div v-if="episode.watched" class="badge badge-lg badge-success gap-1">
                            <Eye class="h-4 w-4" />
                            <span>Watched</span>
                          </div>
                          <div v-else class="badge badge-lg badge-outline gap-1">
                            <EyeOff class="h-4 w-4" />
                            <span>Not Watched</span>
                          </div>
                        </button>
                      </div>
                      <div class="collapse-content p-0">
                        <!-- Episode Files -->
                        <FileRow
                          v-for="file in episode.files"
                          :key="file.path"
                          :file="file"
                          @reload="fetchMedia"
                          class="border-accent/30 rounded-none border-t"
                        />
                      </div>
                    </div>
                  </div>
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
// --- External Libraries ---
import { computed } from 'vue'
import { FileText, Eye, EyeOff } from 'lucide-vue-next'

// --- Local Components ---
import FileRow from './FileRow.vue'

// --- Types ---
import type { Media } from '../../type'
import { update_episode_watched, update_season_watched } from '../../functions/invoker'

// --- Props definition ---
interface Props {
  media: Media
}
const props = defineProps<Props>()

// --- Emits for watched toggling ---
const emit = defineEmits<{
  (e: 'fetch-media'): void
}>()

function fetchMedia() {
  emit('fetch-media')
}

async function setWatchedEpisode(episodeId: number, newState: boolean) {
  await update_episode_watched(episodeId, newState)
  fetchMedia()
}

async function setWatchedSeason(seasonId: number, newState: boolean) {
  await update_season_watched(seasonId, newState)
  fetchMedia()
}

// --- Helper: Check if media has files ---
const hasMediaFiles = computed(() => Array.isArray(props.media.files) && props.media.files.length > 0)

// --- Helper: Total file count (media or series) ---
const fileCount = computed(() => {
  if (hasMediaFiles.value) {
    return props.media.files.length
  }
  if (Array.isArray(props.media.seasons)) {
    return props.media.seasons.reduce((seasonTotal, season) => {
      return seasonTotal + season.episodes.reduce((episodeTotal, episode) => episodeTotal + episode.files.length, 0)
    }, 0)
  }
  return 0
})
</script>
