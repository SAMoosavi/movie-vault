<template>
  <Transition
    :enter-active-class="enterActive"
    :enter-from-class="enterFrom"
    :enter-to-class="enterTo"
    :leave-active-class="leaveActive"
    :leave-from-class="leaveFrom"
    :leave-to-class="leaveTo"
  >
    <slot />
  </Transition>
</template>

<script setup lang="ts">
import { defineProps } from 'vue'

defineProps({
  enterActive: { type: String, default: 'transition-all duration-250 ease-out' },
  enterFrom: { type: String, default: 'opacity-0 -translate-y-2 scale-95' },
  enterTo: { type: String, default: 'opacity-100 translate-y-0 scale-100' },
  leaveActive: { type: String, default: 'transition-all duration-200 ease-in' },
  leaveFrom: { type: String, default: 'opacity-100 translate-y-0 scale-100' },
  leaveTo: { type: String, default: 'opacity-0 -translate-y-2 scale-95' },
})
</script>
