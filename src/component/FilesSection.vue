<template>
  <!-- Files Section Card -->
  <div class="card from-primary/40 to-secondary/40 mb-8 bg-gradient-to-br p-0.5 shadow-xl">
    <div class="card bg-base-100 overflow-hidden shadow-lg">
      <div class="card-body">
        <!-- Title Section -->
        <h2 class="card-title mb-4 flex items-center gap-2">
          <FileText class="text-primary h-6 w-6" />
          <span>Available Files</span>
          <div class="badge badge-secondary">{{ fileCount }} files</div>
        </h2>

        <div class="border-base-200 overflow-hidden rounded-lg border shadow-sm">
          <!-- If Movie Files Exist -->
          <template v-if="hasMovieFiles">
            <FileRow v-for="file in movie.files" :key="file.path" :file="file" />
          </template>

          <!-- If Series (Seasons) Exist -->
          <template v-else>
            <div v-for="season in movie.seasons" :key="season.id" class="border-base-300 border-t">
              <!-- Season Accordion -->
              <div class="collapse-arrow bg-base-200 collapse border overflow-hidden border-primary/30 rounded-none">
                <input type="checkbox" />
                <div class="collapse-title flex items-center justify-between font-semibold">
                  <span>Season {{ season.number }}</span>
                  <button
                    class="z-10 flex cursor-pointer items-center gap-2"
                    @click="$emit('set-watched-season', season.id, !season.watched)"
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
                <div class="collapse-content p-0 bg-base-100">
                  <!-- Episodes List -->
                  <div v-for="episode in season.episodes" :key="episode.id" class="border-base-300 border-t">
                    <!-- Episode Accordion -->
                    <div class="collapse-arrow collapse border-t rounded-none overflow-hidden border-secondary/30">
                      <input type="checkbox" />
                      <div class="collapse-title flex items-center justify-between text-sm font-medium">
                        <span>Episode {{ episode.number }}</span>
                        <button
                          class="z-10 flex cursor-pointer items-center gap-2"
                          @click="$emit('set-watched-episode', episode.id, !episode.watched)"
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
                        <FileRow v-for="file in episode.files" :key="file.path" :file="file" class="border-t rounded-none border-accent/30"/>
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
import type { Media } from '../type'

// --- Props definition ---
interface Props {
  movie: Media
}
const props = defineProps<Props>()

// --- Emits for watched toggling ---
defineEmits(['set-watched-season', 'set-watched-episode'])

// --- Helper: Check if movie has files ---
const hasMovieFiles = computed(() => Array.isArray(props.movie.files) && props.movie.files.length > 0)

// --- Helper: Total file count (movie or series) ---
const fileCount = computed(() => {
  if (hasMovieFiles.value) {
    return props.movie.files.length
  }
  if (Array.isArray(props.movie.seasons)) {
    return props.movie.seasons.reduce((seasonTotal, season) => {
      return seasonTotal + season.episodes.reduce((episodeTotal, episode) => episodeTotal + episode.files.length, 0)
    }, 0)
  }
  return 0
})
</script>
