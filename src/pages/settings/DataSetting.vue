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
import { invoke } from '@tauri-apps/api/core'
import { save, open } from '@tauri-apps/plugin-dialog'
import { readTextFile } from '@tauri-apps/plugin-fs'
import { toast } from 'vue3-toastify'
import SettingCategoryCard from '../../component/SettingCategoryCard.vue'

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

    await invoke('export_data', { filePath })

    toast.success('Export completed successfully!')
  } catch (error) {
    console.error('Export failed:', error)
    toast.error('Export failed: ' + error)
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
    await invoke('import_data', { data: text })

    alert('Data imported successfully!')
  } catch (error) {
    console.error('Import failed:', error)
    alert('Import failed: ' + error)
  } finally {
    isImporting.value = false
  }
}
</script>
