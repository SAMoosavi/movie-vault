<template>
  <!-- Tag Settings Card -->
  <SettingCategoryCard name="Tag Settings" description="Manage your tags">
    <div class="card-body">
      <!-- Section: Add New Tag -->
      <section class="mb-8">
        <h2 class="card-title text-xl">Add New Tag</h2>
        <div class="form-control mt-4">
          <div class="flex gap-2">
            <input
              v-model="newTag.name"
              type="text"
              placeholder="Enter tag name"
              class="input input-bordered flex-1"
              @keyup.enter="handleAddTag"
            />
            <button @click="handleAddTag" class="btn btn-primary">
              <Plus class="h-5 w-5" />
              Add
            </button>
          </div>
        </div>
      </section>

      <!-- Section: Existing Tags List -->
      <section class="mb-8">
        <h2 class="card-title text-xl">Existing Tags</h2>
        <div v-if="tagList.length === 0" class="text-base-content/60 mt-3 italic">
          No tags available. Add a new tag to get started.
        </div>
        <!-- Animated Tag List -->
        <AnimatedList tag="div" class="mt-3 flex flex-wrap gap-3">
          <div
            v-for="tag in tagList"
            :key="tag.id"
            @click="handleSelectTag(tag)"
            class="badge badge-lg badge-outline cursor-pointer transition-all"
            :class="{ 'badge-primary': selectedTag.id === tag.id }"
          >
            <div class="flex items-center gap-2">
              <span class="font-medium">{{ tag.name }}</span>
              <CircleCheckBig v-if="selectedTag.id === tag.id" class="h-3 w-3" />
            </div>
          </div>
        </AnimatedList>
      </section>

      <!-- Section: Selected Tag Actions -->
      <AnimatedShow>
        <section v-if="selectedTag.id">
          <div class="divider from-primary/60 to-secondary/60 before:bg-gradient-to-r after:bg-gradient-to-r"></div>
          <h2 class="card-title text-xl">Selected Tag</h2>
          <div class="mt-4 flex flex-col items-start gap-4 md:flex-row md:items-end">
            <div class="form-control flex-1">
              <label class="label">
                <span class="label-text font-medium">Tag Name</span>
              </label>
              <input v-model="selectedTag.name" type="text" class="input input-bordered w-full" />
            </div>
            <div class="flex gap-2">
              <button @click="handleUpdateTag" class="btn btn-success">
                <Save class="h-5 w-5" />
                Update
              </button>
              <button @click="handleRemoveTag" class="btn btn-error">
                <Trash2 class="h-5 w-5" />
                Remove
              </button>
            </div>
          </div>
        </section>
      </AnimatedShow>
    </div>
  </SettingCategoryCard>
</template>

<script setup lang="ts">
// --- Icons & Vue ---
import { Plus, CircleCheckBig, Save, Trash2 } from 'lucide-vue-next'
import { onMounted, ref } from 'vue'

// --- Types & API ---
import type { Tag } from '../../type'
import { get_tags, insert_tag, remove_tag, update_tag } from '../../functions/invoker'

// --- Components ---
import AnimatedShow from '../../component/AnimatedShow.vue'
import AnimatedList from '../../component/AnimatedList.vue'
import SettingCategoryCard from '../../component/SettingCategoryCard.vue'

// --- State ---
const emptyTag: Tag = { id: 0, name: '' }
const tagList = ref<Tag[]>([])
const newTag = ref<Tag>({ ...emptyTag })
const selectedTag = ref<Tag>({ ...emptyTag })

onMounted(fetchTags)

async function fetchTags() {
  tagList.value = await get_tags()
}

async function handleAddTag() {
  if (!newTag.value.name.trim()) return
  await insert_tag(newTag.value)
  newTag.value = { ...emptyTag }
  await fetchTags()
}

async function handleRemoveTag() {
  if (!selectedTag.value.id) return
  await remove_tag(selectedTag.value.id)
  selectedTag.value = { ...emptyTag }
  await fetchTags()
}

async function handleUpdateTag() {
  if (!selectedTag.value.id || !selectedTag.value.name.trim()) return
  await update_tag(selectedTag.value)
  selectedTag.value = { ...emptyTag }
  await fetchTags()
}

function handleSelectTag(tag: Tag) {
  selectedTag.value = { ...tag }
}
</script>
