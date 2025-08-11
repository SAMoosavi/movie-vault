<template>
  <div class="card bg-base-100 shadow-xl">
    <div class="card-body">
      <h2 class="card-title mb-4">
        <FileText class="mr-2 h-6 w-6" />
        Available Files
        <div class="badge badge-secondary">{{ movie.files_data?.length }} files</div>
      </h2>

      <div class="overflow-x-auto">
        <table class="table-zebra table">
          <thead>
            <tr>
              <th>Title</th>
              <th>Quality</th>
              <th>Subtitles</th>
              <th>Dubbed</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="file in movie.files_data" :key="file.path">
              <td>
                <div class="font-medium">{{ file.title }}</div>
                <div class="tooltip tooltip-primary">
                  <div class="tooltip-content">
                    <div class="text-base-content/70 w-full text-xs wrap-break-word">{{ file.path }}</div>
                  </div>
                  <button class="max-w-xs truncate">{{ file.path }}</button>
                </div>
              </td>
              <td>
                <div class="badge badge-outline">{{ file.quality || 'N/A' }}</div>
              </td>
              <td>
                <div class="flex gap-1">
                  <div class="badge badge-sm" :class="file.has_soft_sub ? 'badge-success' : 'badge-ghost'">Soft</div>
                  <div class="badge badge-sm" :class="file.has_hard_sub ? 'badge-success' : 'badge-ghost'">Hard</div>
                </div>
              </td>
              <td>
                <div class="badge" :class="file.is_dubbed ? 'badge-primary' : 'badge-ghost'">
                  {{ file.is_dubbed ? 'Yes' : 'No' }}
                </div>
              </td>
              <td>
                <div class="flex gap-2">
                  <button class="btn btn-xs btn-primary" @click="playFile(file.path)">Play</button>
                  <button class="btn btn-xs btn-secondary" @click="openFileLocation(file.path)">Location</button>
                  <button class="btn btn-xs btn-primary" @click="MoveFile(file.path)">Move</button>
                  <button class="btn btn-xs btn-secondary" @click="CopyFile(file.path)">Copy</button>
                  <button class="btn btn-xs btn-primary" @click="DeleteFile(file.path)">Delete</button>
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
import type { VideoMetaData } from '../type'
import { FileText } from 'lucide-vue-next'
import { basename, dirname, join } from '@tauri-apps/api/path'
import { openPath } from '@tauri-apps/plugin-opener'
import { copyFile, rename, remove } from '@tauri-apps/plugin-fs'
import { open } from '@tauri-apps/plugin-dialog'
import { toast } from 'vue3-toastify'

defineProps<{ movie: VideoMetaData }>()
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

async function DeleteFile(path: string) {
  try {
    await remove(path)
    toast.success('File deleted successfully')
    emit('reload')
  } catch (e) {
    console.error('Error deleting file:', e)
  }
}
</script>
