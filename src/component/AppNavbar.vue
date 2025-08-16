<template>
  <div class="navbar bg-base-100 sticky top-0 z-10 shadow-lg">
    <!-- App Name -->
    <div class="navbar-start">
      <div class="dropdown">
        <div tabindex="0" role="button" class="btn btn-ghost lg:hidden">
          <AlignJustify class="h-5 w-5" />
        </div>
        <ul tabindex="0" class="menu menu-sm dropdown-content bg-base-100 rounded-box z-10 mt-3 w-52 p-2 shadow">
          <li>
            <RouterLink to="/">Home</RouterLink>
          </li>
          <li><a @click="show_watch_list">Watchlist</a></li>
        </ul>
      </div>
      <span class="from-primary to-secondary ml-2 bg-gradient-to-r bg-clip-text text-xl font-bold text-transparent">
        Movie Vault
      </span>
    </div>

    <!-- Desktop Menu -->
    <div class="navbar-center hidden lg:flex">
      <ul class="menu menu-horizontal px-1">
        <li>
          <RouterLink to="/" class="hover:bg-base-200 rounded-lg">Home</RouterLink>
        </li>
        <li><a class="hover:bg-base-200 rounded-lg" @click="show_watch_list">Watchlist</a></li>
      </ul>
    </div>

    <!-- Search and Actions -->
    <div class="navbar-end flex items-center gap-2">
      <!-- Theme Toggle -->
      <button @click="() => (themNumber = (themNumber + 1) % 3)" class="btn btn-circle btn-ghost">
        <Sun v-if="themNumber == 0" class="h-5 w-5" />
        <Moon v-if="themNumber == 1" class="h-5 w-5" />
        <Coffee v-if="themNumber == 2" class="h-5 w-5" />
      </button>

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
import { FolderPlus, Sun, Moon, AlignJustify, Coffee } from 'lucide-vue-next'
import { toast } from 'vue3-toastify'
import { useVideosStore } from '../stores/Videos'
import { useDirsStore } from '../stores/Dirs'
import { sync_files } from '../functions/invoker'
import { onMounted, ref, watch } from 'vue'

const videos = useVideosStore()
const dirs = useDirsStore()

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

    // Check if directory already exists
    if (!dirs.push(selectedDir)) {
      toast.warning('Directory already added')
      return
    }

    toast.info('Adding directory and syncing files...')

    // Sync files in the new directory
    const number_of_added = await sync_files(selectedDir)

    // Refresh video metadata
    await videos.updata()

    toast.success(`Successfully added directory with ${number_of_added} items!`)
  } catch (error) {
    // Remove the directory if sync failed
    dirs.pop()

    console.error('Error adding directory:', error)
    toast.error(`Failed to add directory: ${error instanceof Error ? error.message : 'Unknown error'}`)
  }
}

const thems = ['dracula', 'winter', 'lemonade']
const themNumber = ref(0)

function setThem(index: number) {
  document.documentElement.setAttribute('data-theme', thems[index])
  localStorage.setItem('theme', index.toString())
}

function show_watch_list() {
  toast.info("This page doesn't exist. Please use the watchlist filter to find your saved movies.")
}

watch(themNumber, setThem)

onMounted(() => {
  let savedThemeNumber = 0
  const savedThemeNumberStr = localStorage.getItem('theme')
  if (savedThemeNumberStr) {
    savedThemeNumber = +savedThemeNumberStr
  } else {
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
    if (prefersDark) savedThemeNumber = 0
    else savedThemeNumber = 1
  }
  themNumber.value = savedThemeNumber
  setThem(savedThemeNumber)
})
</script>
