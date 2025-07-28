import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useBaseStore = defineStore('base', () => {
  const dir_path = ref<string[]>([])

  function push(dir: string) {
    if (dir_path.value.includes(dir))
      throw new Error('Directory already added')

    dir_path.value.push(dir)
  }

  function pop() {
    if (dir_path.value.length > 0)
      dir_path.value.pop()
  }

  return {dir_path, push, pop}
})
