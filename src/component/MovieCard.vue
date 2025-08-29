<template>
  <RouterLink
    :to="{ name: 'movie_page', params: { id: movie.id.toString() } }"
    class="card from-primary/50 to-secondary/50 mb-8 bg-gradient-to-br p-0.5 transition-all duration-200 hover:scale-[1.02]"
  >
    <div class="card bg-base-100 h-full">
      <figure class="relative">
        <img :src="movie.imdb?.poster" :alt="movie.imdb?.title || movie.name" class="h-96 w-full object-cover" />

        <div
          v-if="(movie.imdb?.imdb_rating || NA) !== NA"
          class="badge badge-neutral absolute bottom-3 left-3 flex items-center"
        >
          <Star class="text-warning fill-warning mr-1 h-4 w-4" />
          <span class="font-semibold">{{ movie.imdb?.imdb_rating }}</span>
        </div>
        <div v-if="movie.watched" class="badge badge-success absolute top-3 right-3 flex items-center">
          <Eye class="mr-1 h-4 w-4" />
          <span class="font-semibold">Watched</span>
        </div>

        <div v-if="movie.watch_list" class="badge badge-primary absolute right-3 bottom-3 flex items-center">
          <BookmarkCheck class="mr-1 h-4 w-4" />
          <span class="font-semibold">Watchlist</span>
        </div>
      </figure>

      <div class="card-body p-4">
        <h3 class="card-title truncate text-lg">
          {{ movie.imdb?.title || movie.name }}
        </h3>

        <div class="text-base-content/70 mb-2 flex items-center text-sm">
          <Calendar class="mr-1 h-4 w-4" />
          <span>{{ movie.imdb?.year || movie.year }}</span>
          <span class="mx-2">•</span>
          <span class="capitalize">{{ movie.imdb?.type }}</span>
        </div>

        <div v-if="movie.imdb?.genres.length" class="mb-3 flex flex-wrap gap-1">
          <span v-for="genre in movie.imdb?.genres" :key="genre" class="badge badge-outline badge-sm">
            {{ genre }}
          </span>
        </div>

        <div class="mb-3 flex flex-wrap gap-1">
          <Tags class="h4 mr-1 w-4" />
          <span v-for="tag in movie.tags" :key="tag.id" class="badge badge-md badge-outline badge-accent">
            {{ tag.name }}
          </span>
        </div>

        <div class="text-base-content/80 flex items-center text-sm">
          <User class="mr-1 h-4 w-4" />
          <div class="truncate">
            {{ movie.imdb?.actors?.slice(0, 2).map((a)=>a.name).join(', ') }}
            <span v-if="(movie.imdb?.actors?.length || 0) > 2">, ...</span>
          </div>
        </div>
        <div class="from-primary/50 to-secondary/50 -mb-3 h-0.5 w-full bg-gradient-to-b"></div>
        <div class="mt-3 flex items-center justify-between">
          <span class="mr-2 text-sm">My Rating:</span>
          <div class="flex">
            <Star
              v-for="i in 5"
              :key="i"
              :class="['h-4 w-4', i <= movie.my_ranking ? 'text-warning fill-warning' : 'text-warning']"
            />
          </div>
        </div>
      </div>
    </div>
  </RouterLink>
</template>

<script setup lang="ts">
import { Calendar, Star, User, Eye, BookmarkCheck, Tags } from 'lucide-vue-next'
import type { Media } from '../type'

defineProps<{ movie: Media }>()

const NA = 'N/A'
</script>
