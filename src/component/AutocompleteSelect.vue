<template>
  <div class="dropdown dropdown-start w-full">

    <div tabindex="0" class="flex flex-row items-center gap-1 p-2 border rounded-lg bg-base-100 cursor-text input">
      <input v-model="searchTerm" type="text" class="" placeholder="Type to search..." />


      <div v-if="selectedItemsModel.length > 2" class="badge badge-primary badge-xs z-10 gap-1">
        +{{ selectedItemsModel.length - 2 }}
      </div>

      <div v-for="item in selectedItemsModel.slice(0, 2)" :key="item[0]"
        class="badge badge-primary badge-xs z-10 gap-1">
        {{ item[1] }}
      </div>

    </div>

    <!-- Dropdown -->
    <ul tabindex="0"
      class="dropdown-content bg-base-100 text-base-content rounded-box top-px max-h-60 overflow-y-auto border border-white/5 shadow-2xl outline-1 outline-black/5 mt-16 w-full">
      <li v-for="item in selectedItemsModel" :key="item[0]"
        class="px-4 py-2 cursor-pointer bg-primary text-primary-content flex items-center justify-between"
        @click="removeItem(item)">
        {{ item[1] }}
        <CircleX class="h-3 w-3" />
      </li>
      <li v-for="item in filteredItems" :key="item[0]"
        class="px-4 py-2 cursor-pointer hover:bg-secondary hover:text-secondary-content" @click="selectItem(item)">
        {{ item[1] }}
      </li>
      <li v-if="filteredItems.length === 0" class="px-4 py-2 text-gray-500">
        No results found
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import { CircleX } from 'lucide-vue-next';
import { ref, computed } from 'vue'
import type { NumericalString } from '../type';

const props = defineProps<{ items: NumericalString[] }>()
const selectedItemsModel = defineModel<NumericalString[]>({ default: () => [] })

const searchTerm = ref('')

const filteredItems = computed(() => {
  const selectedIds = selectedItemsModel.value.map(i => i[0])

  return props.items.filter(item => {
    if (selectedIds.includes(item[0])) return false
    if (!searchTerm.value) return true
    return item[1].toLowerCase().includes(searchTerm.value.toLowerCase())
  })
})

// Function to select an item
function selectItem(item: NumericalString) {
  if (!selectedItemsModel.value.includes(item))
    selectedItemsModel.value.push(item)

  searchTerm.value = ''
}


function removeItem(item: NumericalString) {
  selectedItemsModel.value = selectedItemsModel.value.filter(i => i[0] !== item[0])
}
</script>
