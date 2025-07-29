<template>
  <div v-if="!movie">
    loading...
  </div>
  <div v-else class="min-h-screen bg-base-200 p-4 md:p-8">
    <div class="max-w-6xl mx-auto">
      <!-- Back Button -->
      <button @click="$router.back()" class="btn btn-ghost mb-6">
        <ArrowLeft class="h-5 w-5 mr-2" />
        Back to Movies
      </button>

      <!-- Movie Header -->
      <div class="card from-primary/50 to-secondary/50 mb-8 bg-gradient-to-br p-0.5  shadow-xl ">
        <div class="card bg-base-100">
          <div class="card-body">
            <div class="flex flex-col md:flex-row gap-8">
              <!-- Poster -->
              <div class="flex-shrink-0">
                <div class="w-64 rounded-lg shadow-lg">
                  <img :src="movie.imdb_metadata?.poster" :alt="movie.imdb_metadata?.title" class="object-cover" />
                </div>
              </div>

              <!-- Movie Info -->
              <div class="flex-grow">
                <h1 class="text-3xl md:text-4xl font-bold mb-2">
                  {{ movie.imdb_metadata?.title }}
                  <span class="text-2xl text-base-content/70">({{ movie.imdb_metadata?.year }})</span>
                </h1>

                <!-- Rating and Meta Info -->
                <div class="flex flex-wrap items-center gap-4 mb-4">
                  <div class="badge badge-lg badge-warning gap-1">
                    <Star class="h-5 w-5 fill-current" />
                    <span class="font-bold">{{ movie.imdb_metadata?.imdb_rating }}</span>
                    <span class="text-xs">/10</span>
                  </div>

                  <div class="text-sm">
                    <span class="badge badge-outline mr-2">{{ movie.imdb_metadata?.rated }}</span>
                    <span>{{ movie.imdb_metadata?.runtime }}</span>
                  </div>

                  <div class="text-sm">
                    {{ movie.imdb_metadata?.released }}
                  </div>
                </div>

                <!-- Genres -->
                <div class="flex flex-wrap gap-2 mb-4">
                  <span v-for="genre in movie.imdb_metadata?.genre" :key="genre" class="badge badge-primary badge-md">
                    {{ genre }}
                  </span>
                </div>

                <!-- Plot -->
                <p class="text-base-content/90 mb-6">
                  {{ movie.imdb_metadata?.plot }}
                </p>

                <!-- Cast and Crew -->
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
                  <div>
                    <h3 class="font-semibold text-base-content/70 mb-1">Directors</h3>
                    <p>{{ movie.imdb_metadata?.directors?.join(', ') }}</p>
                  </div>
                  <div>
                    <h3 class="font-semibold text-base-content/70 mb-1">Writers</h3>
                    <p>{{ movie.imdb_metadata?.writers?.join(', ') }}</p>
                  </div>
                  <div>
                    <h3 class="font-semibold text-base-content/70 mb-1">Stars</h3>
                    <p>{{ movie.imdb_metadata?.actors?.join(', ') }}</p>
                  </div>
                  <div>
                    <h3 class="font-semibold text-base-content/70 mb-1">Languages</h3>
                    <p>{{ movie.imdb_metadata?.languages?.join(', ') }}</p>
                  </div>
                </div>

                <!-- Additional Info -->
                <div class="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
                  <div>
                    <h3 class="font-semibold text-base-content/70 mb-1">Country</h3>
                    <p>{{ movie.imdb_metadata?.country?.join(', ') }}</p>
                  </div>
                  <div>
                    <h3 class="font-semibold text-base-content/70 mb-1">Box Office</h3>
                    <p>{{ movie.imdb_metadata?.box_office || 'N/A' }}</p>
                  </div>
                  <div>
                    <h3 class="font-semibold text-base-content/70 mb-1">Awards</h3>
                    <p>{{ movie.imdb_metadata?.awards }}</p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Files Section -->
      <div class="card bg-base-100 shadow-xl">
        <div class="card-body">
          <h2 class="card-title mb-4">
            <FileText class="h-6 w-6 mr-2" />
            Available Files
            <div class="badge badge-secondary">{{ movie.files_data?.length }} files</div>
          </h2>

          <div class="overflow-x-auto">
            <table class="table table-zebra">
              <thead>
                <tr>
                  <th>Title</th>
                  <th>Quality</th>
                  <th>Subtitles</th>
                  <th>Dubbed</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(file, index) in movie.files_data" :key="index">
                  <td>
                    <div class="font-medium">{{ file.title }}</div>
                    <div class="text-sm text-base-content/70 truncate max-w-xs">{{ file.path }}</div>
                  </td>
                  <td>
                    <div class="badge badge-outline">{{ file.quality }}</div>
                  </td>
                  <td>
                    <div class="flex gap-1">
                      <div class="badge badge-sm" :class="file.has_soft_sub ? 'badge-success' : 'badge-ghost'">
                        Soft
                      </div>
                      <div class="badge badge-sm" :class="file.has_hard_sub ? 'badge-success' : 'badge-ghost'">
                        Hard
                      </div>
                    </div>
                  </td>
                  <td>
                    <div class="badge" :class="file.is_dubbed ? 'badge-primary' : 'badge-ghost'">
                      {{ file.is_dubbed ? 'Yes' : 'No' }}
                    </div>
                  </td>
                  <td>
                    <div class="flex gap-2">
                      <button class="btn btn-xs btn-primary" @click="playFile(file.path)">
                        Play
                      </button>
                      <button class="btn btn-xs btn-secondary" @click="openFileLocation(file.path)">
                        Location
                      </button>
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <div v-if="!movie.files_data || movie.files_data.length === 0" class="text-center py-8">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-12 w-12 mx-auto text-base-content/30 mb-4" fill="none"
              viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            <p class="text-base-content/70">No files available for this movie</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'

// Methods
function playFile(path: string) {
  // Implement play functionality
  console.log('Playing file:', path)
  // You can use Tauri's shell API to open the file with default player
  // import { open } from '@tauri-apps/api/shell'
  // open(path)
}

function openFileLocation(path: string) {
  // Implement open file location functionality
  console.log('Opening file location:', path)
  // You can use Tauri's shell API to open the directory
  // import { open } from '@tauri-apps/api/shell'
  // open(dirname(path))
}

import type { VideoMetaData } from '../type';
import { useRoute } from 'vue-router';
import { get_video_by_id } from '../functions/invoker';
import { ArrowLeft, Star, FileText } from 'lucide-vue-next';

const movie = ref<VideoMetaData>()

const route = useRoute()

onMounted(async () => {
  movie.value = await get_video_by_id(+route.params.id)
})
</script>
