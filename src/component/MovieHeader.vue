<template>
  <div class="card from-primary/50 to-secondary/50 mb-8 bg-gradient-to-br p-0.5 shadow-xl">
    <div class="card lg:card-side bg-base-100">
      <!-- Poster -->
      <figure class="w-full lg:w-1/4">
        <img :src="movie.imdb?.poster" :alt="movie.imdb?.title" />
      </figure>

      <div class="card-body">
        <!-- Movie Info -->
        <div class="flex items-center justify-between">
          <h1 class="mb-2 text-3xl font-bold md:text-4xl">
            {{ movie.imdb?.title }}
            <span class="text-base-content/70 text-2xl">({{ movie.imdb?.year }})</span>
          </h1>

          <button class="btn btn-primary" @click="$emit('edit')">change imdb data</button>
        </div>
        <!-- Rating and Meta Info -->
        <div class="mb-4 flex flex-wrap items-center gap-4">
          <button class="flex cursor-pointer items-center gap-2" @click="$emit('toggle-watched')">
            <div v-if="movie.watched" class="badge badge-lg badge-success gap-1">
              <Eye class="h-4 w-4" />
              <span>Watched</span>
            </div>
            <div v-else class="badge badge-lg badge-outline gap-1">
              <EyeOff class="h-4 w-4" />
              <span>Not Watched</span>
            </div>
          </button>

          <!-- Personal Rating -->
          <div class="flex items-center gap-2">
            <span class="font-semibold">My Rating:</span>
            <div class="flex">
              <Star
                v-for="i in 5"
                :key="i"
                class="text-warning h-5 w-5 cursor-pointer"
                :class="{
                  'text-warning fill-warning': i <= movie.my_ranking,
                }"
                @click="$emit('set-ranking', i)"
              />
            </div>
          </div>

          <div class="badge badge-lg badge-warning gap-1">
            <Star class="h-5 w-5 fill-current" />
            <span class="font-bold">{{ movie.imdb?.imdb_rating }}</span>
            <span class="text-xs">/10</span>
          </div>

          <div class="text-sm">
            <span class="badge badge-outline mr-2">{{ movie.imdb?.rated }}</span>
            <span>{{ movie.imdb?.runtime }}</span>
          </div>

          <div class="text-sm">
            {{ movie.imdb?.released }}
          </div>
        </div>

        <!-- Genres -->
        <div class="mb-4 flex flex-wrap gap-2">
          <span v-for="genre in movie.imdb?.genre" :key="genre" class="badge badge-primary badge-md">
            {{ genre }}
          </span>
        </div>

        <!-- Plot -->
        <p class="text-base-content/90 mb-6">
          {{ movie.imdb?.plot }}
        </p>

        <!-- Cast and Crew -->
        <div class="mb-6 grid grid-cols-1 gap-4 md:grid-cols-2">
          <div>
            <h3 class="text-base-content/70 mb-1 font-semibold">Directors</h3>
            <p>{{ movie.imdb?.directors?.join(', ') || 'N/A' }}</p>
          </div>
          <div>
            <h3 class="text-base-content/70 mb-1 font-semibold">Writers</h3>
            <p>{{ movie.imdb?.writers?.join(', ') || 'N/A' }}</p>
          </div>
          <div>
            <h3 class="text-base-content/70 mb-1 font-semibold">Stars</h3>
            <p>{{ movie.imdb?.actors?.join(', ') || 'N/A' }}</p>
          </div>
          <div>
            <h3 class="text-base-content/70 mb-1 font-semibold">Languages</h3>
            <p>{{ movie.imdb?.languages?.join(', ') || 'N/A' }}</p>
          </div>
        </div>

        <!-- Additional Info -->
        <div class="grid grid-cols-1 gap-4 text-sm md:grid-cols-3">
          <div>
            <h3 class="text-base-content/70 mb-1 font-semibold">Country</h3>
            <p>{{ movie.imdb?.country?.join(', ') || 'N/A' }}</p>
          </div>

          <div v-if="movie.imdb?.type == 'movie'">
            <h3 class="text-base-content/70 mb-1 font-semibold">Box Office</h3>
            <p>{{ movie.imdb?.box_office || 'N/A' }}</p>
          </div>
          <div v-else-if="movie.imdb?.type == 'series'">
            <h3 class="text-base-content/70 mb-1 font-semibold">Total Seasons</h3>
            <p>{{ movie.imdb?.total_seasons || 'N/A' }}</p>
          </div>

          <div>
            <h3 class="text-base-content/70 mb-1 font-semibold">Awards</h3>
            <p>{{ movie.imdb?.awards }}</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Media } from '../type'
import { Star, Eye, EyeOff } from 'lucide-vue-next'

defineProps<{ movie: Media }>()
defineEmits(['edit', 'toggle-watched', 'set-ranking'])
</script>
