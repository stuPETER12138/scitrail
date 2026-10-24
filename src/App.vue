<script setup lang="ts">
import { onMounted } from 'vue'
import { useAuthStore } from './stores/auth'
import Shell from './components/Shell.vue'
import Unlock from './views/Unlock.vue'

const auth = useAuthStore()
onMounted(() => void auth.init())
</script>

<template>
  <Unlock v-if="!auth.unlocked" />
  <Shell v-else />
</template>
