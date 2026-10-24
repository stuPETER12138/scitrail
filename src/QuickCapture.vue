<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'
import { ref } from 'vue'

const text = ref('')
const win = getCurrentWindow()

function hide() {
  void win.hide()
}

function submit() {
  // P1 接入 quick_parse 速录语法（实/文/钱/会/记/任 + #课题 + !优先级 + 日期时间）
  text.value = ''
  hide()
}
</script>

<template>
  <div class="quick">
    <input
      v-model="text"
      placeholder="速录语法将在 P1 接入（实 / 文 / 钱 / 会 / 记 / 任 + #课题）"
      autofocus
      @keydown.enter.prevent="submit"
      @keydown.esc.prevent="hide"
    />
    <p class="hint">Enter 收起 · Esc 取消 · 全局唤起 Ctrl+Alt+K</p>
  </div>
</template>

<style scoped>
.quick {
  padding: 14px;
}

input {
  width: 100%;
  padding: 9px 12px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: var(--card);
  outline: none;
}

input:focus {
  border-color: var(--accent);
}

.hint {
  margin: 8px 2px 0;
  color: var(--muted-fg);
  font-size: 11px;
}
</style>
