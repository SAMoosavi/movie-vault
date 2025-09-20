<template>
  <div class="card bg-base-100 shadow-xl">
    <div class="card-body">
      <!-- Header Skeleton -->
      <div class="card-title mb-4 flex items-center gap-2">
        <div class="skeleton h-6 w-6 rounded-full"></div>
        <div class="skeleton h-6 w-32 rounded"></div>
        <div class="rounded-badge skeleton h-5 w-16"></div>
      </div>

      <!-- List Skeleton (flex/grid) -->
      <div class="flex flex-col gap-4">
        <div v-for="row in 3" :key="row" class="grid grid-cols-4 items-center gap-4">
          <!-- Column 1 -->
          <div class="skeleton mb-1 h-6 w-96 rounded"></div>

          <!-- Column 2 -->
          <div class="rounded-badge skeleton mx-auto h-6 w-16"></div>

          <!-- Column 3 -->
          <div class="flex gap-1">
            <div class="rounded-badge skeleton mx-auto h-6 w-16"></div>
          </div>

          <!-- Column 4 -->
          <div class="mx-auto flex flex-wrap gap-2">
            <div v-for="i in 5" :key="i" class="skeleton h-6 w-8"></div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
