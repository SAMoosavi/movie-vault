<template>
  <div class="dropdown dropdown-start w-full">
    <!-- Input and selected items badges -->
    <div
      tabindex="0"
      class="bg-base-100 input flex w-full cursor-text flex-row items-center gap-1 rounded-lg border p-2"
    >
      <!-- Search input -->
      <input v-model="searchTerm" type="text" placeholder="Type to search..." class="flex-1" />

      <!-- Badge showing count of extra selected items -->
      <div v-if="selectedItems.length > 2" class="badge badge-primary badge-xs z-10 gap-1">
        +{{ selectedItems.length - 2 }}
      </div>

      <!-- Badges for up to 2 selected items -->
      <div v-for="item in selectedItems.slice(0, 2)" :key="item[0]" class="badge badge-primary badge-xs z-10 gap-1">
        {{ item[1] }}
      </div>
    </div>

    <!-- Dropdown list -->
    <ul
      tabindex="0"
      class="dropdown-content bg-base-100 text-base-content rounded-box top-px mt-12 max-h-60 w-full overflow-y-auto border border-white/5 shadow-2xl outline-1 outline-black/5"
    >
      <!-- Selected items (removable) -->
      <li
        v-for="item in selectedItems"
        :key="'selected-' + item[0]"
        class="bg-primary text-primary-content flex cursor-pointer items-center justify-between px-4 py-2"
        @click="removeItem(item)"
      >
        {{ item[1] }}
        <CircleX class="h-3 w-3" />
      </li>

      <!-- Filtered items (selectable) -->
      <li
        v-for="item in filteredItems"
        :key="'filtered-' + item[0]"
        class="hover:bg-secondary hover:text-secondary-content cursor-pointer px-4 py-2"
        @click="selectItem(item)"
      >
        {{ item[1] }}
      </li>

      <!-- No results message -->
      <li v-if="filteredItems.length === 0" class="px-4 py-2 text-gray-500">No results found</li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import { CircleX } from 'lucide-vue-next'
import { ref, computed } from 'vue'

import Fuse from 'fuse.js'

// Explicitly declare Fuse types
interface FuseResult<T> {
  item: T
}

type KeyType = string | number
type ItemType = [KeyType, string]

// Props: list of items to select from
const props = defineProps<{ items: ItemType[] }>()
const emit = defineEmits<{
  (e: 'selected-items', value: KeyType[]): void
}>()

// Ref for selected items
const selectedItems = ref<ItemType[]>([])

// Search input value
const searchTerm = ref('')

// Setup Fuse.js for fuzzy search
const fuse = computed(() => {
  return new Fuse(props.items, {
    keys: ['1'], // search in second element of tuple
    threshold: 0.4,
  })
})

// Filter items based on search term and exclude already selected
const filteredItems = computed(() => {
  const selectedIds = selectedItems.value.map((item) => item[0])
  if (!searchTerm.value) {
    return props.items.filter((item) => !selectedIds.includes(item[0]))
  }

  const results = fuse.value.search(searchTerm.value)
  const matchedItems = results.map((r: FuseResult<ItemType>) => r.item)
  return matchedItems.filter((item: ItemType) => !selectedIds.includes(item[0]))
})

// Add item to selected list
function selectItem(item: ItemType) {
  if (!selectedItems.value.includes(item)) {
    selectedItems.value.push(item)
    emit(
      'selected-items',
      selectedItems.value.map((i) => i[0]),
    )
  }
  searchTerm.value = ''
}

// Remove item from selected list
function removeItem(item: ItemType) {
  selectedItems.value = selectedItems.value.filter((i) => i[0] !== item[0])
  emit(
    'selected-items',
    selectedItems.value.map((i) => i[0]),
  )
}
</script>
