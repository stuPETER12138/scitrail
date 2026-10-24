<script setup lang="ts">
import { computed, ref } from 'vue'
import { useAuthStore } from '../stores/auth'

const auth = useAuthStore()
const pw = ref('')
const pw2 = ref('')
const error = ref('')
const busy = ref(false)
const isCreate = computed(() => !auth.passwordExists)

async function submit() {
  if (busy.value) return
  error.value = ''
  if (pw.value.length < 4) {
    error.value = '口令至少 4 位'
    return
  }
  if (isCreate.value && pw.value !== pw2.value) {
    error.value = '两次输入的口令不一致'
    return
  }
  busy.value = true
  try {
    const ok = await auth.unlock(pw.value)
    if (!ok) error.value = '口令不正确'
  } catch (e) {
    error.value = String(e)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <div class="unlock-wrap">
    <div class="unlock-card">
      <div class="logo">🧭</div>
      <h1>科迹 SciTrail</h1>
      <p class="hint">{{ isCreate ? '首次使用，请创建解锁口令' : '输入口令解锁' }}</p>
      <form @submit.prevent="submit">
        <input v-model="pw" type="password" :placeholder="isCreate ? '设置口令（至少 4 位）' : '口令'" autofocus />
        <input v-if="isCreate" v-model="pw2" type="password" placeholder="再输入一次" />
        <button type="submit" :disabled="busy || !auth.ready">
          {{ busy ? '校验中…' : isCreate ? '创建并进入' : '解锁' }}
        </button>
      </form>
      <p v-if="error" class="error">{{ error }}</p>
      <p v-if="!auth.ready" class="hint dim">正在连接本地数据库…</p>
    </div>
  </div>
</template>

<style scoped>
.unlock-wrap {
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
}

.unlock-card {
  width: 320px;
  text-align: center;
  background: var(--card);
  border: 1px solid var(--line);
  border-radius: var(--radius);
  padding: 28px 26px;
}

.logo {
  font-size: var(--text-2xl);
}

h1 {
  font-size: var(--text-lg);
  font-weight: 500;
  margin: 8px 0 2px;
}

.hint {
  color: var(--muted-fg);
  font-size: var(--text-sm);
  margin: 0 0 16px;
}

.hint.dim {
  margin: 12px 0 0;
}

form {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

input {
  padding: 9px 12px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: var(--bg);
  outline: none;
}

input:focus {
  border-color: var(--accent);
}

button {
  padding: 9px 12px;
  border: none;
  border-radius: 8px;
  background: var(--accent);
  color: var(--accent-fg);
  cursor: pointer;
}

button:disabled {
  opacity: 0.6;
  cursor: default;
}

.error {
  color: var(--danger);
  font-size: var(--text-sm);
  margin: 12px 0 0;
}
</style>
