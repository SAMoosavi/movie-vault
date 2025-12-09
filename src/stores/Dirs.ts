import { defineStore } from 'pinia'
import { ref } from 'vue'
import { info, error, warn } from '@tauri-apps/plugin-log'

/**
 * Pinia store for managing a list of directory paths.
 */
export const useDirsStore = defineStore(
  'dirs',
  () => {
    // Reactive array holding directory paths
    const directoryPaths = ref<string[]>([])

    function isParent(parent: string, child: string): boolean {
      const parentPath = parent.replace(/\\/g, '/').replace(/\/$/, '')
      const childPath = child.replace(/\\/g, '/').replace(/\/$/, '')
      return childPath.startsWith(parentPath + '/')
    }

    function addDirectory(dir: string): boolean {
      const normalizedDir = dir.replace(/\\/g, '/').replace(/\/$/, '')
      info(`Attempting to add directory: ${normalizedDir}`)

      if (directoryPaths.value.includes(normalizedDir)) {
        warn(`Directory already exists: ${normalizedDir}`)
        return false
      }

      for (const existing of directoryPaths.value) {
        if (isParent(existing, normalizedDir)) {
          warn(`Directory is already covered by parent: ${existing}`)
          return false
        }
      }

      directoryPaths.value = directoryPaths.value.filter((existing) => !isParent(normalizedDir, existing))
      directoryPaths.value.push(normalizedDir)
      info(`Directory added: ${normalizedDir}`)
      return true
    }

    function removeDirectory(dir: string): boolean {
      const index = directoryPaths.value.indexOf(dir)
      if (index === -1) {
        error(`Directory not found: ${dir}`)
        return false
      }
      directoryPaths.value.splice(index, 1)
      info(`Directory removed: ${dir}`)
      return true
    }

    function removeLastDirectory(): boolean {
      if (directoryPaths.value.length === 0) {
        warn(`No directories to remove.`)
        return false
      }
      const removedDir = directoryPaths.value.pop()
      info(`Last directory removed: ${removedDir}`)
      return true
    }

    return { directoryPaths, addDirectory, removeDirectory, removeLastDirectory }
  },
  {
    persist: {
      storage: localStorage,
      pick: ['directoryPaths'],
    },
  },
)
