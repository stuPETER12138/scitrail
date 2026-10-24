<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { onChanged } from './api/commands'
import Shell from './components/Shell.vue'
import Unlock from './views/Unlock.vue'
import { useProjectsStore } from './stores/projects'
import { useTasksStore } from './stores/tasks'
import { useAuthStore } from './stores/auth'

const auth = useAuthStore()
const projects = useProjectsStore()
const tasks = useTasksStore()

let unlisten: (() => void) | null = null

onMounted(async () => {
  void auth.init()
  // 速录小窗（独立窗口）保存后广播刷新，主窗口各 store 重载
  unlisten = await onChanged(() => {
    if (!auth.unlocked) return
    void projects.load(true)
    void tasks.refresh()
  })
})

onUnmounted(() => unlisten?.())
</script>

<template>
  <Unlock v-if="!auth.unlocked" />
  <Shell v-else />
</template>
