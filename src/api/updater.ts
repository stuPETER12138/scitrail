import { ref } from 'vue'
import { check, type Update } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'

// 自动更新：启动后静默检查 + 设置页手动检查，共享同一份单例状态。
// v0.1.4 起发布的安装包才内置更新器；此前的版本需手动下载覆盖。

export type UpdState =
  | 'idle'
  | 'checking'
  | 'available'
  | 'downloading'
  | 'uptodate'
  | 'failed'

const state = ref<UpdState>('idle')
const newVersion = ref('')
const received = ref(0)
const total = ref(0)
const error = ref('')
const dismissed = ref(false)

let pending: Update | null = null

/** silent=true 用于启动检查：网络失败不打扰用户 */
async function checkUpdate(silent = false): Promise<void> {
  if (state.value === 'checking' || state.value === 'downloading') return
  state.value = 'checking'
  error.value = ''
  try {
    const update = await check()
    if (update) {
      pending = update
      newVersion.value = update.version
      state.value = 'available'
    } else {
      pending = null
      state.value = 'uptodate'
    }
  } catch (e) {
    state.value = 'failed'
    error.value = String(e)
    if (silent) state.value = 'idle'
  }
}

async function installUpdate(): Promise<void> {
  if (!pending || state.value === 'downloading') return
  state.value = 'downloading'
  received.value = 0
  total.value = 0
  try {
    await pending.downloadAndInstall((event) => {
      switch (event.event) {
        case 'Started':
          total.value = event.data.contentLength ?? 0
          break
        case 'Progress':
          received.value += event.data.chunkLength
          break
        case 'Finished':
          break
      }
    })
    await relaunch()
  } catch (e) {
    state.value = 'failed'
    error.value = String(e)
  }
}

/** 用户点「稍后」：本轮不再弹横幅，下次启动重新检查 */
function dismiss() {
  dismissed.value = true
}

function progressPercent(): number {
  return total.value > 0 ? Math.min(100, Math.round((received.value / total.value) * 100)) : 0
}

export function useUpdater() {
  return {
    state,
    newVersion,
    received,
    total,
    error,
    dismissed,
    checkUpdate,
    installUpdate,
    dismiss,
    progressPercent,
  }
}
