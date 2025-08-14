<template>
  <div class="card bg-base-100 shadow-xl">
    <div class="card-body">
      <h2 class="card-title mb-4">
        <FileText class="mr-2 h-6 w-6" />
        Available Files
        <div class="badge badge-secondary">{{ movie.files?.length }} files</div>
      </h2>
      <div class="border-base-200 overflow-x-auto rounded-lg border shadow-sm">
        <table class="table-zebra table">
          <!-- Header with improved styling -->
          <thead class="bg-primary text-primary-content">
            <tr>
              <th class="text-center">File Details</th>
              <th class="text-center">Quality</th>
              <th class="text-center">Language Format</th>
              <th class="text-center">Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="file in movie.files" :key="file.path" class="group hover:bg-base-300">
              <!-- File Details Column -->
              <td>
                <div class="font-medium">{{ file.title }}</div>
                <div class="tooltip tooltip-primary" data-tip="Click to copy path">
                  <button
                    class="link link-hover max-w-xs truncate transition-all duration-300 group-hover:max-w-full"
                    @click="copyToClipboard(file.path)"
                  >
                    {{ file.path }}
                  </button>
                </div>
              </td>

              <!-- Quality Column -->
              <td class="text-center">
                <div class="badge badge-lg badge-outline">
                  {{ file.quality || 'N/A' }}
                </div>
              </td>

              <!-- Type Column -->
              <td class="text-center">
                <div class="badge badge-md badge-primary gap-1">
                  <span v-if="file.language_format"> {{ file.language_format }} </span>
                  <span v-else> unknown </span>
                </div>
              </td>

              <!-- Actions Column -->
              <td class="py-3">
                <div class="flex flex-wrap items-center justify-center gap-1">
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
              </td>
            </tr>
          </tbody>
        </table>
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
