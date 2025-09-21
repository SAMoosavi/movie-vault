<template>
  <LoadingResult />
  <div class="flex flex-wrap justify-center gap-2">
    <LoadingCard v-for="i in 10" :key="i" />
  </div>
</template>

<script setup lang="ts">
import LoadingCard from './LoadingCard.vue'
import LoadingResult from './LoadingResult.vue'
</script>
