<script setup lang="ts">
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import MarkdownIt from 'markdown-it'
import { computed, onMounted, ref } from 'vue'
import {
  exportWeekly,
  generateWeekly,
  listWeeklies,
  saveWeekly,
  type WeeklyReport,
} from '../api/commands'
import { addDays, todayStr } from '../utils/date'

const md = new MarkdownIt()

const reports = ref<WeeklyReport[]>([])
const selected = ref('')
const content = ref('')
const editing = ref(false)
const dirty = ref(false)
const message = ref('')
const busy = ref(false)

function mondayOf(base: string): string {
  const [y, m, d] = base.split('-').map(Number)
  const idx = (new Date(y, m - 1, d).getDay() + 6) % 7
  return addDays(base, -idx)
}

const thisMonday = mondayOf(todayStr())
const mondays = Array.from({ length: 8 }, (_, i) => addDays(thisMonday, -7 * i))

const current = computed(() => reports.value.find((r) => r.weekStart === selected.value))
const html = computed(() => md.render(content.value || '（空——点击「生成」从本周数据拼装草稿）'))

function label(ws: string): string {
  if (ws === thisMonday) return `本周（${ws} 起）`
  return `${ws} 起`
}

async function select(ws: string) {
  selected.value = ws
  editing.value = false
  dirty.value = false
  message.value = ''
  const r = reports.value.find((x) => x.weekStart === ws)
  content.value = r?.content ?? ''
}

async function refreshList() {
  reports.value = await listWeeklies()
}

onMounted(async () => {
  await refreshList()
  await select(thisMonday)
})

async function doGenerate() {
  if (busy.value) return
  if (dirty.value && !window.confirm('有未保存的修改，重新生成将覆盖，继续？')) return
  busy.value = true
  try {
    content.value = await generateWeekly(selected.value)
    dirty.value = false
    editing.value = true
    message.value = '已生成草稿，可直接编辑后保存'
    await refreshList()
  } finally {
    busy.value = false
  }
}

async function doSave() {
  if (busy.value || !dirty.value) return
  busy.value = true
  try {
    await saveWeekly(selected.value, content.value)
    dirty.value = false
    message.value = '已保存'
    await refreshList()
  } finally {
    busy.value = false
  }
}

async function doCopy() {
  await writeText(content.value)
  message.value = '已复制到剪贴板'
}

async function doExport() {
  if (busy.value) return
  busy.value = true
  try {
    if (dirty.value) await saveWeekly(selected.value, content.value)
    const path = await exportWeekly(selected.value)
    message.value = `已导出并在资源管理器中定位：${path}`
  } catch (e) {
    message.value = String(e)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <div class="page">
    <h2>周报</h2>

    <div class="bar">
      <select v-model="selected" class="week" @change="select(($event.target as HTMLSelectElement).value)">
        <option v-for="m in mondays" :key="m" :value="m">{{ label(m) }}</option>
      </select>
      <button class="btn primary" type="button" :disabled="busy" @click="doGenerate">
        {{ current ? '重新生成' : '生成本周草稿' }}
      </button>
      <button class="btn" type="button" @click="editing = !editing">
        {{ editing ? '预览' : '编辑' }}
      </button>
      <button class="btn" type="button" :disabled="!dirty || busy" @click="doSave">保存</button>
      <button class="btn" type="button" :disabled="!content" @click="doCopy">复制全文</button>
      <button class="btn" type="button" :disabled="!content || busy" @click="doExport">导出 .md</button>
    </div>

    <p v-if="current" class="meta">
      生成于 {{ current.generatedAt }}
      <template v-if="current.editedAt"> · 编辑于 {{ current.editedAt }}</template>
    </p>
    <p class="meta dim">草稿从本周完成任务、实验/文献/报销/组会记录、下周到期任务与 DDL 自动拼装</p>

    <textarea
      v-if="editing"
      v-model="content"
      class="editor"
      spellcheck="false"
      @input="dirty = true"
    />
    <div v-else class="preview md" v-html="html" />

    <p v-if="message" class="msg">{{ message }}</p>
    <p v-if="dirty" class="msg warn">● 有未保存的修改</p>
  </div>
</template>

<style scoped>
.bar {
  display: flex;
  gap: 8px;
  margin: 14px 0 6px;
  flex-wrap: wrap;
}

.week {
  padding: 7px 10px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: var(--bg);
  color: var(--fg);
}

.btn {
  border: 1px solid var(--line);
  background: var(--card);
  border-radius: 8px;
  padding: 7px 14px;
  cursor: pointer;
}

.btn.primary {
  background: var(--accent);
  border-color: var(--accent);
  color: var(--accent-fg);
}

.btn:disabled {
  opacity: 0.55;
  cursor: default;
}

.meta {
  font-size: 12px;
  color: var(--muted-fg);
  margin: 4px 0;
}

.editor {
  width: 100%;
  min-height: 480px;
  margin-top: 10px;
  padding: 14px;
  border: 1px solid var(--line);
  border-radius: var(--radius);
  background: var(--card);
  color: var(--fg);
  font-family: ui-monospace, Consolas, 'Microsoft YaHei', monospace;
  font-size: 13px;
  line-height: 1.7;
  resize: vertical;
  outline: none;
}

.editor:focus {
  border-color: var(--accent);
}

.preview {
  margin-top: 10px;
  border: 1px solid var(--line);
  border-radius: var(--radius);
  background: var(--card);
  padding: 18px 22px;
  min-height: 300px;
  line-height: 1.8;
}

.msg {
  font-size: 12px;
  color: var(--ok);
  margin: 10px 0 0;
}

.msg.warn {
  color: var(--warn);
}

:deep(.md h1) {
  font-size: 18px;
  margin: 0 0 12px;
}

:deep(.md h2) {
  font-size: 15px;
  margin: 18px 0 6px;
}

:deep(.md h3) {
  font-size: 13px;
  margin: 12px 0 4px;
  color: var(--muted-fg);
}

:deep(.md ul) {
  padding-left: 20px;
  margin: 4px 0;
}

:deep(.md li) {
  margin: 3px 0;
  font-size: 13px;
}
</style>
