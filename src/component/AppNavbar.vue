<template>
  <aside
    class="bg-base-200 relative z-50 flex h-screen flex-col overflow-hidden shadow-lg"
    :class="isCollapsed ? 'w-16' : 'w-full md:w-64'"
  >
    <!-- Header -->
    <header class="border-base-300 flex items-center justify-between border-b p-4">
      <RouterLink
        to="/"
        class="from-primary to-secondary bg-gradient-to-r bg-clip-text text-xl font-bold whitespace-nowrap text-transparent transition-all duration-500 ease-in-out"
        :class="{ hidden: isCollapsed }"
      >
        Movie Vault
      </RouterLink>

      <button
        @click="toggleSidebar"
        class="btn btn-ghost btn-sm btn-circle hover:bg-base-300 transition-transform duration-300 ease-in-out"
      >
        <MenuIcon v-if="isCollapsed" class="h-5 w-5 rotate-0 transform transition-transform duration-300" />
        <XIcon v-else class="h-5 w-5 rotate-180 transform transition-transform duration-300" />
      </button>
    </header>

    <!-- Nav -->
    <nav class="flex-1 space-y-2 p-2">
      <RouterLink
        to="/"
        v-slot="{ isActive }"
        class="group hover:bg-base-300 flex items-center gap-3 rounded-lg px-3 py-2 transition-all duration-500 ease-in-out"
        :class="{ 'justify-center': isCollapsed }"
        data-tooltip="Home"
      >
        <HomeIcon
          class="h-5 w-5 flex-shrink-0 transition-colors duration-300"
          :class="isActive ? 'text-primary' : 'text-base-content'"
        />
        <span
          :class="[
            { hidden: isCollapsed, 'font-medium': isActive },
            'whitespace-nowrap transition-opacity duration-500 ease-in-out',
          ]"
          >Home</span
        >
      </RouterLink>

      <RouterLink
        :to="{ name: 'setting_page' }"
        v-slot="{ isActive }"
        class="group hover:bg-base-300 flex items-center gap-3 rounded-lg px-3 py-2 transition-all duration-500 ease-in-out"
        :class="{ 'justify-center': isCollapsed }"
        data-tooltip="Settings"
      >
        <SettingsIcon
          class="h-5 w-5 flex-shrink-0 transition-colors duration-300"
          :class="isActive ? 'text-primary' : 'text-base-content'"
        />
        <span
          :class="[
            { hidden: isCollapsed, 'font-medium': isActive },
            'whitespace-nowrap transition-opacity duration-500 ease-in-out',
          ]"
          >Settings</span
        >
      </RouterLink>
    </nav>

    <!-- Footer -->
    <footer class="border-base-300 space-y-2 border-t p-4">
      <RouterLink
        :to="{ name: 'add_media' }"
        class="btn btn-secondary btn-sm flex w-full items-center gap-2 transition-all duration-500 ease-in-out"
        :class="{ 'justify-center': isCollapsed }"
      >
        <Plus class="h-4 w-4 flex-shrink-0" />
        <span :class="{ hidden: isCollapsed, 'transition-opacity duration-500 ease-in-out': true }">Add Media</span>
      </RouterLink>

      <button
        class="btn btn-primary btn-sm flex w-full items-center gap-2 transition-all duration-500 ease-in-out"
        :class="{ 'justify-center': isCollapsed }"
        @click="onAddDirectory"
      >
        <FolderPlus class="h-4 w-4 flex-shrink-0" />
        <span :class="{ hidden: isCollapsed, 'transition-opacity duration-500 ease-in-out': true }">Add Folder</span>
      </button>
    </footer>
  </aside>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { RouterLink } from 'vue-router'
import { open } from '@tauri-apps/plugin-dialog'
import { FolderPlus, Plus, HomeIcon, SettingsIcon, MenuIcon, XIcon } from 'lucide-vue-next'
import { toast } from 'vue3-toastify'
import { useMediasStore } from '@/stores/medias'
import { useDirsStore } from '@/stores/Dirs'
import { sync_files } from '@/functions/invoker'
import { handleFrontendError } from '@/functions/errorHandling'

const isCollapsed = ref(false)

const mediasStore = useMediasStore()
const dirsStore = useDirsStore()

function toggleSidebar() {
  isCollapsed.value = !isCollapsed.value
}

defineExpose({ toggleSidebar })

function normalizeDirectoryPath(path: string): string {
  return path.replace(/\\/g, '/').replace(/\/$/, '')
}

async function onAddDirectory() {
  const selectedDirectory = await open({ multiple: false, directory: true })
  if (!selectedDirectory || typeof selectedDirectory !== 'string') {
    toast.info('No directory selected')
    return
  }

  const normalizedDirectory = normalizeDirectoryPath(selectedDirectory)
  const wasAdded = dirsStore.addDirectory(normalizedDirectory)
  if (!wasAdded) {
    toast.warning('Directory already exists or is already covered by another directory.')
    return
  }

  toast.info('Adding directory and syncing files...')
  let addedCount = 0
  try {
    addedCount = await sync_files(normalizedDirectory)
  } catch (error) {
    dirsStore.removeDirectory(normalizedDirectory)
    handleFrontendError('navbar.add_directory.sync', error, 'Failed to sync added directory')
    return
  }

  try {
    await mediasStore.reload()
  } catch (error) {
    handleFrontendError(
      'navbar.add_directory.reload',
      error,
      'Directory added and synced, but failed to refresh media list',
    )
    return
  }

  toast.success(`Successfully added directory with ${addedCount} items!`)
}
</script>
