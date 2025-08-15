<template>
  <div class="card bg-base-100 shadow-xl">
    <div class="card-body">
      <h2 class="card-title mb-4 flex items-center gap-2">
        <FileText class="h-6 w-6" />
        Available Files
        <div class="badge badge-secondary">{{ movie.files?.length }} files</div>
      </h2>

      <div class="border-base-200 overflow-hidden rounded-lg border shadow-sm">
        <!-- Header -->
        <div class="bg-primary text-primary-content hidden p-3 text-center font-bold sm:grid sm:grid-cols-4">
          <div>File Details</div>
          <div>Quality</div>
          <div>Language Format</div>
          <div>Actions</div>
        </div>

        <!-- Rows -->
        <div
          v-for="file in movie.files"
          :key="file.path"
          class="border-base-200 odd:bg-base-200 even:bg-base-100 hover:bg-base-300 grid grid-cols-1 items-center gap-4 border-t p-3 transition-colors first:border-t-0 sm:grid-cols-4"
        >
          <!-- File Details -->
          <div class="flex flex-col sm:col-span-1">
            <div class="font-medium">{{ file.title }}</div>
            <div class="tooltip tooltip-primary" data-tip="Click to copy path">
              <button
                class="link link-hover max-w-xs truncate text-left transition-all duration-300 hover:max-w-full"
                @click="copyToClipboard(file.path)"
              >
                {{ file.path }}
              </button>
            </div>
          </div>

          <!-- Quality -->
          <div class="flex justify-start sm:justify-center">
            <div class="badge badge-lg badge-outline">
              {{ file.quality || 'N/A' }}
            </div>
          </div>

          <!-- Language Format -->
          <div class="flex justify-start sm:justify-center">
            <div class="badge badge-md badge-primary gap-1">
              <span v-if="file.language_format">{{ file.language_format }}</span>
              <span v-else>unknown</span>
            </div>
          </div>

          <!-- Actions -->
          <div class="flex flex-wrap justify-start gap-1 sm:justify-center">
            <button
              class="btn btn-xs btn-square btn-primary btn-outline tooltip tooltip-top"
              data-tip="Play"
              @click="playFile(file.path)"
            >
              <Play class="h-3 w-3" />
            </button>

            <button
              class="btn btn-xs btn-square btn-secondary btn-outline tooltip tooltip-top"
              data-tip="Open Location"
              @click="openFileLocation(file.path)"
            >
              <FolderOpen class="h-3 w-3" />
            </button>

            <button
              class="btn btn-xs btn-square btn-accent btn-outline tooltip tooltip-top"
              data-tip="Move"
              @click="MoveFile(file.path)"
            >
              <Scissors class="h-3 w-3" />
            </button>

            <button
              class="btn btn-xs btn-square btn-info btn-outline tooltip tooltip-top"
              data-tip="Copy"
              @click="CopyFile(file.path)"
            >
              <Files class="h-3 w-3" />
            </button>

            <button
              class="btn btn-xs btn-square btn-error btn-outline tooltip tooltip-top"
              data-tip="Delete"
              @click="DeleteFile(file.path)"
            >
              <Trash2 class="h-3 w-3" />
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Media } from '../type'
import { Files, FileText, FolderOpen, Play, Scissors, Trash2 } from 'lucide-vue-next'
import { basename, dirname, join } from '@tauri-apps/api/path'
import { openPath } from '@tauri-apps/plugin-opener'
import { copyFile, rename, remove } from '@tauri-apps/plugin-fs'
import { open } from '@tauri-apps/plugin-dialog'
import { toast } from 'vue3-toastify'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'

defineProps<{ movie: Media }>()
const emit = defineEmits(['reload'])

function playFile(path: string) {
  openPath(path).catch((e) => console.error('Error playing file:', e))
}

async function openFileLocation(path: string) {
  try {
    const dir = await dirname(path)
    await openPath(dir)
  } catch (e) {
    console.error('Error opening location:', e)
  }
}

async function MoveFile(path: string) {
  try {
    const targetDir = await open({
      directory: true,
      multiple: false,
      title: 'Select target folder',
    })
    if (!targetDir) return

    const fileName = await basename(path)
    const targetPath = await join(targetDir, fileName)

    toast.info('Move started...')

    try {
      await rename(path, targetPath)
    } catch (err: unknown) {
      if (String(err).includes('Invalid cross-device link')) {
        await copyFile(path, targetPath)
        await remove(path)
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

async function CopyFile(path: string) {
  try {
    const targetDir = await open({
      directory: true,
      multiple: false,
      title: 'Select target folder',
    })
    if (!targetDir) return

    const fileName = await basename(path)
    const targetPath = await join(targetDir, fileName)

    toast.info('copy started...')
    await copyFile(path, targetPath)
    toast.success('File copied successfully')
  } catch (e) {
    toast.error('copy Failed!')
    console.error('Error copying file:', e)
  }
}

function DeleteFile(path: string) {
  remove(path)
    .then(() => {
      toast.success('File deleted successfully')
      emit('reload')
    })
    .catch((e) => toast.error('Error deleting file:', e))
}

function copyToClipboard(text: string) {
  writeText(text)
    .then(() => toast.success('Path copied to clipboard'))
    .catch((err) => toast.error(`Failed to copy: ${err}`))
}
</script>
