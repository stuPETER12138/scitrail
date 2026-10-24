<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { appInfo, lock, type AppInfo } from '../api/commands'
import { useAuthStore } from '../stores/auth'

const auth = useAuthStore()
const info = ref<AppInfo | null>(null)
const loadError = ref('')

onMounted(async () => {
  try {
    info.value = await appInfo()
  } catch (e) {
    loadError.value = String(e)
  }
})

async function doLock() {
  try {
    await lock()
  } finally {
    auth.unlocked = false
  }
}
</script>

<template>
  <div class="page">
    <h2>设置</h2>

    <section v-if="info">
      <h3>关于</h3>
      <dl>
        <dt>版本</dt>
        <dd>v{{ info.version }}（P0 骨架）</dd>
        <dt>数据库</dt>
        <dd class="path">{{ info.dbPath }}</dd>
        <dt>Schema 版本</dt>
        <dd>v{{ info.schemaVersion }}</dd>
      </dl>
    </section>
    <p v-else-if="loadError" class="placeholder">{{ loadError }}</p>

    <section>
      <h3>安全</h3>
      <button class="lock-btn" @click="doLock">锁定应用</button>
      <p class="note">锁定后需重新输入口令；主窗口关闭时最小化到托盘，托盘菜单可退出。</p>
    </section>

    <section>
      <h3>待接入（P1~P5）</h3>
      <p class="note">备份与恢复 / 全量导出 / 里程碑维护 / 字典与默认值 将在后续阶段进入本页。</p>
    </section>
  </div>
</template>

<style scoped>
section {
  margin-top: 18px;
}

h3 {
  font-size: 13px;
  font-weight: 500;
  color: var(--muted-fg);
  margin: 0 0 8px;
}

dl {
  margin: 0;
  display: grid;
  grid-template-columns: 90px 1fr;
  gap: 6px 12px;
}

dt {
  color: var(--muted-fg);
}

dd {
  margin: 0;
}

.path {
  font-family: ui-monospace, Consolas, monospace;
  font-size: 12px;
  word-break: break-all;
}

.lock-btn {
  padding: 8px 14px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: var(--card);
  cursor: pointer;
}

.lock-btn:hover {
  background: var(--hover);
}

.note {
  color: var(--muted-fg);
  font-size: 12px;
  margin: 8px 0 0;
}
</style>
