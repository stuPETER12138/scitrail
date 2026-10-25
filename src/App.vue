<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'
import { onChanged } from './api/commands'
import { useUpdater } from './api/updater'
import Shell from './components/Shell.vue'
import Unlock from './views/Unlock.vue'
import { useProjectsStore } from './stores/projects'
import { useTasksStore } from './stores/tasks'
import { useTimeStore } from './stores/time'
import { useAuthStore } from './stores/auth'

const auth = useAuthStore()
const projects = useProjectsStore()
const tasks = useTasksStore()
const time = useTimeStore()

const { state, newVersion, dismissed, checkUpdate, installUpdate, dismiss, progressPercent } =
  useUpdater()

const showBanner = computed(
  () =>
    auth.unlocked &&
    !dismissed.value &&
    (state.value === 'available' || state.value === 'downloading'),
)

let unlisten: (() => void) | null = null

onMounted(async () => {
  void auth.init()
  // 速录小窗（独立窗口）保存后广播刷新，主窗口各 store 重载
  unlisten = await onChanged(() => {
    if (!auth.unlocked) return
    void projects.load(true)
    void tasks.refresh()
    void time.load(true)
  })
  // 启动 5 秒后静默检查更新；发现新版本在右下角弹横幅
  setTimeout(() => void checkUpdate(true), 5000)
})

onUnmounted(() => unlisten?.())
</script>

<template>
  <Unlock v-if="!auth.unlocked" />
  <Shell v-else />

  <div v-if="showBanner" class="upd-banner">
    <span v-if="state === 'downloading'">
      正在下载 v{{ newVersion }}… {{ progressPercent() }}%
    </span>
    <template v-else>
      <span>发现新版本 v{{ newVersion }}</span>
      <button class="btn primary" type="button" @click="installUpdate">立即更新</button>
      <button class="btn" type="button" @click="dismiss">稍后</button>
    </template>
  </div>
</template>

<style scoped>
.upd-banner {
  position: fixed;
  right: 16px;
  bottom: 16px;
  z-index: 90;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  border: 1px solid var(--accent);
  border-radius: var(--radius);
  background: var(--card);
  box-shadow: 0 6px 24px rgba(0, 0, 0, 0.14);
  font-size: var(--text-md);
}

.upd-banner .btn {
  padding: 4px 10px;
  font-size: var(--text-sm);
}
</style>
