<template>
  <div class="mb-6 flex items-center justify-between">
    <h2 class="text-xl font-bold">{{ totalMovies }} {{ totalMovies === 1 ? 'Result' : 'Results' }}</h2>
    <div class="text-base-content/50 text-sm">
      Showing {{ numberOfSearchedMovies }} of {{ totalMovies }} media items
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{ totalMovies: number; numberOfSearchedMovies: number }>()
</script>
