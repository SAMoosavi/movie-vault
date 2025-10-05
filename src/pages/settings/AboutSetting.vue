<template>
  <!-- About Settings Card -->
  <SettingCategoryCard name="About" description="Information about the application and developer">
    <div class="card-body">
      <h2 class="card-title mb-4 text-2xl">About {{ appInfo.name }}</h2>
      <p class="text-base-content/70">{{ appInfo.description }}</p>

      <div class="my-4 flex flex-col gap-4 sm:flex-row">
        <div class="form-control">
          <label class="label cursor-pointer">
            <span class="label-text mr-4">Auto Update</span>
            <input type="checkbox" class="toggle toggle-primary" v-model="autoUpdate" />
          </label>
        </div>

        <div class="form-control">
          <label class="label cursor-pointer">
            <span class="label-text mr-4">Install Beta Version</span>
            <input type="checkbox" class="toggle toggle-secondary" v-model="betaVersions" />
          </label>
        </div>
      </div>

      <div class="card-actions items-center gap-4">
        <button class="btn btn-primary" @click="checkNow">
          <svg xmlns="http://www.w3.org/2000/svg" class="mr-2 h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
            <path
              fill-rule="evenodd"
              d="M4 2a1 1 0 011 1v2.101a7.002 7.002 0 0111.601 2.566 1 1 0 11-1.885.666A5.002 5.002 0 005.999 7H9a1 1 0 010 2H4a1 1 0 01-1-1V3a1 1 0 011-1zm.008 9.057a1 1 0 011.276.61A5.002 5.002 0 0014.001 13H11a1 1 0 110-2h5a1 1 0 011 1v5a1 1 0 11-2 0v-2.101a7.002 7.002 0 01-11.601-2.566 1 1 0 01.61-1.276z"
              clip-rule="evenodd"
            />
          </svg>
          Check for Updates
        </button>
        <div class="badge badge-outline">Version {{ appInfo.version }}</div>
      </div>
    </div>
    <div class="divider"></div>

    <div class="card-body">
      <h2 class="card-title mb-4 text-2xl">About Me</h2>
      <p class="text-base-content/70">{{ developerInfo.description }}</p>

      <div class="card-actions mt-4 justify-center gap-4">
        <a :href="developerInfo.github" target="_blank" class="btn btn-outline btn-primary btn-circle">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="currentColor">
            <path
              d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"
            />
          </svg>
        </a>

        <a :href="developerInfo.telegram" target="_blank" class="btn btn-outline btn-secondary btn-circle">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 24 24" fill="currentColor">
            <path
              d="M9.78 18.65l.28-4.23 7.68-6.92c.34-.31-.07-.46-.52-.19L7.74 13.3 3.64 12c-.88-.25-.89-.86.2-1.3l15.97-5.85c.73-.27 1.37.17 1.13.94l-2.67 12.61c-.24 1.05-1.01 1.31-1.63.82l-4.49-3.31-2.17 2.08c-.22.22-.42.42-.83.42z"
            />
          </svg>
        </a>
      </div>
    </div>
  </SettingCategoryCard>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import * as packageData from '../../../package.json'

// --- Components ---
import SettingCategoryCard from '../../component/SettingCategoryCard.vue'

// --- Update functions ---
import { loadUpdateSettings, saveUpdateSettings, checkForUpdates, handleUpdateFound } from '../../functions/update.ts'
import { toast } from 'vue3-toastify'

// --- App Version ---
const appVersion = packageData.version

// --- Update Settings ---
const autoUpdate = ref(false)
const betaVersions = ref(false)

const appInfo = {
  name: 'Movie Vault',
  description:
    'Movie Vault is a personal media management application. It helps you organize and manage your movie and TV show collection with ease.',
  version: appVersion,
}

// Developer information
const developerInfo = ref({
  name: 'John Doe',
  description:
    'Passionate developer with expertise in modern web technologies. I love creating intuitive and efficient applications that solve real-world problems.',
  github: 'https://github.com/SAMoosavi',
  telegram: 'https://t.me/s_a_moosavi',
})

watch(autoUpdate, saveSettings)
watch(betaVersions, saveSettings)

async function saveSettings() {
  await saveUpdateSettings({
    autoUpdate: autoUpdate.value,
    betaVersions: betaVersions.value,
  })
}

async function checkNow() {
  try {
    const update = await checkForUpdates()
    if (update) {
      await handleUpdateFound(update)
    } else {
      toast.info('You already have the latest version.')
    }
  } catch (error) {
    toast.error('Failed to check updates: ' + (error instanceof Error ? error.message : String(error)))
  }
}

onMounted(async () => {
  const settings = await loadUpdateSettings()
  autoUpdate.value = settings.autoUpdate
  betaVersions.value = settings.betaVersions
})
</script>
