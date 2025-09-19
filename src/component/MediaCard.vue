<template>
  <RouterLink
    :to="{ name: 'movie_page', params: { id: media.id.toString() } }"
    class="card from-primary/50 to-secondary/50 h-72 w-52 bg-gradient-to-br p-0.5 transition-transform duration-200 hover:scale-[102%]"
  >
    <div class="card card-compact bg-base-100 h-full w-full overflow-hidden shadow-lg">
      <figure class="relative h-full">
        <img
          :src="media.imdb?.poster"
          :alt="media.name"
          class="h-full w-full object-cover"
          @error="
            (e) =>
              ((e.target as HTMLImageElement).src =
                `https://placehold.jp/20rem/3d4070/ffffff/200x280.png?text=${media.name}`)
          "
          loading="lazy"
        />

        <!-- IMDB Rating Badge -->
        <div
          v-if="media.imdb?.imdb_rating"
          class="badge badge-warning absolute top-2 left-2 flex items-center gap-1 text-xs"
        >
          <StarIcon class="h-3 w-3" />
          <span>{{ media.imdb?.imdb_rating || 'N/A' }}</span>
        </div>

        <!-- Year Badge -->
        <div
          v-if="media.year || media.imdb?.year"
          class="badge badge-primary absolute top-2 right-2 flex items-center gap-1 text-xs"
        >
          <CalendarIcon class="h-3 w-3" />
          <span>{{ media.year || media.imdb?.year }}</span>
        </div>

        <!-- Type Icon -->
        <div class="bg-secondary absolute right-2 bottom-2 flex h-6 w-6 items-center justify-center rounded-full p-1">
          <component
            v-if="media.imdb?.type"
            :is="media.imdb?.type === 'Movie' ? FilmIcon : TvIcon"
            class="text-secondary-content h-3 w-3"
          />
          <component v-else :is="media.seasons.length ? TvIcon : FilmIcon" class="text-secondary-content h-3 w-3" />
        </div>

        <!-- Watched Icon -->
        <div
          v-if="media.watched"
          class="bg-success absolute bottom-2 left-2 flex h-6 w-6 items-center justify-center rounded-full p-1"
        >
          <EyeIcon class="text-success-content h-3 w-3" />
        </div>

        <div
          v-if="media.imdb?.title"
          class="bg-primary text-primary-content card absolute bottom-2 left-1/2 max-w-1/2 -translate-x-1/2 p-1 text-center text-xs text-wrap"
        >
          {{ media.imdb?.title }}
        </div>
      </figure>
    </div>
  </RouterLink>
</template>

<script setup lang="ts">
import { CalendarIcon, StarIcon, FilmIcon, TvIcon, EyeIcon } from 'lucide-vue-next'
import type { Media } from '../type'

defineProps<{ media: Media }>()
</script>
