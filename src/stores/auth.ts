import { defineStore } from 'pinia'
import { ref } from 'vue'
import { hasPassword, unlock as invokeUnlock } from '../api/commands'

export const useAuthStore = defineStore('auth', () => {
  const ready = ref(false)
  const unlocked = ref(false)
  const passwordExists = ref(false)

  async function init() {
    try {
      passwordExists.value = await hasPassword()
    } catch (e) {
      console.error('后端探测失败：', e)
    } finally {
      ready.value = true
    }
  }

  async function unlock(password: string): Promise<boolean> {
    const ok = await invokeUnlock(password)
    if (ok) unlocked.value = true
    return ok
  }

  return { ready, unlocked, passwordExists, init, unlock }
})
