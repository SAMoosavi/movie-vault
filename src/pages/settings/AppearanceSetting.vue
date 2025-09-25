<template>
  <!-- Appearance Settings Card -->
  <SettingCategoryCard name="Appearance Settings" description="Customize the look and feel of your application">
    <div class="card-body">
      <div class="flex flex-wrap gap-4">
        <!-- Theme selection cards -->
        <div
          v-for="theme in themes"
          :key="theme"
          :data-theme="theme"
          @click="selectTheme(theme)"
          class="card cursor-pointer border transition-all"
          :class="{
            'border-primary ring-primary/50 bg-primary/5 ring-2': selectedTheme === theme,
            'border-base-300 hover:border-base-400': selectedTheme !== theme,
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
// --- Vue & store ---
import { ref, watch, onMounted } from 'vue'
import { Store } from '@tauri-apps/plugin-store'

// --- Theme helpers ---
import { getDefaultTheme, initStore, loadTheme, setTheme, themes } from '../../functions/theme.ts'

// --- Components ---
import SettingCategoryCard from '../../component/SettingCategoryCard.vue'

// --- State ---
let settingsStore: Store | null = null
const selectedTheme = ref('')

function selectTheme(theme: string) {
  selectedTheme.value = theme
}

watch(
  selectedTheme,
  async (newTheme) => {
    if (settingsStore) {
      try {
        await setTheme(newTheme, settingsStore)
      } catch (error) {
        console.error('Failed to save theme:', error)
      }
    }
  },
  { immediate: true },
)

onMounted(async () => {
  settingsStore = await initStore()
  let theme = await loadTheme(settingsStore)
  if (!theme) theme = getDefaultTheme()
  selectedTheme.value = theme
})
</script>
