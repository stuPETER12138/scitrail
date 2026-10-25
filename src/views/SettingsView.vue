<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import {
  appInfo,
  changePassword,
  exportData,
  listBackups,
  restoreBackup,
  runBackup,
  lock,
  type AppInfo,
  type BackupLog,
} from '../api/commands'
import { useUpdater } from '../api/updater'
import { useAuthStore } from '../stores/auth'

const auth = useAuthStore()
const info = ref<AppInfo | null>(null)
const loadError = ref('')
const message = ref('')

const { state: updState, newVersion, error: updError, checkUpdate, installUpdate, progressPercent } =
  useUpdater()

const updStatus = computed(() => {
  switch (updState.value) {
    case 'checking':
      return '正在检查…'
    case 'available':
      return `发现新版本 v${newVersion.value}，下载安装完成后会自动重启`
    case 'downloading':
      return `正在下载… ${progressPercent()}%`
    case 'uptodate':
      return '已是最新版本'
    case 'failed':
      return `检查失败：${updError.value}`
    default:
      return ''
  }
})

const backups = ref<BackupLog[]>([])
const restoring = ref<string | null>(null)

const pwdOld = ref('')
const pwdNew = ref('')
const pwdMsg = ref('')

onMounted(async () => {
  try {
    info.value = await appInfo()
  } catch (e) {
    loadError.value = String(e)
  }
  await refreshBackups()
})

async function refreshBackups() {
  try {
    backups.value = await listBackups()
  } catch {
    backups.value = []
  }
}

async function doBackup() {
  message.value = ''
  try {
    const name = await runBackup()
    message.value = `备份完成并在资源管理器中定位：${name}`
    await refreshBackups()
  } catch (e) {
    message.value = String(e)
  }
}

async function doRestore(file: string) {
  if (restoring.value !== file) {
    restoring.value = file
    return
  }
  restoring.value = null
  await restoreBackup(file)
  window.location.reload() // 数据库已替换，整页刷新重载全部状态
}

async function doExport(format: 'json' | 'md') {
  message.value = ''
  try {
    const path = await exportData(format)
    message.value = `已导出并在资源管理器中定位：${path}`
  } catch (e) {
    message.value = String(e)
  }
}

async function doChangePassword() {
  pwdMsg.value = ''
  try {
    await changePassword(pwdOld.value, pwdNew.value)
    pwdOld.value = ''
    pwdNew.value = ''
    pwdMsg.value = '口令已更新'
  } catch (e) {
    pwdMsg.value = String(e)
  }
}

async function doLock() {
  try {
    await lock()
  } finally {
    auth.unlocked = false
  }
}

function kb(size: number | null): string {
  if (size == null) return '—'
  return size > 1024 * 1024 ? `${(size / 1024 / 1024).toFixed(1)} MB` : `${Math.round(size / 1024)} KB`
}
</script>

<template>
  <div class="page">
    <h2>设置</h2>

    <section v-if="info">
      <h3>关于</h3>
      <dl>
        <dt>版本</dt>
        <dd>v{{ info.version }}（M1 · P0~P5）</dd>
        <dt>数据库</dt>
        <dd class="path">{{ info.dbPath }}</dd>
        <dt>Schema</dt>
        <dd>v{{ info.schemaVersion }} · SQLite WAL + FTS5</dd>
      </dl>
    </section>
    <p v-else-if="loadError" class="placeholder">{{ loadError }}</p>

    <section>
      <h3>软件更新</h3>
      <div class="btns">
        <button class="btn" type="button" @click="checkUpdate()">检查更新</button>
        <button
          v-if="updState === 'available'"
          class="btn primary"
          type="button"
          @click="installUpdate"
        >
          下载并重启（v{{ newVersion }}）
        </button>
      </div>
      <p v-if="updStatus" class="note">{{ updStatus }}</p>
    </section>

    <section>
      <h3>备份（每日 00:30 自动，保留 30 份）</h3>
      <div class="btns">
        <button class="btn" type="button" @click="doBackup">立即备份</button>
        <button class="btn" type="button" @click="refreshBackups">刷新列表</button>
      </div>
      <div v-if="backups.length" class="bk-list">
        <div v-for="b in backups" :key="b.id" class="bk-row">
          <span class="bk-name">{{ b.file }}</span>
          <span class="bk-meta">{{ kb(b.sizeBytes) }} · {{ b.createdAt }}</span>
          <button class="btn danger" type="button" @click="doRestore(b.file)">
            {{ restoring === b.file ? '确认恢复？（会先备份当前）' : '恢复' }}
          </button>
        </div>
      </div>
      <p v-else class="note">还没有备份——点「立即备份」生成第一份 VACUUM INTO 一致性快照。</p>
    </section>

    <section>
      <h3>全量导出（人类可读副本，与备份互补）</h3>
      <div class="btns">
        <button class="btn" type="button" @click="doExport('json')">导出 JSON</button>
        <button class="btn" type="button" @click="doExport('md')">导出 Markdown</button>
      </div>
    </section>

    <section>
      <h3>口令</h3>
      <form class="pwd" @submit.prevent="doChangePassword">
        <input v-model="pwdOld" type="password" placeholder="当前口令" />
        <input v-model="pwdNew" type="password" placeholder="新口令（至少 4 位）" />
        <button class="btn" type="submit">修改</button>
      </form>
      <p v-if="pwdMsg" class="note">{{ pwdMsg }}</p>
    </section>

    <section>
      <h3>安全</h3>
      <button class="btn" type="button" @click="doLock">锁定应用</button>
      <p class="note">主窗口关闭时最小化到托盘，托盘菜单可退出。</p>
    </section>

    <p v-if="message" class="note msg">{{ message }}</p>
  </div>
</template>

<style scoped>
dl {
  margin: 0;
  display: grid;
  grid-template-columns: 70px 1fr;
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
  font-size: var(--text-sm);
  word-break: break-all;
}

.btns {
  display: flex;
  gap: 8px;
}

.bk-list {
  margin-top: 10px;
  max-height: 220px;
  overflow-y: auto;
  border: 1px solid var(--line);
  border-radius: var(--radius);
  padding: 4px 12px;
}

.bk-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 6px 0;
  border-bottom: 1px dashed var(--line);
  font-size: var(--text-sm);
}

.bk-row:last-child {
  border-bottom: none;
}

.bk-name {
  font-family: ui-monospace, Consolas, monospace;
}

.bk-meta {
  color: var(--muted-fg);
  margin-left: auto;
}

.pwd {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.pwd input {
  padding: 7px 10px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: var(--bg);
  color: var(--fg);
  outline: none;
}
</style>
