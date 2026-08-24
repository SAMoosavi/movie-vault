<template>
  <SettingCategoryCard name="API Keys" description="Manage your OMDb API keys pool">
    <div class="card-body">
      <section class="mb-8">
        <h2 class="card-title text-xl">Add API Key</h2>
        <p class="text-base-content/60 mt-1 text-sm">
          Get a free key at
          <a href="https://www.omdbapi.com/apikey.aspx" target="_blank" rel="noopener" class="link link-primary"
            >omdbapi.com</a
          >. Keys are tried in order when one is rejected or rate-limited.
        </p>
        <form class="mt-4 flex gap-2" @submit.prevent="handleAddKey">
          <input v-model="newKey" type="text" placeholder="Enter API key" class="input input-bordered w-full" />
          <button type="submit" class="btn btn-primary">
            <PlusIcon class="h-5 w-5" />
            Add
          </button>
        </form>
      </section>

      <section class="mb-8">
        <h2 class="card-title text-xl">Existing Keys</h2>
        <div v-if="apiKeys.length === 0" class="text-base-content/60 mt-3 italic">
          No API keys configured. IMDb lookups will fail until you add one.
        </div>
        <AnimatedList tag="div" class="mt-3 space-y-2">
          <div
            v-for="key in apiKeys"
            :key="key"
            @click="handleRemoveKey(key)"
            class="badge badge-lg badge-outline flex cursor-pointer items-center gap-2 p-3 transition-all"
            title="Click to remove"
          >
            <KeyRound class="h-4 w-4" />
            <span class="font-medium">{{ mask(key) }}</span>
            <Trash2 class="text-error h-4 w-4" />
          </div>
        </AnimatedList>
      </section>
    </div>
  </SettingCategoryCard>
</template>

<script setup lang="ts">
// --- Icons ---
import { KeyRound, PlusIcon, Trash2 } from 'lucide-vue-next'
import { ref } from 'vue'
import { storeToRefs } from 'pinia'

// --- Stores ---
import { useApiKeysStore } from '../../stores/ApiKeys'

// --- Components ---
import AnimatedList from '../../component/AnimatedList.vue'
import SettingCategoryCard from '../../component/SettingCategoryCard.vue'

const apiKeysStore = useApiKeysStore()
const { apiKeys } = storeToRefs(apiKeysStore)
const newKey = ref('')

function mask(key: string): string {
  if (key.length <= 4) return key
  return '*'.repeat(key.length - 4) + key.slice(-4)
}

function handleAddKey() {
  if (apiKeysStore.addKey(newKey.value)) {
    newKey.value = ''
  }
}

function handleRemoveKey(key: string) {
  apiKeysStore.removeKey(key)
}
</script>
