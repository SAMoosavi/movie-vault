<template>
  <SettingCategoryCard name="Appearance Settings" description="Customize the look and feel of your application">
    <div class="card-body">
      <div class="flex flex-wrap gap-4">
        <div
          v-for="theme in themes"
          :key="theme"
          :data-theme="theme"
          @click="themeName = theme"
          class="card cursor-pointer border transition-all"
          :class="{
            'border-primary ring-primary/50 bg-primary/5 ring-2': themeName === theme,
            'border-base-300 hover:border-base-400': themeName !== theme,
          }"
        >
          <div class="card-body p-3">
            <div class="grid grid-cols-2 gap-1">
              <div class="bg-primary h-4 w-4 rounded-full"></div>
              <div class="bg-secondary h-4 w-4 rounded-full"></div>
              <div class="bg-accent h-4 w-4 rounded-full"></div>
              <div class="bg-neutral h-4 w-4 rounded-full"></div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </SettingCategoryCard>
</template>
<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { Store } from '@tauri-apps/plugin-store'
import { getDefaultTheme, initStore, loadTheme, setTheme, themes } from '../../functions/theme.ts'
import SettingCategoryCard from '../../component/SettingCategoryCard.vue'

let store: Store | null = null
const themeName = ref('')

watch(
  themeName,
  async (newVal) => {
    if (store) {
      try {
        await setTheme(newVal, store)
      } catch (e) {
        console.error('Failed to save theme index:', e)
      }
    }
  },
  { immediate: true },
)

onMounted(async () => {
  store = await initStore()
  let theme = await loadTheme(store)
  if (!theme) {
    theme = getDefaultTheme()
  }

  themeName.value = theme
})
</script>
