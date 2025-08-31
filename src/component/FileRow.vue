<template>
  <!-- File Row -->
  <div
    class="odd:bg-base-200/50 even:bg-base-100 hover:bg-base-300 grid grid-cols-1 items-center gap-4 p-3 transition-colors sm:grid-cols-4"
  >
    <!-- File Details -->
    <div class="flex flex-col">
      <div class="font-medium">{{ file.file_name }}.{{ fileExtension }}</div>
      <div class="tooltip tooltip-primary" data-tip="Click to copy path">
        <button
          class="link link-hover max-w-xs truncate text-left transition-all duration-300 hover:max-w-full"
          @click="copyPathToClipboard"
        >
          {{ file.path }}
        </button>
      </div>
    </div>

    <!-- Quality Badge -->
    <div class="flex justify-start sm:justify-center">
      <div class="badge badge-lg badge-outline">
        {{ file.quality || 'N/A' }}
      </div>
    </div>

    <!-- Language Format Badge -->
    <div class="flex justify-start sm:justify-center">
      <div class="badge badge-md badge-primary gap-1">
        <span v-if="file.language_format">{{ file.language_format }}</span>
        <span v-else>unknown</span>
      </div>
    </div>

    <!-- Action Buttons -->
    <div class="flex flex-wrap justify-start gap-1 sm:justify-center">
      <button
        class="btn btn-xs btn-square btn-primary btn-outline tooltip tooltip-top"
        data-tip="Play"
        @click="playFile"
      >
        <Play class="h-3 w-3" />
      </button>
      <button
        class="btn btn-xs btn-square btn-secondary btn-outline tooltip tooltip-top"
        data-tip="Open Location"
        @click="openFileLocation"
      >
        <FolderOpen class="h-3 w-3" />
      </button>
      <button
        class="btn btn-xs btn-square btn-accent btn-outline tooltip tooltip-top"
        data-tip="Move"
        @click="moveFile"
      >
        <Scissors class="h-3 w-3" />
      </button>
      <button class="btn btn-xs btn-square btn-info btn-outline tooltip tooltip-top" data-tip="Copy" @click="copyFile">
        <Files class="h-3 w-3" />
      </button>
      <button
        class="btn btn-xs btn-square btn-error btn-outline tooltip tooltip-top"
        data-tip="Delete"
        @click="deleteFile"
      >
        <Trash2 class="h-3 w-3" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
// --- External types & icons ---
import type { File } from '../type'
import { Files, FolderOpen, Play, Scissors, Trash2 } from 'lucide-vue-next'

// --- Tauri APIs (rename copyFile import to avoid collision with local function) ---
import { basename, dirname, join } from '@tauri-apps/api/path'
import { openPath } from '@tauri-apps/plugin-opener'
import { copyFile as fsCopyFile, rename, remove } from '@tauri-apps/plugin-fs'
import { open } from '@tauri-apps/plugin-dialog'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'

// --- Utilities ---
import { toast } from 'vue3-toastify'

// --- Props & emits ---
const props = defineProps<{ file: File }>()
const emit = defineEmits(['reload'])
const filePath = props.file.path

// --- Helper: Get file extension from path ---
const fileExtension = filePath.split('.').pop() ?? ''

// --- Function: Play the file using system default ---
function playFile() {
  openPath(filePath).catch((e) => console.error('Error playing file:', e))
}

// --- Function: Open the folder containing the file ---
async function openFileLocation() {
  try {
    const dir = await dirname(filePath)
    await openPath(dir)
  } catch (e) {
    console.error('Error opening location:', e)
  }
}

// --- Function: Move the file to a selected directory ---
async function moveFile() {
  try {
    const targetDir = await open({
      directory: true,
      multiple: false,
      title: 'Select target folder',
    })
    if (!targetDir) return

    const fileName = await basename(filePath)
    const targetPath = await join(targetDir, fileName)

    toast.info('Move started...')
    try {
      await rename(filePath, targetPath)
    } catch (err: unknown) {
      // Fallback for cross-device move
      if (String(err).includes('Invalid cross-device link')) {
        await fsCopyFile(filePath, targetPath)
        await remove(filePath)
      } else {
        throw err
      }
    }
    toast.success('File moved successfully')
    emit('reload')
  } catch (e) {
    toast.error('Move failed!')
    console.error('Error moving file:', e)
  }
}

// --- Function: Copy the file to a selected directory ---
async function copyFile() {
  try {
    const targetDir = await open({
      directory: true,
      multiple: false,
      title: 'Select target folder',
    })
    if (!targetDir) return

    const fileName = await basename(filePath)
    const targetPath = await join(targetDir, fileName)

    toast.info('Copy started...')
    await fsCopyFile(filePath, targetPath)
    toast.success('File copied successfully')
  } catch (e) {
    toast.error('Copy failed!')
    console.error('Error copying file:', e)
  }
}

// --- Function: Delete the file ---
function deleteFile() {
  remove(filePath)
    .then(() => {
      toast.success('File deleted successfully')
      emit('reload')
    })
    .catch((e) => toast.error('Error deleting file:', e))
}

// --- Function: Copy file path to clipboard ---
function copyPathToClipboard() {
  writeText(filePath)
    .then(() => toast.success('Path copied to clipboard'))
    .catch((err) => toast.error(`Failed to copy: ${err}`))
}
</script>
