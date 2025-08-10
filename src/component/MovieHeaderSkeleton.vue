<template>
  <div class="card from-primary/50 to-secondary/50 mb-8 w-full bg-gradient-to-br p-0.5 shadow-xl">
    <div class="card lg:card-side bg-base-100">
      <!-- Poster Skeleton -->
      <div class="skeleton w-full lg:w-1/4"></div>

      <div class="card-body">
        <!-- Title and Button Skeleton -->
        <div class="flex items-center justify-between">
          <div class="skeleton h-10 w-3/4 rounded"></div>
          <div class="rounded-btn skeleton h-10 w-1/5"></div>
        </div>

        <!-- Rating and Meta Info Skeleton -->
        <div class="mb-4 flex flex-wrap items-center gap-4">
          <div class="badge badge-lg skeleton w-40"></div>
        </div>

        <!-- Genres Skeleton -->
        <div class="mb-4 flex flex-wrap gap-2">
          <div v-for="i in 3" :key="i" class="badge badge-md skeleton h-6 w-16 rounded"></div>
        </div>

        <!-- Plot Skeleton -->
        <div class="mb-6 space-y-2">
          <div class="skeleton h-4 w-full rounded"></div>
          <div class="skeleton h-4 w-11/12 rounded"></div>
          <div class="skeleton h-4 w-10/12 rounded"></div>
        </div>

        <!-- Cast and Crew Skeleton -->
        <div class="mb-6 grid grid-cols-1 gap-4 md:grid-cols-2">
          <div v-for="i in 4" :key="i">
            <div class="skeleton mb-1 h-4 w-24 rounded"></div>
            <div class="skeleton h-4 w-32 rounded"></div>
          </div>
        </div>

        <!-- Additional Info Skeleton -->
        <div class="grid grid-cols-1 gap-4 text-sm md:grid-cols-3">
          <div v-for="i in 3" :key="i">
            <div class="skeleton mb-1 h-4 w-20 rounded"></div>
            <div class="skeleton h-4 w-28 rounded"></div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
