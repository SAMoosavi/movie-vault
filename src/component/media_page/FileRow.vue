<template>
  <!-- File Row -->
  <div
    class="odd:bg-base-200/50 even:bg-base-100 hover:bg-base-300 flex flex-wrap items-center justify-between gap-4 px-10 py-3 transition-colors"
  >
    <!-- File Details -->
    <div class="flex flex-col">
      <div class="tooltip tooltip-primary" data-tip="Click to copy path">
        <button
          class="link link-hover max-w-full truncate text-left text-wrap transition-all duration-300"
          @click="copyPathToClipboard"
        >
          {{ file.path }}
        </button>
      </div>
    </div>

    <div class="flex flex-wrap gap-4">
      <!-- Quality Badge -->
      <div v-if="file.quality" class="flex justify-start sm:justify-center">
        <div class="badge badge-lg badge-outline">
          {{ file.quality }}
        </div>
      </div>

      <!-- Language Format Badge -->
      <div
        v-if="file.language_format && file.language_format !== 'Unknown'"
        class="flex justify-start sm:justify-center"
      >
        <div class="badge badge-md badge-primary gap-1">
          {{ file.language_format }}
        </div>
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
import type { File } from '../../type'
import { Files, FolderOpen, Play, Scissors, Trash2 } from 'lucide-vue-next'

// --- Tauri APIs (rename copyFile import to avoid collision with local function) ---
import { basename, dirname } from '@tauri-apps/api/path'
import { openPath } from '@tauri-apps/plugin-opener'
import { copyFile as fsCopyFile, rename, remove } from '@tauri-apps/plugin-fs'
import { save } from '@tauri-apps/plugin-dialog'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'

// --- Utilities ---
import { toast } from 'vue3-toastify'
import { getErrorMessage } from '@/functions/errorMessage'
import { handleFrontendError } from '@/functions/errorHandling'

// --- Props & emits ---
const props = defineProps<{ file: File }>()
const emit = defineEmits<{
  (e: 'reload'): void
}>()
const filePath = props.file.path

// --- Function: Play the file using system default ---
function playFile() {
  void openPath(filePath).catch((error) => handleFrontendError('media.file.play', error, 'Failed to play file'))
}

// --- Function: Open the folder containing the file ---
async function openFileLocation() {
  try {
    const dir = await dirname(filePath)
    await openPath(dir)
  } catch (error) {
    handleFrontendError('media.file.open_location', error, 'Failed to open file location')
  }
}

// --- Function: Move the file to a selected location ---
async function moveFile() {
  try {
    const fileName = await basename(filePath)

    const targetPath = await save({
      defaultPath: fileName,
      title: 'Select target location for file move',
    })

    if (!targetPath) return

    // Prevent moving to the same location
    if (targetPath === filePath) {
      toast.warning('Source and destination are the same')
      return
    }

    toast.info('Moving file...')

    try {
      await rename(filePath, targetPath)
    } catch (err: unknown) {
      const errorMessage = getErrorMessage(err)
      // Handle cross-device moves (different filesystems/drives)
      if (
        errorMessage.includes('Invalid cross-device link') ||
        errorMessage.includes('cross-device') ||
        errorMessage.includes('EXDEV')
      ) {
        // Copy then delete for cross-device moves
        await fsCopyFile(filePath, targetPath)
        await remove(filePath)
      } else {
        throw err
      }
    }

    toast.success('File moved successfully')
    emit('reload')
  } catch (error) {
    handleFrontendError('media.file.move', error, 'Move failed')
  }
}

// --- Function: Copy the file to a selected location ---
async function copyFile() {
  try {
    const fileName = await basename(filePath)

    const targetPath = await save({
      defaultPath: fileName,
      title: 'Select target location for file copy',
    })

    if (!targetPath) return

    // Prevent copying to the same location
    if (targetPath === filePath) {
      toast.warning('Source and destination are the same')
      return
    }

    toast.info('Copying file...')

    await fsCopyFile(filePath, targetPath)

    toast.success('File copied successfully')
  } catch (error) {
    handleFrontendError('media.file.copy', error, 'Copy failed')
  }
}

// --- Function: Delete the file ---
async function deleteFile() {
  try {
    await remove(filePath)
    toast.success('File deleted successfully')
    emit('reload')
  } catch (error) {
    handleFrontendError('media.file.delete', error, 'Failed to delete file')
  }
}

// --- Function: Copy file path to clipboard ---
async function copyPathToClipboard() {
  try {
    await writeText(filePath)
    toast.success('Path copied to clipboard')
  } catch (error) {
    handleFrontendError('media.file.copy_path', error, 'Failed to copy path')
  }
}
</script>
