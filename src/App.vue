<template>
  <AppNavbar @add-dir="addDir" />

  <router-view />

</template>

<script setup lang="ts">
import AppNavbar from './component/AppNavbar.vue'

function addDir(dir: string) {
  console.log(dir)
}
</script>
