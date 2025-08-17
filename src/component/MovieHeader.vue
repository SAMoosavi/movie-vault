<template>
  <div class="card from-primary/50 to-secondary/50 mb-8 bg-gradient-to-br p-0.5 shadow-xl">
    <div class="card lg:card-side bg-base-100">
      <!-- Poster -->
      <figure class="w-full lg:w-1/4">
        <img :src="media.imdb?.poster" :alt="media.imdb?.title" />
      </figure>

      <div class="card-body">
        <!-- Movie Info -->
        <div class="flex items-center justify-between">
          <h1 class="mb-2 text-3xl font-bold md:text-4xl">
            {{ media.imdb?.title }}
            <span class="text-base-content/70 text-2xl">({{ media.imdb?.year }})</span>
          </h1>

          <button class="btn btn-primary" @click="$emit('edit')">change imdb data</button>
        </div>
        <!-- Rating and Meta Info -->
        <div class="mb-4 flex flex-wrap items-center gap-4">
          <div class="flex cursor-pointer items-center gap-2" @click="$emit('toggle-watched')">
            <div v-if="media.watched" class="badge badge-lg badge-success gap-1">
              <Eye class="h-4 w-4" />
              <span>Watched</span>
            </div>
            <div v-else class="badge badge-lg badge-outline gap-1">
              <EyeOff class="h-4 w-4" />
              <span>Not Watched</span>
            </div>
          </div>

          <div
            v-if="!media.watched"
            class="flex cursor-pointer items-center gap-2"
            @click="$emit('toggle-watch-list')"
          >
            <div v-if="media.watch_list" class="badge badge-lg badge-primary gap-1">
              <BookmarkCheck class="h-4 w-4" />
              <span>On Watchlist</span>
            </div>
            <div v-else class="badge badge-lg badge-outline gap-1">
              <BookmarkPlus class="h-4 w-4" />
              <span>Add to Watchlist</span>
            </div>
          </div>

          <!-- Personal Rating -->
          <div class="flex items-center gap-2">
            <span class="font-semibold">My Rating:</span>
            <div class="flex">
              <Star
                v-for="i in 5"
                :key="i"
                class="text-warning h-5 w-5 cursor-pointer"
                :class="{
                  'text-warning fill-warning': i <= media.my_ranking,
                }"
                @click="$emit('set-ranking', i)"
              />
            </div>
          </div>

          <div class="badge badge-lg badge-warning gap-1">
            <Star class="h-5 w-5 fill-current" />
            <span class="font-bold">{{ media.imdb?.imdb_rating }}</span>
            <span class="text-xs">/10</span>
          </div>

          <div class="text-sm">
            <span class="badge badge-outline mr-2">{{ media.imdb?.rated }}</span>
            <span>{{ media.imdb?.runtime }}</span>
          </div>

          <div class="text-sm">
            {{ media.imdb?.released }}
          </div>
        </div>

        <!-- Genres -->
        <div class="mb-4 flex flex-wrap gap-2">
          <span v-for="genre in media.imdb?.genres" :key="genre" class="badge badge-primary badge-md">
            {{ genre }}
          </span>
        </div>

        <!-- 🏷 Tags -->
        <div class="flex items-center gap-2">
          <!-- Add Tag -->
          <div class="flex items-center gap-2">
            <TagsIcon class="text-primary h-5 w-5" />
            <select v-model="selectedTagId" class="select select-bordered select-sm w-40">
              <option disabled :value="0">+ Select Tag</option>
              <option v-for="tag in selectableTags" :key="tag.id" :value="tag.id">
                {{ tag.name }}
              </option>
            </select>
            <button class="btn btn-sm btn-primary" :disabled="!selectedTagId" @click="addTagToMovie">Add</button>
          </div>

          <!-- Current Tags -->
          <div class="flex flex-wrap gap-2">
            <span
              v-for="tag in media.tags"
              :key="tag.id"
              class="badge badge-md badge-outline badge-accent cursor-pointer"
              @click="removeTag(tag.id)"
            >
              {{ tag.name }}
              <CircleX class="h-4 w-4" />
            </span>
          </div>
        </div>

        <!-- Plot -->
        <p class="text-base-content/90 mb-6">
          {{ media.imdb?.plot }}
        </p>

        <!-- Cast and Crew -->
        <div class="mb-6 grid grid-cols-1 gap-4 md:grid-cols-2">
          <div>
            <h3 class="text-base-content/70 mb-1 font-semibold">Directors</h3>
            <p>{{ media.imdb?.directors?.join(', ') || 'N/A' }}</p>
          </div>
          <div>
            <h3 class="text-base-content/70 mb-1 font-semibold">Writers</h3>
            <p>{{ media.imdb?.writers?.join(', ') || 'N/A' }}</p>
          </div>
          <div>
            <h3 class="text-base-content/70 mb-1 font-semibold">Stars</h3>
            <p>{{ media.imdb?.actors?.join(', ') || 'N/A' }}</p>
          </div>
          <div>
            <h3 class="text-base-content/70 mb-1 font-semibold">Languages</h3>
            <p>{{ media.imdb?.languages?.join(', ') || 'N/A' }}</p>
          </div>
        </div>

        <!-- Additional Info -->
        <div class="grid grid-cols-1 gap-4 text-sm md:grid-cols-3">
          <div>
            <h3 class="text-base-content/70 mb-1 font-semibold">Country</h3>
            <p>{{ media.imdb?.country?.join(', ') || 'N/A' }}</p>
          </div>

          <div v-if="media.imdb?.type == 'movie'">
            <h3 class="text-base-content/70 mb-1 font-semibold">Box Office</h3>
            <p>{{ media.imdb?.box_office || 'N/A' }}</p>
          </div>
          <div v-else-if="media.imdb?.type == 'series'">
            <h3 class="text-base-content/70 mb-1 font-semibold">Total Seasons</h3>
            <p>{{ media.imdb?.total_seasons || 'N/A' }}</p>
          </div>

          <div>
            <h3 class="text-base-content/70 mb-1 font-semibold">Awards</h3>
            <p>{{ media.imdb?.awards }}</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import type { Media, Tag } from '../type'
import { Star, Eye, EyeOff, BookmarkPlus, BookmarkCheck, TagsIcon, CircleX } from 'lucide-vue-next'
import { get_tags, insert_media_tag, remove_media_tag } from '../functions/invoker'

const props = defineProps<{ media: Media }>()
defineEmits(['edit', 'toggle-watched', 'set-ranking', 'toggle-watch-list'])

const tags = ref<Tag[]>([])
const selectedTagId = ref(0)

onMounted(async () => {
  tags.value = await get_tags()
})

const selectableTags = computed(() => tags.value.filter((tag) => !props.media.tags.some((t) => t.id === tag.id)))

async function addTagToMovie() {
  await insert_media_tag(props.media.id, selectedTagId.value)
  selectedTagId.value = 0
}

async function removeTag(tag_id: number) {
  console.log(tag_id)

  await remove_media_tag(props.media.id, tag_id)
}
</script>
