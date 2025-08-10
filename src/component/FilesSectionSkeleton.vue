<template>
  <div class="card bg-base-100 shadow-xl">
    <div class="card-body">
      <!-- Header Skeleton -->
      <div class="card-title mb-4 flex items-center">
        <div class="skeleton mr-2 h-6 w-6 rounded-full"></div>
        <div class="skeleton h-6 w-32 rounded"></div>
        <div class="rounded-badge skeleton ml-2 h-5 w-16"></div>
      </div>

      <!-- Table Skeleton -->
      <div class="overflow-x-auto">
        <table class="table">
          <thead>
            <tr>
              <th v-for="i in 5" :key="i" class="skeleton h-4"></th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in 3" :key="row">
              <td>
                <div class="skeleton mb-1 h-4 w-32 rounded"></div>
                <div class="skeleton h-3 w-40 rounded"></div>
              </td>
              <td>
                <div class="rounded-badge skeleton h-6 w-16"></div>
              </td>
              <td>
                <div class="flex gap-1">
                  <div v-for="i in 2" :key="i" class="rounded-badge skeleton h-4 w-12"></div>
                </div>
              </td>
              <td>
                <div class="rounded-badge skeleton h-6 w-12"></div>
              </td>
              <td>
                <div class="flex gap-2">
                  <div v-for="i in 5" :key="i" class="rounded-btn skeleton h-6 w-16"></div>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>
