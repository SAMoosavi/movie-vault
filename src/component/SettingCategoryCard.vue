<template>
  <div class="mx-auto">
    <!-- Page Header -->
    <header class="mb-8">
      <h1 class="text-base-content text-3xl font-bold">{{ name }}</h1>
      <p class="text-base-content/60 mt-2">{{ description }}</p>
    </header>

    <div class="card from-primary/50 to-secondary/50 mb-8 h-fit bg-gradient-to-br p-0.5 shadow-xl">
      <div class="card bg-base-200">
        <slot />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{ name: string; description: string }>()
</script>
