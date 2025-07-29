import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useDirsStore = defineStore('dirs', () => {
  const dir_path = ref<string[]>([])

  function push(dir: string): boolean {
    if (dir_path.value.includes(dir)) return false

    dir_path.value.push(dir)
    return true
  }

  function pop(): boolean {
    if (dir_path.value.length == 0) return false

    dir_path.value.pop()
    return true
  }

  return { dir_path, push, pop }
})
