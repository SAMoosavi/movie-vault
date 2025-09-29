<template>
  <div class="card bg-base-200 p-6">
    <h2 class="card-title mb-4">Data Management</h2>
    <div class="space-y-4">
      <div class="card bg-base-100 p-4">
        <h3 class="mb-2 text-lg font-semibold">Export Data</h3>
        <p class="text-base-content/70 mb-4 text-sm">
          Export all your movie data to a JSON file for backup or transfer.
        </p>
        <button class="btn btn-primary" @click="exportData" :disabled="isExporting">
          <span v-if="isExporting" class="loading loading-spinner loading-sm"></span>
          {{ isExporting ? 'Exporting...' : 'Export Data' }}
        </button>
      </div>

      <div class="card bg-base-100 p-4">
        <h3 class="mb-2 text-lg font-semibold">Import Data</h3>
        <p class="text-base-content/70 mb-4 text-sm">
          Import movie data from a previously exported JSON file. This will merge with existing data.
        </p>
        <button class="btn btn-secondary" @click="importData" :disabled="isImporting">
          <span v-if="isImporting" class="loading loading-spinner loading-sm"></span>
          {{ isImporting ? 'Importing...' : 'Import Data' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { save, open } from '@tauri-apps/plugin-dialog'
import { readTextFile } from '@tauri-apps/plugin-fs'
import { toast } from 'vue3-toastify'

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
