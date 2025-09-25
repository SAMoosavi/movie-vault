<!-- eslint-disable vue/multi-word-component-names -->
<template>
  <div v-if="show" class="modal modal-open">
    <div class="modal-box">
      <h3 class="text-lg font-bold">{{ title }}</h3>
      <p class="py-4">{{ message }}</p>
      <div class="modal-action">
        <button class="btn" @click="$emit('cancel')">Cancel</button>
        <button class="btn btn-error" @click="$emit('confirm')">Delete</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  show: boolean
  title: string
  message: string
}>()

defineEmits<{
  confirm: []
  cancel: []
}>()
</script>
