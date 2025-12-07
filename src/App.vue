<template>
  <div class="bg-base-100 flex h-screen">
    <!-- Sidebar Navigation -->
    <AppNavbar class="sidebar-nav" />

    <!-- Main Content Area -->
    <div class="flex flex-1 flex-col overflow-hidden">
      <!-- Sync-progress banner -->
      <div v-if="showProgress" class="z-50 px-4 py-2">
        <div class="alert alert-info shadow-lg">
          <span>Syncing media… {{ progress }}%</span>
          <progress class="progress progress-primary w-full" :value="progress" max="100"></progress>
        </div>
      </div>

      <!-- Router View -->
      <div class="flex-1 overflow-auto">
        <router-view />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// --- External Libraries ---
import { onMounted, onBeforeUnmount, watch, ref } from 'vue'
import { toast } from 'vue3-toastify'

// --- Local Components ---
import AppNavbar from './component/AppNavbar.vue'

// --- Tauri API ---
import { watch as fsWatch, stat, type UnwatchFn } from '@tauri-apps/plugin-fs'
import { listen } from '@tauri-apps/api/event'
import { dirname, normalize } from '@tauri-apps/api/path'
import { info, error, warn } from '@tauri-apps/plugin-log'

// --- Stores ---
import { useDirsStore } from './stores/Dirs'
import { storeToRefs } from 'pinia'
import { useMediasStore } from './stores/medias.ts'

// --- Functions ---
import { sync_files } from './functions/invoker'
import { getDefaultTheme, initStore, loadTheme, setTheme } from './functions/theme.ts'
import { checkForUpdates, handleUpdateFound } from './functions/update.ts'

// --- State ---
const mediasStore = useMediasStore()
const dirsStore = useDirsStore()
const { directoryPaths } = storeToRefs(dirsStore)
let unwatchFns: UnwatchFn[] = []

// --- Helper: Stop watching directories ---
function stopWatching() {
  unwatchFns.forEach((fn) => fn())
  unwatchFns = []
}

interface SyncFileProgressBare {
  inserted: number
  total: number
}

const progress = ref(0)
const showProgress = ref(false)

listen<SyncFileProgressBare>('sync-progress', (event) => {
  const { inserted, total } = event.payload
  progress.value = total > 0 ? Math.round((inserted / total) * 100) : 0
  showProgress.value = true
  console.log(`Sync progress: ${progress.value}%`)

  if (inserted === total) {
    setTimeout(() => (showProgress.value = false), 500)
  }
})

async function resolveToDirectory(inputPath: string) {
  const cleanPath = await normalize(inputPath)

  try {
    const info = await stat(cleanPath)
    if (info.isDirectory) return cleanPath
    return await dirname(cleanPath)
  } catch {
    warn(`Failed to stat path, using dirname: ${inputPath}`)
    return await dirname(cleanPath)
  }
}

// --- Helper: Start watching directories ---
async function startWatching(paths: string[]) {
  stopWatching()
  try {
    info(`Setting up file watchers for: ${paths.join(', ')}`)
    const unwatch = await fsWatch(
      paths,
      async (e) => {
        if (typeof e.type === 'object' && !('access' in e.type)) {
          info(`File change detected: ${e.paths.join(', ')}`)
          for (const path of e.paths) {
            info(`File change detected: ${path}`)
            const dir = await resolveToDirectory(path)
            info(`File change detected dir: ${dir}`)
            await sync_files(dir)
            info(`sync_file successfully from ${dir}`)
          }

          info('reload media')
          await mediasStore.reload()
        }
      },
      { recursive: true, delayMs: 1000 },
    )
    unwatchFns.push(unwatch)
    info('File watchers started successfully')
  } catch (err) {
    error(`Failed to set up file watcher for ${paths}: ${err}`)
  }
}

// --- Lifecycle: On mount, initialize theme and sync files ---
onMounted(async () => {
  info('App mounted: initializing theme and syncing files')

  try {
    info('Initializing store for theme')
    const store = await initStore()
    info('Loading theme from store')
    const theme = (await loadTheme(store)) ?? getDefaultTheme()
    info(`Applying theme: ${theme}`)
    await setTheme(theme, store)
    info('Theme applied successfully')
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    warn(`Theme initialization failed: ${msg}`)
    toast.error(msg)
  }

  try {
    info('Checking for updates')
    const update = await checkForUpdates()
    if (update) {
      info('Update found, handling update')
      handleUpdateFound(update)
    } else {
      info('No update found')
    }
  } catch (e) {
    error(`Update check failed: ${e}`)
  }

  try {
    info('Starting initial sync for directories')
    for (const dir of directoryPaths.value) {
      info(`Syncing directory: ${dir}`)
      await sync_files(dir)
      info(`Sync completed for: ${dir}`)
    }
    info('Reloading media store')
    await mediasStore.reload()
    info('Media store reloaded successfully')
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    error(`Initial sync failed: ${msg}`)
    toast.error(msg)
  }
})

// --- Watch for changes in directory paths ---
watch(
  () => directoryPaths.value,
  async (paths) => {
    info(`Directory paths changed: ${paths.join(', ')}`)
    await startWatching(paths)
  },
  { immediate: true, deep: true },
)

// --- Clean up watchers on unmount ---
onBeforeUnmount(() => {
  info('Cleaning up watchers on unmount')
  stopWatching()
})
</script>
