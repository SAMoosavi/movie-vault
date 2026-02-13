<template>
  <SettingCategoryCard name="Data Management" description="Export / Import all your movie data for backup.">
    <div class="card-body flex w-full flex-row">
      <button class="btn btn-primary basis-1/2" @click="exportData" :disabled="isExporting">
        <span v-if="isExporting" class="loading loading-spinner loading-sm"></span>
        {{ isExporting ? 'Exporting...' : 'Export Data' }}
      </button>
      <button class="btn btn-secondary basis-1/2" @click="importData" :disabled="isImporting">
        <span v-if="isImporting" class="loading loading-spinner loading-sm"></span>
        {{ isImporting ? 'Importing...' : 'Import Data' }}
      </button>
    </div>
  </SettingCategoryCard>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { save, open } from '@tauri-apps/plugin-dialog'
import { readTextFile } from '@tauri-apps/plugin-fs'
import { toast } from 'vue3-toastify'
import SettingCategoryCard from '@/component/SettingCategoryCard.vue'
import { export_data, import_data } from '@/functions/invoker'
import { getErrorMessage } from '@/functions/errorMessage'

const isExporting = ref(false)
const isImporting = ref(false)

const exportData = async () => {
  try {
    // Open save dialog
    const filePath = await save({
      filters: [
        {
          name: 'JSON',
          extensions: ['json'],
        },
      ],
      defaultPath: `movie-vault-data-${new Date().toISOString().split('T')[0]}.json`,
    })

    if (!filePath) {
      return // User cancelled
    }

    isExporting.value = true

    await export_data(filePath)

    toast.success('Export completed successfully!')
  } catch (error: unknown) {
    const message = getErrorMessage(error)
    console.error('Export failed:', message, error)
    toast.error('Export failed: ' + message)
  } finally {
    isExporting.value = false
  }
}

const importData = async () => {
  try {
    // Open file dialog
    const filePath = await open({
      multiple: false,
      filters: [
        {
          name: 'JSON',
          extensions: ['json'],
        },
      ],
    })

    if (!filePath || Array.isArray(filePath)) {
      return // User cancelled or selected multiple files
    }

    isImporting.value = true

    // Read file content
    const text = await readTextFile(filePath)

    // Import data
    await import_data(text)

    toast.success('Data imported successfully!')
  } catch (error: unknown) {
    const message = getErrorMessage(error)
    console.error('Import failed:', message, error)
    toast.error('Import failed: ' + message)
  } finally {
    isImporting.value = false
  }
}
</script>
