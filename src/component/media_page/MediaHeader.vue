<template>
  <div class="card from-primary/40 to-secondary/40 mb-8 bg-gradient-to-br p-0.5 shadow-xl">
    <div class="card bg-base-100 overflow-hidden shadow-lg">
      <div class="card-body p-6">
        <div class="flex flex-col gap-6 md:flex-row">
          <div class="flex-shrink-0 md:w-64">
            <img
              :src="media.imdb?.poster"
              :alt="media.name"
              class="w-full rounded-xl object-cover shadow"
              loading="lazy"
            />
          </div>

          <div class="flex-1">
            <h1 class="mb-4 flex items-center gap-2 text-4xl font-bold">
              {{ media.imdb?.title || media.name }}
            </h1>
            <h2 class="mb-4 flex items-center gap-2 text-2xl font-bold">
              <InfoIcon class="text-primary h-6 w-6" />
              Overview
            </h2>
            <p class="text-base-content/80 mb-6 leading-relaxed">
              {{ media.imdb?.plot || 'No plot description available.' }}
            </p>

            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
              <div class="md:col-span-2">
                <h3 class="mb-2 flex items-center gap-2 font-semibold">
                  <UsersIcon class="text-primary h-5 w-5" />
                  Cast
                </h3>
                <div class="flex flex-wrap gap-2">
                  <a
                    v-for="actor in media.imdb?.actors"
                    :key="actor.id"
                    target="_blank"
                    :href="`https://www.imdb.com/name/${actor.id}`"
                    class="flex cursor-pointer flex-col items-center gap-1"
                  >
                    <div class="avatar">
                      <div class="h-12 w-12 rounded-full">
                        <img :alt="actor.name" :src="actor.url" />
                      </div>
                    </div>
                    <span> {{ actor.name }} </span>
                  </a>
                </div>
              </div>

              <div>
                <h3 class="mb-2 flex items-center gap-2 font-semibold">
                  <GlobeIcon class="text-primary h-5 w-5" />
                  Countries
                </h3>
                <div class="flex flex-wrap gap-2">
                  <span v-for="country in media.imdb?.countries" :key="country" class="badge badge-secondary">
                    {{ country }}
                  </span>
                </div>
              </div>

              <div>
                <h3 class="mb-2 flex items-center gap-2 font-semibold">
                  <PuzzleIcon class="text-primary h-5 w-5" />
                  genres
                </h3>
                <div class="flex flex-wrap gap-2">
                  <span v-for="genre in media.imdb?.genres" :key="genre" class="badge badge-secondary">
                    {{ genre }}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { UsersIcon, InfoIcon, GlobeIcon, PuzzleIcon } from 'lucide-vue-next'
import type { Media } from '../../type'

defineProps<{ media: Media }>()
</script>
