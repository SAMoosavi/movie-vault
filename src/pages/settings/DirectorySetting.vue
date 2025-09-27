<template>
  <!-- Directory Settings Card -->
  <SettingCategoryCard name="Directory Settings" description="Manage your media directories">
    <div class="card-body">
      <!-- Section: Add New Directory -->
      <section class="mb-8">
        <h2 class="card-title text-xl">Add New Directory</h2>
        <div class="form-control mt-4">
          <button @click="handleAddDirectory" class="btn btn-primary">
            <FolderPlusIcon class="mr-2 h-5 w-5" />
            Select Directory
          </button>
        </div>
      </section>

      <!-- Section: Existing Directories List -->
      <section class="mb-8">
        <h2 class="card-title text-xl">Existing Directories</h2>
        <div v-if="directoryPaths.length === 0" class="text-base-content/60 mt-3 italic">
          No directories available. Add a new directory to get started.
        </div>
        <!-- Animated Directory List -->
        <AnimatedList tag="div" class="mt-3 space-y-2">
          <div
            v-for="dir in directoryPaths"
            :key="dir"
            @click="handleRemoveDirectory(dir)"
            class="badge badge-lg badge-outline flex cursor-pointer items-center gap-2 p-3 transition-all"
          >
            <Folder class="h-4 w-4" />
            <span class="font-medium">{{ dir }}</span>
          </div>
        </AnimatedList>
      </section>
    </div>
  </SettingCategoryCard>
</template>

<script setup lang="ts">
// --- Icons & Vue ---
import { Folder, FolderPlusIcon } from 'lucide-vue-next'
import { computed } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'

// --- Stores ---
import { useDirsStore } from '../../stores/Dirs'
import { useMediasStore } from '../../stores/medias'

// --- Components ---
import AnimatedList from '../../component/AnimatedList.vue'
import SettingCategoryCard from '../../component/SettingCategoryCard.vue'

// --- Functions ---
import { sync_files } from '../../functions/invoker'
import { toast } from 'vue3-toastify'

// --- State ---
const dirsStore = useDirsStore()
const mediasStore = useMediasStore()

// Computed
const directoryPaths = computed(() => dirsStore.directoryPaths)

async function handleAddDirectory() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
    })
    if (selected && typeof selected === 'string') {
      dirsStore.addDirectory(selected)
      const addedCount = await sync_files(selected)
      await mediasStore.reload()
      toast.success(`Successfully added directory with ${addedCount} items!`)
    }
  } catch (error) {
    dirsStore.removeLastDirectory()
    console.error('Failed to add directory:', error)
    toast.error(`Failed to add directory: ${error instanceof Error ? error.message : 'Unknown error'}`)
  }
}

async function handleRemoveDirectory(dir: string) {
  if (!dir) return
  dirsStore.removeDirectory(dir)
}
</script>
