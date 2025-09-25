<template>
  <div class="card from-primary/40 to-secondary/40 mb-8 bg-gradient-to-br p-0.5 shadow-xl">
    <div class="card bg-base-100 overflow-hidden shadow-lg">
      <div class="card-body p-6">
        <div class="flex flex-col gap-6 md:flex-row">
          <div class="skeleton h-96 w-64 rounded-xl"></div>

          <div class="flex-1">
            <div class="skeleton mb-4 h-12 w-64 rounded-xl"></div>

            <h2 class="mb-4 flex items-center gap-2 text-2xl font-bold">
              <InfoIcon class="text-primary h-6 w-6" />
              Overview
            </h2>
            <div class="skeleton mb-4 h-24 w-full"></div>

            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
              <div class="md:col-span-2">
                <h3 class="mb-2 flex items-center gap-2 font-semibold">
                  <UsersIcon class="text-primary h-5 w-5" />
                  Cast
                </h3>
                <div class="flex flex-wrap gap-2">
                  <div v-for="i in 4" :key="i" class="flex flex-col items-center gap-1">
                    <div class="avatar">
                      <div class="skeleton h-12 w-12 rounded-full"></div>
                    </div>
                    <div class="skeleton h-4 w-24"></div>
                  </div>
                </div>
              </div>

              <div>
                <h3 class="mb-2 flex items-center gap-2 font-semibold">
                  <UsersIcon class="text-primary h-5 w-5" />
                  Directors
                </h3>
                <div class="flex flex-wrap gap-2">
                  <div v-for="i in 2" :key="i" class="flex flex-col items-center gap-1">
                    <div class="avatar">
                      <div class="skeleton h-12 w-12 rounded-full"></div>
                    </div>
                    <div class="skeleton h-4 w-24"></div>
                  </div>
                </div>
              </div>

              <div>
                <h3 class="mb-2 flex items-center gap-2 font-semibold">
                  <UsersIcon class="text-primary h-5 w-5" />
                  Writers
                </h3>
                <div class="flex flex-wrap gap-2">
                  <div v-for="i in 2" :key="i" class="flex flex-col items-center gap-1">
                    <div class="avatar">
                      <div class="skeleton h-12 w-12 rounded-full"></div>
                    </div>
                    <div class="skeleton h-4 w-24"></div>
                  </div>
                </div>
              </div>

              <div>
                <h3 class="mb-2 flex items-center gap-2 font-semibold">
                  <GlobeIcon class="text-primary h-5 w-5" />
                  Countries
                </h3>
                <div class="flex flex-wrap gap-2">
                  <div v-for="i in 3" :key="i" class="skeleton h-6 w-24"></div>
                </div>
              </div>

              <div>
                <h3 class="mb-2 flex items-center gap-2 font-semibold">
                  <PuzzleIcon class="text-primary h-5 w-5" />
                  genres
                </h3>
                <div class="flex flex-wrap gap-2">
                  <div v-for="i in 3" :key="i" class="skeleton h-6 w-24"></div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { GlobeIcon, InfoIcon, PuzzleIcon, UsersIcon } from 'lucide-vue-next'
</script>
