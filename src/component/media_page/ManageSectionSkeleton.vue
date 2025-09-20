<template>
  <div class="card from-primary/40 to-secondary/40 mb-8 bg-gradient-to-br p-0.5 shadow-xl">
    <div class="card bg-base-100 sticky top-8 overflow-hidden shadow-lg">
      <div class="card-body p-6">
        <h2 class="mb-4 flex items-center gap-2 text-2xl font-bold">
          <SettingsIcon class="text-primary h-6 w-6" />
          Manage
        </h2>

        <div class="space-y-3">
          <div class="skeleton h-8 w-full"></div>

          <div class="skeleton h-8 w-full"></div>
          <div class="skeleton h-8 w-full"></div>

          <div class="divider my-4"></div>

          <div class="skeleton h-8 w-full"></div>

          <div class="divider my-4"></div>

          <div class="skeleton h-40 w-full"></div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { SettingsIcon } from 'lucide-vue-next'
</script>
