<template>
  <div class="bg-base-100 container mx-auto px-4 py-8">
    <div class="flex flex-col gap-6 md:flex-row">
      <!-- Category Panel -->
      <div class="card from-primary/50 to-secondary/50 mb-8 h-fit bg-gradient-to-br p-0.5 shadow-md">
        <div class="card bg-base-200 h-fit w-full md:w-64">
          <div class="card-body p-4">
            <h2 class="card-title mb-2 text-lg">Categories</h2>
            <ul class="menu bg-base-200 w-full rounded-lg">
              <li v-for="(category, index) in categories" :key="index">
                <RouterLink
                  :to="{ name: category.route }"
                  class="flex items-center gap-2"
                  active-class="bg-primary text-primary-content"
                >
                  <component :is="category.icon" class="h-5 w-5" />
                  <span>{{ category.name }}</span>
                </RouterLink>
              </li>
            </ul>
          </div>
        </div>
      </div>

      <!-- Settings Panel -->
      <div class="ml-8 flex-1 py-8">
        <router-view />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { TagsIcon, PaletteIcon } from 'lucide-vue-next'

const categories = ref([
  { name: 'Tag', icon: TagsIcon, route: 'tags_setting' },
  {
    name: 'Appearance',
    icon: PaletteIcon,
    route: 'appearance_setting',
  },
])
</script>
