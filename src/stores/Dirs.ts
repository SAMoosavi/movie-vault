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
     * Checks if a path is a parent of another path.
     * @param parent - The potential parent path.
     * @param child - The potential child path.
     * @returns True if parent is a parent of child.
     */
    function isParent(parent: string, child: string): boolean {
      const parentPath = parent.replace(/\\/g, '/').replace(/\/$/, '')
      const childPath = child.replace(/\\/g, '/').replace(/\/$/, '')
      return childPath.startsWith(parentPath + '/')
    }

    /**
     * Adds a new directory path with parent-child logic.
     * If the new directory is a parent of existing directories, removes the children.
     * If an existing directory is a parent of the new one, doesn't add it.
     * @param dir - The directory path to add.
     * @returns True if added, false if not added or already covered.
     */
    function addDirectory(dir: string): boolean {
      const normalizedDir = dir.replace(/\\/g, '/').replace(/\/$/, '')

      // Check if already exists
      if (directoryPaths.value.includes(normalizedDir)) return false

      // Check if any existing directory is a parent of the new one
      for (const existing of directoryPaths.value) {
        if (isParent(existing, normalizedDir)) {
          return false // Already covered by parent
        }
      }

      // Remove any existing directories that are children of the new one
      directoryPaths.value = directoryPaths.value.filter((existing) => !isParent(normalizedDir, existing))

      // Add the new directory
      directoryPaths.value.push(normalizedDir)
      return true
    }

    /**
     * Removes a specific directory path from the list.
     * @param dir - The directory path to remove.
     * @returns True if removed, false if not found.
     */
    function removeDirectory(dir: string): boolean {
      const index = directoryPaths.value.indexOf(dir)
      if (index === -1) return false
      directoryPaths.value.splice(index, 1)
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

    return { directoryPaths, addDirectory, removeDirectory, removeLastDirectory }
  },
  {
    persist: {
      storage: localStorage,
      pick: ['directoryPaths'],
    },
  },
)
