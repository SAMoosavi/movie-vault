<template>
  <div v-if="!movie" class="flex justify-center items-center h-64">
    <div class="loading loading-spinner loading-lg"></div>
  </div>
  <div v-else class="bg-base-200 min-h-screen p-4 md:p-8">
    <div class="mx-auto max-w-6xl">
      <!-- Back Button -->
      <button @click="$router.back()" class="btn btn-ghost mb-6">
        <ArrowLeft class="mr-2 h-5 w-5" />
        Back to Movies
      </button>

      <!-- Movie Header -->
      <div class="card from-primary/50 to-secondary/50 mb-8 bg-gradient-to-br p-0.5 shadow-xl">
        <div class="card bg-base-100">
          <div class="card-body">
            <div v-if="movie.imdb_metadata && !change" class="flex flex-col gap-8 md:flex-row">
              <!-- Poster -->
              <div class="flex-shrink-0">
                <div class="w-64 rounded-lg shadow-lg">
                  <img :src="movie.imdb_metadata?.poster" :alt="movie.imdb_metadata?.title" class="object-cover" />
                </div>
              </div>

              <!-- Movie Info -->
              <div class="flex-grow">
                <div class="flex justify-between items-center">
                  <h1 class="mb-2 text-3xl font-bold md:text-4xl">
                    {{ movie.imdb_metadata?.title }}
                    <span class="text-base-content/70 text-2xl">({{ movie.imdb_metadata?.year }})</span>
                  </h1>

                  <button class="btn btn-primary" @click="() => change = true"> change imdb data</button>
                </div>
                <!-- Rating and Meta Info -->
                <div class="mb-4 flex flex-wrap items-center gap-4">
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
                <div class="mb-4 flex flex-wrap gap-2">
                  <span v-for="genre in movie.imdb_metadata?.genre" :key="genre" class="badge badge-primary badge-md">
                    {{ genre }}
                  </span>
                </div>

                <!-- Plot -->
                <p class="text-base-content/90 mb-6">
                  {{ movie.imdb_metadata?.plot }}
                </p>

                <!-- Cast and Crew -->
                <div class="mb-6 grid grid-cols-1 gap-4 md:grid-cols-2">
                  <div>
                    <h3 class="text-base-content/70 mb-1 font-semibold">Directors</h3>
                    <p>{{ movie.imdb_metadata?.directors?.join(', ') }}</p>
                  </div>
                  <div>
                    <h3 class="text-base-content/70 mb-1 font-semibold">Writers</h3>
                    <p>{{ movie.imdb_metadata?.writers?.join(', ') }}</p>
                  </div>
                  <div>
                    <h3 class="text-base-content/70 mb-1 font-semibold">Stars</h3>
                    <p>{{ movie.imdb_metadata?.actors?.join(', ') }}</p>
                  </div>
                  <div>
                    <h3 class="text-base-content/70 mb-1 font-semibold">Languages</h3>
                    <p>{{ movie.imdb_metadata?.languages?.join(', ') }}</p>
                  </div>
                </div>

                <!-- Additional Info -->
                <div class="grid grid-cols-1 gap-4 text-sm md:grid-cols-3">
                  <div>
                    <h3 class="text-base-content/70 mb-1 font-semibold">Country</h3>
                    <p>{{ movie.imdb_metadata?.country?.join(', ') }}</p>
                  </div>
                  <div>
                    <h3 class="text-base-content/70 mb-1 font-semibold">Box Office</h3>
                    <p>{{ movie.imdb_metadata?.box_office || 'N/A' }}</p>
                  </div>
                  <div>
                    <h3 class="text-base-content/70 mb-1 font-semibold">Awards</h3>
                    <p>{{ movie.imdb_metadata?.awards }}</p>
                  </div>
                </div>
              </div>
            </div>

            <template v-else>
              <div class="form-control">
                <label class="label">
                  <span class="label-text font-semibold">Search Imdb of {{ movie.name }}</span> <span v-if="movie.year">
                    ({{ movie.year }})</span>
                </label>
                <div class="join w-full mt-3">
                  <input v-model="movieName" type="text" placeholder="Enter movie name..."
                    class="join-item input input-bordered w-full" />
                  <button v-if="change" class=" join-item btn btn-primary" @click="() => change = false">
                    cancel change
                  </button>
                </div>
              </div>

              <!-- Search Results -->
              <div v-if="loading" class="mt-6">
                <h3 class="text-xl font-semibold mb-4">Search Results</h3>
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                  <div v-for="i in 6" :key="i"
                    class="card from-primary/50 to-secondary/50 mb-8 bg-gradient-to-br p-0.5 shadow-xl">
                    <div
                      class="card card-compact bg-base-100 shadow-lg hover:shadow-xl transition-shadow cursor-pointer">
                      <figure class="h-48 bg-base-200 rounded-t-2xl">
                        <div class="w-full h-full bg-base-300 rounded-t-2xl skeleton"></div>
                      </figure>
                      <div class="card-body">
                        <div class="h-6 bg-base-300 rounded w-3/4 mb-2 skeleton"></div>
                        <div class="h-4 bg-base-300 rounded w-1/2 skeleton"></div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
              <div v-else-if="searchItems.length > 0" class="mt-6">
                <h3 class="text-xl font-semibold mb-4">Search Results</h3>
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">

                  <div v-for="item in searchItems" :key="item['#IMDB_ID']"
                    class="card from-primary/50 to-secondary/50 mb-8 bg-gradient-to-br p-0.5 shadow-xl transition-all duration-200 hover:scale-105">
                    <div
                      class="card card-compact  bg-base-100 shadow-lg hover:shadow-xl transition-shadow cursor-pointer"
                      @click="selectMovie(item['#IMDB_ID'])">
                      <figure class="h-48">
                        <img
                          :src="item['#IMG_POSTER'] !== 'N/A' ? item['#IMG_POSTER'] : 'https://placehold.co/300x450?text=No+Image'"
                          :alt="item['#TITLE']" class="object-cover w-full h-full" />
                      </figure>
                      <div class="card-body">
                        <h4 class="card-title text-lg">{{ item['#TITLE'] }}</h4>
                        <p class="text-sm opacity-70">{{ item["#YEAR"] }}</p>
                      </div>
                    </div>
                  </div>

                </div>
              </div>

              <!-- No Results -->
              <div v-else-if="searchItems.length === 0 && movieName" class="text-center py-8">
                <i data-lucide="search-x" class="h-16 w-16 mx-auto text-base-content/30 mb-4"></i>
                <p class="text-base-content/70">No movies found for "{{ movieName }}"</p>
              </div>

              <!-- Empty State -->
              <div v-else class="text-center py-12">
                <i data-lucide="search" class="h-16 w-16 mx-auto text-base-content/30 mb-4"></i>
                <p class="text-base-content/70">Enter a movie name to search for information</p>
              </div>
            </template>
          </div>
        </div>
      </div>

      <!-- Files Section -->
      <div class="card bg-base-100 shadow-xl">
        <div class="card-body">
          <h2 class="card-title mb-4">
            <FileText class="mr-2 h-6 w-6" />
            Available Files
            <div class="badge badge-secondary">{{ movie.files_data?.length }} files</div>
          </h2>

          <div class="overflow-x-auto">
            <table class="table-zebra table">
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
                    <div class="text-base-content/70 max-w-xs truncate text-sm">{{ file.path }}</div>
                  </td>
                  <td>
                    <div class="badge badge-outline">{{ file.quality || 'N/A' }}</div>
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
                      <button class="btn btn-xs btn-primary" @click="playFile(file.path)">Play</button>
                      <button class="btn btn-xs btn-secondary" @click="openFileLocation(file.path)">Location</button>
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import type { VideoMetaData } from '../type'
import { useRoute } from 'vue-router'
import { get_video_by_id, update_video_imdb } from '../functions/invoker'
import { ArrowLeft, Star, FileText } from 'lucide-vue-next'
import { dirname } from '@tauri-apps/api/path'
import { openPath } from '@tauri-apps/plugin-opener'
import { fetch } from '@tauri-apps/plugin-http'
import { toast } from 'vue3-toastify'

