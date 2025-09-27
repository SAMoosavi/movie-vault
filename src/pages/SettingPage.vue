<template>
  <div class="bg-base-100 container mx-auto px-4 py-8">
    <div class="flex flex-col gap-6 md:flex-row">
      <!-- Category Panel -->
      <aside class="card from-primary/50 to-secondary/50 mb-8 h-fit bg-gradient-to-br p-0.5 shadow-md">
        <nav class="card bg-base-200 h-fit w-full md:w-64">
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
        </nav>
      </aside>

      <!-- Settings Panel with Animation -->
      <main class="ml-8 flex-1 py-8">
        <transition
          enter-active-class="transition duration-300 ease-out"
          enter-from-class="opacity-0 translate-y-4"
          enter-to-class="opacity-100 translate-y-0"
          leave-active-class="transition duration-200 ease-in"
          leave-from-class="opacity-100 translate-y-0"
          leave-to-class="opacity-0 translate-y-4"
        >
          <router-view />
        </transition>
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { TagsIcon, PaletteIcon, FolderIcon } from 'lucide-vue-next'

// List of settings categories with their icons and routes
const categories = ref([
  { name: 'Tag', icon: TagsIcon, route: 'tags_setting' },
  { name: 'Appearance', icon: PaletteIcon, route: 'appearance_setting' },
  { name: 'Directories', icon: FolderIcon, route: 'directories_setting' },
])
</script>
