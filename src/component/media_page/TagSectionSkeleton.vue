<template>
  <!-- Outer card with gradient border -->
  <div class="card from-primary/40 to-secondary/40 mb-8 bg-gradient-to-br p-0.5 shadow-xl">
    <div class="card bg-base-100 overflow-hidden shadow-lg">
      <div class="card-body p-6">
        <div class="mb-4 flex flex-col sm:flex-row sm:items-center sm:justify-between">
          <h2 class="mb-3 flex items-center gap-2 text-2xl font-bold sm:mb-0">
            <TagIcon class="text-primary h-6 w-6" />
            Tags
          </h2>

          <div class="skeleton h-8 w-40"></div>
        </div>

        <div class="flex min-h-[40px] flex-wrap gap-2">
          <span v-for="i in 4" :key="i" class="skeleton h-6 w-20"> </span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { TagIcon } from 'lucide-vue-next'
</script>
