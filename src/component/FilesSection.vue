<template>
  <div class="card bg-base-100 shadow-xl">
    <div class="card-body">
      <h2 class="card-title mb-4 flex items-center gap-2">
        <FileText class="h-6 w-6" />
        <span>Available Files</span>
        <div class="badge badge-secondary">{{ totalFileCount }} files</div>
      </h2>

      <div class="border-base-200 overflow-hidden rounded-lg border shadow-sm">
        <!-- Header -->
        <div class="bg-primary text-primary-content hidden p-3 text-center font-bold sm:grid sm:grid-cols-4">
          <div>File Details</div>
          <div>Quality</div>
          <div>Language Format</div>
          <div>Actions</div>
        </div>

        <!-- Movie Files -->
        <template v-if="movie.files?.length">
          <FileRow v-for="file in movie.files" :key="file.path" :file="file" />
        </template>

        <!-- Series -->
        <template v-else>
          <div v-for="season in movie.seasons" :key="season.id" class="border-base-300 border-t">
            <!-- Season Accordion -->
            <div class="collapse-arrow bg-base-200 collapse rounded-none">
              <input type="checkbox" />
              <div class="collapse-title flex items-center justify-between font-semibold">
                <span class="font-semibold"> Season {{ season.number }} </span>
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
              <div class="collapse-content bg-base-100">
                <div v-for="episode in season.episodes" :key="episode.id" class="border-base-300 border-t">
                  <!-- Episode Accordion -->
                  <div class="collapse-arrow collapse">
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
                    <div class="collapse-content">
                      <FileRow v-for="file in episode.files" :key="file.path" :file="file" />
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
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { FileText, Eye, EyeOff } from 'lucide-vue-next'
import FileRow from './FileRow.vue'
import type { Media } from '../type'

interface Props {
  movie: Media
}
const props = defineProps<Props>()

defineEmits(['set-watched-season', 'set-watched-episode'])

const totalFileCount = computed(() => {
  if (props.movie.files?.length) return props.movie.files.length
  if (props.movie.seasons?.length) {
    return props.movie.seasons.reduce((seasonAcc, s) => {
      return seasonAcc + s.episodes.reduce((epAcc, e) => epAcc + e.files.length, 0)
    }, 0)
  }
  return 0
})
</script>
