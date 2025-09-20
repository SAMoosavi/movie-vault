<template>
  <!-- Outer card with gradient border -->
  <div class="card from-primary/40 to-secondary/40 mb-8 bg-gradient-to-br p-0.5 shadow-xl">
    <!-- Inner card -->
    <div class="card bg-base-100 overflow-hidden shadow-lg">
      <div class="card-body p-6">
        <!-- Header -->
        <div class="mb-4 flex flex-col sm:flex-row sm:items-center sm:justify-between">
          <h2 class="mb-3 flex items-center gap-2 text-2xl font-bold sm:mb-0">
            <TagIcon class="text-primary h-6 w-6" />
            Tags
          </h2>

          <!-- Add-tag controls -->
          <div class="join">
            <select v-model="selectedTagId" aria-label="Select a tag to add" class="select select-bordered join-item">
              <option disabled :value="0">Select tag</option>
              <option v-for="tag in selectableTags" :key="tag.id" :value="tag.id">
                {{ tag.name }}
              </option>
            </select>
            <button
              class="btn btn-primary join-item"
              aria-label="Add selected tag"
              :disabled="selectedTagId === 0"
              @click="addTagToMedia"
            >
              <PlusIcon class="h-4 w-4" />
            </button>
          </div>
        </div>

        <!-- Tag list -->
        <div class="flex min-h-[40px] flex-wrap gap-2">
          <span
            v-for="tag in media.tags"
            :key="tag.id"
            class="badge badge-lg badge-primary hover:badge-error group flex cursor-pointer items-center gap-1 transition-transform hover:scale-105"
            :aria-label="`Remove tag ${tag.name}`"
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
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import type { Media, Tag } from '../../type'
import { get_tags, insert_media_tag, remove_media_tag } from '../../functions/invoker'
import { toast } from 'vue3-toastify'
import { TagIcon, PlusIcon, XCircleIcon } from 'lucide-vue-next'

/* Props */
const props = defineProps<{ media: Media }>()
const emit = defineEmits<{
  (e: 'fetch-media'): void
}>()

function fetchMedia() {
  emit('fetch-media')
}

/* State */
const tags = ref<Tag[]>([])
const selectedTagId = ref<number>(0)

/* Computed */
const selectableTags = computed(() => {
  const mediaTagIds = new Set(props.media.tags?.map((t) => t.id) ?? [])
  return tags.value.filter((t) => !mediaTagIds.has(t.id))
})

/* Methods */
async function addTagToMedia() {
  if (!selectedTagId.value) return
  try {
    await insert_media_tag(props.media.id, selectedTagId.value)
    selectedTagId.value = 0 // Reset selector
    fetchMedia()
  } catch (err) {
    toast.error('Failed to add tag')
    console.error(err)
  }
}

async function removeTag(tagId: number) {
  try {
    await remove_media_tag(props.media.id, tagId)
    fetchMedia()
  } catch (err) {
    toast.error('Failed to remove tag')
    console.error(err)
  }
}

/* Lifecycle */
onMounted(async () => {
  try {
    tags.value = await get_tags()
  } catch (err) {
    console.error(err)
  }
})
</script>
