<template>
  <!-- Main Container -->
  <div class="container mx-auto mt-5 min-h-screen">
    <div class="grid grid-cols-1 gap-8 lg:grid-cols-3">
      <!-- Left Column -->
      <div class="space-y-6 lg:col-span-2">
        <template v-if="!media">
          <MediaHeaderSkeleton />
        </template>
        <template v-else>
          <MediaHeader v-if="media.imdb && !isEditing" :media="media" />
          <SearchMediaImdb v-else :media="media" @updated="updated" />
        </template>
      </div>

      <!-- Right Column -->
      <div class="space-y-6 lg:col-span-1">
        <ManageSection
          v-if="media"
          :is-editing="isEditing"
          :media="media"
          @toggle-editing="toggleEditing"
          @fetch-media="fetchMedia"
          @delete-media="goBack"
        />
        <ManageSectionSkeleton v-else />

        <TagSection v-if="media" :media="media" @fetch-media="fetchMedia" />
        <TagSectionSkeleton v-else />
      </div>
    </div>

    <FilesSection v-if="media" :media="media" @fetch-media="fetchMedia" />
    <FilesSectionSkeleton v-else />
  </div>
</template>

<script setup lang="ts">
// --- External ---
import { ref, onMounted } from 'vue'
import { toast } from 'vue3-toastify'

// --- Routing & types ---
import { useRouter, useRoute } from 'vue-router'
import type { Media } from '../type'

// --- Functions & components ---
import { get_media_by_id } from '../functions/invoker'

import MediaHeader from '../component/media_page/MediaHeader.vue'
import SearchMediaImdb from '../component/media_page/SearchMediaImdb.vue'
import MediaHeaderSkeleton from '../component/media_page/MediaHeaderSkeleton.vue'
import FilesSectionSkeleton from '../component/media_page/FilesSectionSkeleton.vue'
import FilesSection from '../component/media_page/FilesSection.vue'

import TagSection from '../component/media_page/TagSection.vue'
import ManageSection from '../component/media_page/ManageSection.vue'
import ManageSectionSkeleton from '../component/media_page/ManageSectionSkeleton.vue'
import TagSectionSkeleton from '../component/media_page/TagSectionSkeleton.vue'

// --- State ---
const route = useRoute()
const router = useRouter()
const media = ref<Media | null>(null)
const isEditing = ref(false)

// --- Navigation ---
function goBack() {
  router.back()
}

// Fetch media data by ID (safer error handling)
function fetchMedia() {
  get_media_by_id(Number(route.params.id))
    .then((data) => (media.value = data))
    .catch((error) => {
      toast.error(typeof error === 'string' ? error : error instanceof Error ? error.message : 'Failed to fetch media')
      goBack()
    })
}

// --- Edit mode handlers ---
function toggleEditing() {
  isEditing.value = !isEditing.value
}

// --- Media actions ---
async function updated(id: number) {
  await router.push({ name: 'media_page', params: { id } })
  fetchMedia()
  toggleEditing()
}

onMounted(() => {
  fetchMedia()
})
</script>
