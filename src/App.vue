<template>
  <!-- App Navbar -->
  <AppNavbar />
  <!-- Router View -->
  <router-view />
</template>

<script setup lang="ts">
// --- External Libraries ---
import { onMounted, onBeforeUnmount, watch } from 'vue'
import { toast } from 'vue3-toastify'

// --- Local Components ---
import AppNavbar from './component/AppNavbar.vue'

// --- Tauri API ---
import { watch as fsWatch, type UnwatchFn } from '@tauri-apps/plugin-fs'

// --- Stores ---
import { useDirsStore } from './stores/Dirs'
import { storeToRefs } from 'pinia'
import { useVideosStore } from './stores/Videos'

// --- Functions ---
import { sync_files } from './functions/invoker'
import { getDefaultTheme, initStore, loadTheme, setTheme } from './functions/theme.ts'

// --- State ---
const videos = useVideosStore()
const dirsStore = useDirsStore()
const { directoryPaths } = storeToRefs(dirsStore)
let unwatchFns: UnwatchFn[] = []

// --- Helper: Stop watching directories ---
function stopWatching() {
  unwatchFns.forEach((fn) => fn())
  unwatchFns = []
}

// --- Helper: Start watching directories ---
async function startWatching(paths: string[]) {
  stopWatching()
  for (const path of paths) {
    try {
      const unwatch = await fsWatch(
        path,
        async () => {
          await sync_files(path)
          await videos.reload()
        },
        { recursive: true, delayMs: 1000 },
      )
      unwatchFns.push(unwatch)
    } catch (error) {
      console.error(`Failed to set up file watcher for ${path}:`, error)
    }
  }
}

// --- Lifecycle: On mount, initialize theme and sync files ---
onMounted(async () => {
  try {
    // Theme initialization
    const store = await initStore()
    const theme = (await loadTheme(store)) ?? getDefaultTheme()
    await setTheme(theme, store)
  } catch (e) {
    toast.error(e instanceof Error ? e.message : String(e))
  }

  try {
    // Initial sync and watcher setup
    for (const dir of directoryPaths.value) {
      await sync_files(dir)
    }
    await videos.reload()
    await startWatching(directoryPaths.value)
  } catch (e) {
    toast.error(e instanceof Error ? e.message : String(e))
  }
})

// --- Watch for changes in directory paths ---
watch(
  () => directoryPaths.value,
  async (paths) => {
    await startWatching(paths)
  },
  { immediate: true, deep: true },
)

// --- Clean up watchers on unmount ---
onBeforeUnmount(stopWatching)
</script>
