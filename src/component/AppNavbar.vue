<template>
  <div class="navbar bg-base-100 sticky top-0 z-10 shadow-lg">
    <!-- App Name -->
    <div class="navbar-start">
      <div class="dropdown">
        <div tabindex="0" role="button" class="btn btn-ghost lg:hidden">
          <AlignJustify class="h-5 w-5" />
        </div>
        <ul tabindex="0" class="menu menu-sm dropdown-content bg-base-100 rounded-box z-10 mt-3 w-52 p-2 shadow">
          <li><a>Home</a></li>
          <li><a>Movies</a></li>
          <li><a>TV Shows</a></li>
          <li><a>Watchlist</a></li>
        </ul>
      </div>
      <span class="from-primary to-secondary ml-2 bg-gradient-to-r bg-clip-text text-xl font-bold text-transparent">
        Movie Vault
      </span>
    </div>

    <!-- Desktop Menu -->
    <div class="navbar-center hidden lg:flex">
      <ul class="menu menu-horizontal px-1">
        <li><a class="hover:bg-base-200 rounded-lg">Home</a></li>
        <li><a class="hover:bg-base-200 rounded-lg">Movies</a></li>
        <li><a class="hover:bg-base-200 rounded-lg">TV Shows</a></li>
        <li><a class="hover:bg-base-200 rounded-lg">Watchlist</a></li>
      </ul>
    </div>

    <!-- Search and Actions -->
    <div class="navbar-end flex items-center gap-2">
      <!-- Theme Toggle -->
      <label class="swap swap-rotate">
        <input type="checkbox" class="theme-controller" value="synthwave" />
        <Sun class="swap-off h-5 w-5" />
        <Moon class="swap-on h-5 w-5" />
      </label>

      <!-- Search Box -->
      <div class="form-control">
        <label class="input input-bordered input-sm w-full max-w-xs">
          <Search class="h-4" />
          <input v-model="search_box" type="search" placeholder="Search movie name..." />
        </label>
      </div>

      <!-- Add Folder Button -->
      <button class="btn btn-primary btn-sm" @click="handleAddDirectory">
        <FolderPlus class="h-4 w-4" />
        Add Folder
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'

import { Search, FolderPlus, Sun, Moon, AlignJustify } from 'lucide-vue-next'
import { toast } from 'vue3-toastify'

const search_box = defineModel<string>()

const emit = defineEmits<{ (e: 'addDir', path: string): void }>()

async function handleAddDirectory() {
  try {
    const selectedDir = await open({
      multiple: false,
      directory: true,
    })

    if (!selectedDir) {
      toast.info('No directory selected')
      return
    }

    if (typeof selectedDir === 'string') {
      emit('addDir', selectedDir)
    } else {
      throw new Error('Invalid directory path returned')
    }
  } catch (error) {
    console.error('Error selecting directory:', error)
    toast.error(`Failed to select directory: ${error instanceof Error ? error.message : 'Unknown error'}`)
  }
}
</script>
