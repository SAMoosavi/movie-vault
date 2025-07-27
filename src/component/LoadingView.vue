<template>
  <LoadingResult />
  <div class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
    <LoadingCard v-for="i in 8" :key="i" />
  </div>
</template>

<script setup lang="ts">
import LoadingCard from './LoadingCard.vue'
import LoadingResult from './LoadingResult.vue'
</script>
