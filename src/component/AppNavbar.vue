<template>
  <!-- Navbar Container -->
  <div class="navbar bg-base-100 sticky top-0 z-99 shadow-lg">
    <!-- Left: App Name & Mobile Menu -->
    <div class="navbar-start">
      <!-- Mobile Dropdown Menu -->
      <div class="dropdown">
        <div tabindex="0" role="button" class="btn btn-ghost lg:hidden">
          <AlignJustify class="h-5 w-5" />
        </div>
        <ul tabindex="0" class="menu menu-sm dropdown-content bg-base-100 rounded-box z-10 mt-3 w-52 p-2 shadow">
          <li>
            <RouterLink to="/">Home</RouterLink>
          </li>
          <li>
            <a @click="showWatchlistInfo">Watchlist</a>
          </li>
          <li>
            <RouterLink :to="{ name: 'setting_page' }">Setting</RouterLink>
          </li>
        </ul>
      </div>
      <!-- App Name -->
      <span class="from-primary to-secondary ml-2 bg-gradient-to-r bg-clip-text text-xl font-bold text-transparent">
        Movie Vault
      </span>
    </div>

    <!-- Center: Desktop Menu -->
    <div class="navbar-center hidden lg:flex">
      <ul class="menu menu-horizontal px-1">
        <li>
          <RouterLink to="/" class="hover:bg-base-200 rounded-lg">Home</RouterLink>
        </li>
        <li>
          <a class="hover:bg-base-200 rounded-lg" @click="showWatchlistInfo">Watchlist</a>
        </li>
        <li>
          <RouterLink :to="{ name: 'setting_page' }">Setting</RouterLink>
        </li>
      </ul>
    </div>

    <!-- Right: Actions -->
    <div class="navbar-end flex items-center gap-2">
      <!-- Add Media Button -->
      <RouterLink :to="{ name: 'add_media' }" class="btn btn-secondary btn-sm">
        <Plus class="h-4 w-4" />
        Add Media
      </RouterLink>
      <!-- Add Folder Button -->
      <button class="btn btn-primary btn-sm" @click="onAddDirectory">
        <FolderPlus class="h-4 w-4" />
        Add Folder
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
// --- Tauri API imports ---
import { open } from '@tauri-apps/plugin-dialog'

// --- Icon imports ---
import { FolderPlus, AlignJustify, Plus } from 'lucide-vue-next'

// --- Toast ---
import { toast } from 'vue3-toastify'

// --- Stores ---
import { useMediasStore } from '../stores/medias'
import { useDirsStore } from '../stores/Dirs'

// --- Functions ---
import { sync_files } from '../functions/invoker'

// --- Store instances ---
const mediasStore = useMediasStore()
const dirsStore = useDirsStore()

/**
 * Handles adding a new directory.
 * - Opens a directory picker dialog.
 * - Adds the directory if not already present.
 * - Syncs files from the directory.
 * - Refreshes video metadata.
 * - Shows notifications for each step.
 */
async function onAddDirectory() {
  try {
    // Open directory picker
    const selectedDirectory = await open({
      multiple: false,
      directory: true,
    })
    if (!selectedDirectory) {
      toast.info('No directory selected')
      return
    }
    // Add directory to store, check for duplicates
    const wasAdded = dirsStore.addDirectory(selectedDirectory)
    if (!wasAdded) {
      toast.warning('Directory already added')
      return
    }
    toast.info('Adding directory and syncing files...')
    // Sync files and update video metadata
    const addedCount = await sync_files(selectedDirectory)
    await mediasStore.reload()
    toast.success(`Successfully added directory with ${addedCount} items!`)
  } catch (error) {
    // Remove last directory if sync failed
    dirsStore.removeLastDirectory()
    console.error('Error adding directory:', error)
    toast.error(`Failed to add directory: ${error instanceof Error ? error.message : 'Unknown error'}`)
  }
}

/**
 * Shows info about the watchlist feature.
 */
function showWatchlistInfo() {
  toast.info("This page doesn't exist. Please use the watchlist filter to find your saved movies.")
}
</script>
