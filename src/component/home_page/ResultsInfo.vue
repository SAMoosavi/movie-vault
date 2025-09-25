<template>
  <div class="mb-6 ml-auto w-fit">
    <div class="join">
      <button
        @click="isShowCard = true"
        :class="['btn join-item transition-colors duration-200', isShowCard && 'btn-primary']"
      >
        <LayoutGridIcon class="size-6" />
      </button>

      <button
        @click="isShowCard = false"
        :class="['btn join-item transition-colors duration-200', !isShowCard && 'btn-primary']"
      >
        <Rows3Icon class="size-6" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { LayoutGridIcon, Rows3Icon } from 'lucide-vue-next'

const isShowCard = defineModel<boolean>()
</script>
