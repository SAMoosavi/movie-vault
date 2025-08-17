<template>
  <SettingCategoryCard name="Tag Settings" description="Manage your tags">
    <!-- Main Card -->
    <div class="card-body">
      <!-- Add Tag Section -->
      <section class="mb-8">
        <h2 class="card-title text-xl">Add New Tag</h2>
        <div class="form-control mt-4">
          <div class="flex gap-2">
            <input
              v-model="tag.name"
              type="text"
              placeholder="Enter tag name"
              class="input input-bordered flex-1"
              @keyup.enter="addTag"
            />
            <button @click="addTag" class="btn btn-primary">
              <Plus class="h-5 w-5" />
              Add
            </button>
          </div>
        </div>
      </section>

      <!-- Tags List Section -->
      <section class="mb-8">
        <h2 class="card-title text-xl">Existing Tags</h2>
        <div v-if="tags.length === 0" class="text-base-content/60 mt-3 italic">
          No tags available. Add a new tag to get started.
        </div>

        <!-- Animated Tag List -->
        <AnimatedList tag="div" class="mt-3 flex flex-wrap gap-3">
          <div
            v-for="t in tags"
            :key="t.id"
            @click="selectTag(t)"
            class="badge badge-lg badge-outline cursor-pointer transition-all"
            :class="{ 'badge-primary': selectedTag.id === t.id }"
          >
            <div class="flex items-center gap-2">
              <span class="font-medium">{{ t.name }}</span>
              <CircleCheckBig v-if="selectedTag.id === t.id" class="h-3 w-3" />
            </div>
          </div>
        </AnimatedList>
      </section>

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
              <button @click="updateTag" class="btn btn-success">
                <Save class="h-5 w-5" />
                Update
              </button>
              <button @click="removeTag" class="btn btn-error">
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
import { Plus, CircleCheckBig, Save, Trash2 } from 'lucide-vue-next'
import type { Tag } from '../../type'
import { get_tags, insert_tag, remove_tag, update_tag } from '../../functions/invoker'
import { onMounted, ref } from 'vue'
import AnimatedShow from '../../component/AnimatedShow.vue'
import AnimatedList from '../../component/AnimatedList.vue'
import SettingCategoryCard from '../../component/SettingCategoryCard.vue'

const defaultTag: Tag = { id: 0, name: '' }
const tags = ref<Tag[]>([])
const tag = ref<Tag>({ ...defaultTag })
const selectedTag = ref({ ...defaultTag })

onMounted(getTags)

async function getTags() {
  tags.value = await get_tags()
}

async function addTag() {
  await insert_tag(tag.value)
  tag.value = { ...defaultTag }
  await getTags()
}

async function removeTag() {
  await remove_tag(selectedTag.value.id)
  await getTags()
  selectedTag.value = { ...defaultTag }
}

async function updateTag() {
  await update_tag(selectedTag.value)
  await getTags()
  selectedTag.value = { ...defaultTag }
}

function selectTag(tagToSelect: Tag) {
  selectedTag.value = { ...tagToSelect }
}
</script>
