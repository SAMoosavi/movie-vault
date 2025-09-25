import { defineStore } from 'pinia'
import { ref } from 'vue'

/**
 * Pinia store for managing a list of directory paths.
 */
export const useDirsStore = defineStore(
  'dirs',
  () => {
    // Reactive array holding directory paths
    const directoryPaths = ref<string[]>([])

    /**
     * Adds a new directory path if it doesn't already exist.
     * @param dir - The directory path to add.
     * @returns True if added, false if already present.
     */
    function addDirectory(dir: string): boolean {
      if (directoryPaths.value.includes(dir)) return false
      directoryPaths.value.push(dir)
      return true
    }

    /**
     * Removes the last directory path from the list.
     * @returns True if removed, false if the list was empty.
     */
    function removeLastDirectory(): boolean {
      if (directoryPaths.value.length === 0) return false
      directoryPaths.value.pop()
      return true
    }

    return { directoryPaths, addDirectory, removeLastDirectory }
  },
  {
    persist: {
      storage: localStorage,
      pick: ['directoryPaths'],
    },
  },
)
