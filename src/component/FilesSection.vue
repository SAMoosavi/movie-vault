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
                <div class="text-base-content/70 max-w-xs truncate text-sm">{{ file.path }}</div>
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
                  <button class="btn btn-xs btn-primary" @click="openFileLocation(file.path)">Move</button>
                  <button class="btn btn-xs btn-secondary" @click="openFileLocation(file.path)">Copy</button>
                  <button class="btn btn-xs btn-primary" @click="openFileLocation(file.path)">Delete</button>
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
import { dirname } from '@tauri-apps/api/path'
import { openPath } from '@tauri-apps/plugin-opener'

defineProps<{ movie: VideoMetaData }>()

function playFile(path: string) {
  // Implement play functionality
  console.log('Playing file:', path)
  // You can use Tauri's shell API to open the file with default player
  openPath(path).catch((e) => console.log(e))
}

async function openFileLocation(path: string) {
  try {
    // Implement open file location functionality
    const dir = await dirname(path)
    console.log('Opening file location:', path)
    console.log('Opening dir location:', dir)
    // You can use Tauri's shell API to open the directory
    await openPath(dir)
  } catch (e) {
    console.log(e)
  }
}
</script>