function playFile(path: string) {
  // Implement play functionality
  console.log('Playing file:', path)
  // You can use Tauri's shell API to open the file with default player
  openPath(path).catch((e) => console.log(e))
}

async function openFileLocation(path: string) {
  try {
    // Implement open file location functionality
    const dir = await dirname(path)
    console.log('Opening file location:', path)
    console.log('Opening dir location:', dir)
    // You can use Tauri's shell API to open the directory
    await openPath(dir)
  } catch (e) {
    console.log(e)
  }
}

const movie = ref<VideoMetaData>()

const route = useRoute()

onMounted(async () => {
  movie.value = await get_video_by_id(+route.params.id)
  movieName.value = movie.value?.name
})

const movieName = ref('');

interface MovieSearchResult {
  ok: boolean;
  description: SearchedMovie[];
  error_code: number;
}

interface SearchedMovie {
  "#TITLE": string;
  "#YEAR": number;
  "#IMDB_ID": string;
  "#RANK": number;
  "#ACTORS": string;
  "#AKA": string;
  "#IMDB_URL": string;
  "#IMDB_IV": string;
  "#IMG_POSTER"?: string;
  photo_width?: number;
  photo_height?: number;
}

const searchItems = ref<SearchedMovie[]>([])
const loading = ref(false)
async function searchMovies(title: string) {
  loading.value = true
  const response = await fetch(`https://imdb.iamidiotareyoutoo.com/search?q=${encodeURIComponent(title)}`);
  const result: MovieSearchResult = await response.json();
  searchItems.value = result.description
  loading.value = false
}

watch(movieName, searchMovies)

const change = ref(false)

async function selectMovie(imdb_id: string) {
  try {
    await update_video_imdb(+route.params.id, imdb_id);
    movie.value = await get_video_by_id(+route.params.id)
  }
  catch (e: unknown) {
    const message = e instanceof Error ? e.message : 'Failed to set imdb'
    toast.error(message)
  }
  finally {
    change.value = false
  }
}

</script>
