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
import { create_table, sync_app } from './functions/invoker'
import { useVideosStore } from './stores/Videos'
import { toast } from 'vue3-toastify'

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

  paths.forEach(async (path) => {
    try {
      const unwatch = await fsWatch(
        path,
        async (event) => {
          if (event?.type && 'access' in (event.type as object)) {
            return
          }
          console.log(event.type)
          await sync_app(path)
          await videos.updata()
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
  })
}

onMounted(async () => {
  try {
    // Initialize database
    await create_table()

    // Sync files with better error handling
    const syncPromises = dir_path.value.map(sync_app)
    await Promise.all(syncPromises)

    await startWatching(dir_path.value)

    toast.success('initialized successfully!')
  } catch (e) {
    console.error('Initialization error:', e)
    toast.error(`Failed to initialize: ${e instanceof Error ? e.message : 'Unknown error'}`)
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
