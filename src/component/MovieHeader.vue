<template>
  <div class="container mx-auto px-4 py-8">
    <!-- Hero Section with Cinematic Feel -->
    <div class="relative mb-8 overflow-hidden rounded-3xl shadow-2xl">
      <div class="from-base-900/90 absolute inset-0 z-10 bg-gradient-to-t to-transparent"></div>

      <div class="relative aspect-video overflow-hidden">
        <video
          :src="`https://imdb.iamidiotareyoutoo.com/media/${media.imdb?.imdb_id}`"
          :poster="`https://placehold.jp/50rem/3d4070/ffffff/1280x720.png?text=${media.imdb?.title}`"
          autoplay
          loop
          muted
          playsinline
          class="h-full w-full object-cover"
          loading="lazy"
        ></video>

        <div class="absolute inset-x-0 bottom-0 z-20 p-6 md:p-10">
          <div class="flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
            <div>
              <h1 class="mb-2 text-3xl font-bold text-white drop-shadow-lg md:text-5xl">
                {{ media.imdb?.title || media.name }}
              </h1>
              <div class="flex flex-wrap items-center gap-3 text-white/90">
                <span class="text-lg font-medium">{{ media.year || media.imdb?.year }}</span>
                <div v-for="genre in media.imdb?.genres" :key="genre" class="badge badge-outline">
                  {{ genre }}
                </div>
              </div>
            </div>

            <div class="flex flex-wrap gap-3">
              <button class="btn btn-ghost hover:btn-primary btn-circle" @click="$emit('toggle-watched')">
                <component :is="media.watched ? EyeIcon : EyeOffIcon" class="h-5 w-5" />
              </button>
              <button class="btn btn-ghost hover:btn-secondary btn-circle" @click="$emit('edit')">
                <EditIcon class="h-5 w-5" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Main Content -->
    <div class="grid grid-cols-1 gap-8 lg:grid-cols-3">
      <!-- Left Column -->
      <div class="space-y-6 lg:col-span-2">
        <!-- Media Details Card -->
        <div class="card from-primary/40 to-secondary/40 mb-8 bg-gradient-to-br p-0.5 shadow-xl">
          <div class="card bg-base-100 overflow-hidden shadow-lg">
            <div class="card-body p-6">
              <div class="flex flex-col gap-6 md:flex-row">
                <div class="flex-shrink-0 md:w-64">
                  <img
                    :src="`https://imdb.iamidiotareyoutoo.com/photo/${media.imdb?.imdb_id}`"
                    :alt="media.name"
                    class="w-full rounded-xl object-cover shadow"
                    loading="lazy"
                    @error="(e) => ((e.target as HTMLImageElement).src = media.imdb?.poster || '')"
                  />
                </div>

                <div class="flex-1">
                  <h2 class="mb-4 flex items-center gap-2 text-2xl font-bold">
                    <InfoIcon class="text-primary h-6 w-6" />
                    Overview
                  </h2>
                  <p class="text-base-content/80 mb-6 leading-relaxed">
                    {{ media.imdb?.plot || 'No plot description available.' }}
                  </p>

                  <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                    <div>
                      <h3 class="mb-2 flex items-center gap-2 font-semibold">
                        <UsersIcon class="h-5 w-5" />
                        Cast
                      </h3>
                      <div class="flex flex-wrap gap-2">
                        <a
                          v-for="actor in media.imdb?.actors"
                          :key="actor.id"
                          :href="actor.url"
                          target="_blank"
                          rel="noopener"
                          class="badge badge-outline hover:badge-primary transition-transform hover:scale-105"
                        >
                          {{ actor.name }}
                        </a>
                      </div>
                    </div>

                    <div>
                      <h3 class="mb-2 flex items-center gap-2 font-semibold">
                        <GlobeIcon class="h-5 w-5" />
                        Countries
                      </h3>
                      <div class="flex flex-wrap gap-2">
                        <span v-for="country in media.imdb?.countries" :key="country" class="badge badge-secondary">
                          {{ country }}
                        </span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Tags Card -->
        <div class="card from-primary/40 to-secondary/40 mb-8 bg-gradient-to-br p-0.5 shadow-xl">
          <div class="card bg-base-100 overflow-hidden shadow-lg">
            <div class="card-body p-6">
              <div class="mb-4 flex flex-col sm:flex-row sm:items-center sm:justify-between">
                <h2 class="mb-3 flex items-center gap-2 text-2xl font-bold sm:mb-0">
                  <TagIcon class="text-primary h-6 w-6" />
                  Tags
                </h2>

                <div class="join">
                  <select v-model="selectedTagId" class="select select-bordered join-item">
                    <option value="0">Select tag</option>
                    <option v-for="tag in selectableTags" :key="tag.id" :value="tag.id">{{ tag.name }}</option>
                  </select>
                  <button class="btn btn-primary join-item" @click="addTagToMovie" :disabled="selectedTagId === 0">
                    <PlusIcon class="h-4 w-4" />
                  </button>
                </div>
              </div>

              <div class="flex min-h-[40px] flex-wrap gap-2">
                <span
                  v-for="tag in media.tags"
                  :key="tag.id"
                  class="badge badge-lg badge-primary hover:badge-error group gap-1transition-transform flex cursor-pointer items-center hover:scale-105"
                  @click="removeTag(tag.id)"
                >
                  {{ tag.name }}
                  <XCircleIcon class="h-4 w-4" />
                </span>
                <span v-if="media.tags.length === 0" class="text-base-content/50 italic"> No tags added yet </span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Right Column -->
      <div class="space-y-6 lg:col-span-1">
        <div class="card from-primary/40 to-secondary/40 mb-8 bg-gradient-to-br p-0.5 shadow-xl">
          <div class="card bg-base-100 sticky top-8 overflow-hidden shadow-lg">
            <div class="card-body p-6">
              <h2 class="mb-4 text-xl font-bold">Manage Media</h2>

              <div class="space-y-3">
                <button
                  class="btn btn-outline hover:bg-base-200 w-full justify-between transition-colors"
                  @click="$emit('edit')"
                >
                  <span>Edit Details</span>
                  <PencilIcon class="h-5 w-5" />
                </button>

                <button
                  class="btn w-full justify-between"
                  :class="media.watch_list ? 'btn-secondary' : 'btn-outline'"
                  @click="$emit('toggle-watch-list')"
                >
                  <span>{{ media.watch_list ? 'Remove from Watchlist' : 'Add to Watchlist' }}</span>
                  <BookmarkIcon class="h-5 w-5" />
                </button>

                <div class="divider my-4"></div>

                <div class="card from-warning to-warning/40 mb-8 bg-gradient-to-br p-0.5 shadow-xl">
                  <div class="card bg-base-100 p-4 text-center">
                    <h3 class="mb-2 flex items-center justify-center gap-2 font-semibold">
                      <StarIcon class="text-warning h-5 w-5" />
                      IMDb Rating
                    </h3>
                    <div class="py-3">
                      <div class="text-warning mb-1 text-4xl font-bold">
                        {{ media.imdb?.imdb_rating || 'N/A' }}
                      </div>
                      <div class="text-base-content/70 text-sm">{{ media.imdb?.imdb_votes || '0' }} votes</div>
                    </div>
                  </div>
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
import { ref, computed, onMounted } from 'vue'
import {
  StarIcon,
  UsersIcon,
  TagIcon,
  EditIcon,
  InfoIcon,
  GlobeIcon,
  PlusIcon,
  PencilIcon,
  BookmarkIcon,
  EyeIcon,
  EyeOffIcon,
  XCircleIcon,
} from 'lucide-vue-next'
import type { Media, Tag } from '../type'
import { get_tags, insert_media_tag, remove_media_tag } from '../functions/invoker'
import { toast } from 'vue3-toastify'

const props = defineProps<{ media: Media }>()

defineEmits<{
  (e: 'edit'): void
  (e: 'toggle-watched'): void
  (e: 'set-ranking', rank: number): void
  (e: 'toggle-watch-list'): void
}>()

const tags = ref<Tag[]>([])
const selectedTagId = ref<number>(0)

onMounted(async () => {
  try {
    tags.value = await get_tags()
  } catch (err) {
    console.error('Failed to load tags:', err)
  }
})

const selectableTags = computed(() => {
  const mediaTags = props.media?.tags ?? []
  return tags.value.filter((t) => !mediaTags.some((mt) => mt.id === t.id))
})

async function addTagToMovie() {
  const tagId = selectedTagId.value
  if (!tagId) return
  try {
    await insert_media_tag(props.media.id, tagId)
    selectedTagId.value = 0
  } catch (err) {
    console.error('Failed to add tag:', err)
    toast.error('Failed to add tag')
  }
}

async function removeTag(tag_id: number) {
  try {
    await remove_media_tag(props.media.id, tag_id)
  } catch (err) {
    console.error('Failed to remove tag:', err)
    toast.error('Failed to remove tag')
  }
}
</script>
