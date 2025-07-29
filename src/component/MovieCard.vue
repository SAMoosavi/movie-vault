<template>
  <RouterLink :to="{ name: 'movie_page', params: { id: movie.id.toString() } }"
    class="card from-primary/50 to-secondary/50 mb-8 bg-gradient-to-br p-0.5 transition-all duration-200 hover:scale-[1.02]">
    <div class="card bg-base-100">
      <figure class="relative">
        <img :src="movie.imdb_metadata?.poster" :alt="movie.imdb_metadata?.title || movie.name"
          class="h-96 w-full object-cover" />

        <div v-if="(movie.imdb_metadata?.imdb_rating || NA) !== NA"
          class="badge badge-neutral absolute bottom-3 left-3 flex items-center">
          <Star class="text-warning fill-warning mr-1 h-4 w-4" />
          <span class="font-semibold">{{ movie.imdb_metadata?.imdb_rating }}</span>
        </div>
      </figure>

      <div class="card-body p-4">
        <h3 class="card-title truncate text-lg">
          {{ movie.imdb_metadata?.title || movie.name }}
        </h3>

        <div class="text-base-content/70 mb-2 flex items-center text-sm">
          <Calendar class="mr-1 h-4 w-4" />
          <span>{{ movie.imdb_metadata?.year || movie.year }}</span>
          <span class="mx-2">•</span>
          <span class="capitalize">{{ movie.imdb_metadata?.type }}</span>
        </div>

        <div class="mb-3 flex flex-wrap gap-1">
          <span v-for="genre in movie.imdb_metadata?.genre" :key="genre" class="badge badge-outline badge-sm">
            {{ genre }}
          </span>
        </div>

        <div class="text-base-content/80 flex items-center text-sm">
          <User class="mr-1 h-4 w-4" />
          <div class="truncate">
            {{ movie.imdb_metadata?.actors?.slice(0, 2).join(', ') }}
            <span v-if="(movie.imdb_metadata?.actors?.length || 0) > 2">, ...</span>
          </div>
        </div>
      </div>
    </div>
  </RouterLink>
</template>

<script setup lang="ts">
import { Calendar, Star, User } from 'lucide-vue-next'
import type { VideoMetaData } from '../type'

defineProps<{ movie: VideoMetaData }>()

const NA = 'N/A'
</script>
