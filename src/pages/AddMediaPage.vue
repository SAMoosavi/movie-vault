<template>
  <div class="container mx-auto px-4 py-8">
    <div class="mb-8">
      <h1 class="mb-2 text-3xl font-bold">Add Media Manually</h1>
      <p class="text-base-content/70">
        Search for media on IMDb or enter an IMDb ID directly to add it to your collection.
      </p>
    </div>

    <AddMediaManual />
  </div>
</template>

<script setup lang="ts">
import AddMediaManual from '../component/AddMediaManual.vue'
</script>
