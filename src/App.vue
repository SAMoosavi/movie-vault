<template>
  <AppNavbar />

  <router-view />
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, onUnmounted, watch } from 'vue'
import AppNavbar from './component/AppNavbar.vue'
import { watch as fsWatch, type UnwatchFn } from '@tauri-apps/plugin-fs'
import { useDirsStore } from './stores/Dirs'
import { storeToRefs } from 'pinia'
import { sync_files } from './functions/invoker'
import { useVideosStore } from './stores/Videos'
import { toast } from 'vue3-toastify'
import { getDefaultTheme, initStore, loadTheme, setTheme } from './functions/theme.ts'

const videos = useVideosStore()
const dir = useDirsStore()
const { dir_path } = storeToRefs(dir)

let unwatchFns: UnwatchFn[] = []

const stopWatching = () => {
  unwatchFns.forEach((fn) => fn())
  unwatchFns = []
}

const startWatching = async (paths: string[]) => {
  stopWatching()

  for (const path of paths) {
    try {
      const unwatch = await fsWatch(
        path,
        async (event) => {
          if (event?.type && 'access' in (event.type as object)) {
            return
          }
          console.log(event.type)
          await sync_files(path)
          await videos.reload_media()
        },
        {
          recursive: true,
          delayMs: 1000,
        },
      )

      unwatchFns.push(unwatch)
    } catch (error) {
      console.error(`Failed to set up file watcher for path ${path}:`, error)
    }
  }
}

function getErrorMessage(e: unknown): string {
  return e instanceof Error ? e.message : String(e)
}

onMounted(async () => {
  try {
    const store = await initStore()
    const theme = (await loadTheme(store)) ?? getDefaultTheme()
    await setTheme(theme, store)
  } catch (e) {
    const message = getErrorMessage(e)
    console.error('Initialization error:', e)
    toast.error(`Failed to initialize: ${message}`)
  }

  try {

    // Sync files with better error handling
    const syncPromises = dir_path.value.map(sync_files)
    await Promise.all(syncPromises)

    await startWatching(dir_path.value)
  } catch (e) {
    const message = getErrorMessage(e)

    console.error('Initialization error:', e)
    toast.error(`Failed to initialize: ${message}`)
  }
})

watch(
  () => dir_path.value,
  async (v) => {
    await startWatching(v)
  },
  { immediate: true, deep: true },
)

onBeforeUnmount(() => stopWatching())

onUnmounted(() => stopWatching())
</script>
