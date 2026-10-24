<script setup lang="ts">
import { revealItemInDir } from '@tauri-apps/plugin-opener'
import { computed, ref, watch } from 'vue'
import {
  createTask,
  deleteEntry,
  entryHistory,
  type Entry,
  type HistoryItem,
} from '../api/commands'
import EntryForm from './EntryForm.vue'
import { useProjectsStore } from '../stores/projects'
import { KIND_META } from '../stores/trajectory'

const props = defineProps<{ entry: Entry }>()
const emit = defineEmits<{ close: []; changed: [] }>()

const projects = useProjectsStore()
const mode = ref<'view' | 'edit' | 'history'>('view')
const history = ref<HistoryItem[]>([])
const confirmDelete = ref(false)
const message = ref('')
const openError = ref('')

watch(
  () => props.entry.id,
  () => {
    mode.value = 'view'
    confirmDelete.value = false
    message.value = ''
    openError.value = ''
  },
)

const meta = computed(() => KIND_META[props.entry.kind])
const project = computed(() => projects.byId(props.entry.projectId))
const fieldRows = computed(() =>
  Object.entries(props.entry.fields ?? {}).filter(
    ([, v]) => v !== null && v !== undefined && v !== '',
  ),
)

const FIELD_LABELS: Record<string, string> = {
  purpose: '目的',
  params: '参数',
  conditions: '条件',
  sample: '样品',
  device: '设备',
  conclusion: '结论',
  followup: '后续',
  authors: '作者',
  year: '年份',
  venue: '期刊/会议',
  doi: 'DOI/链接',
  note: '笔记',
  amount: '金额',
  reason: '事由',
  status: '状态',
  topic: '主题',
  advisorFeedback: '导师意见',
}

async function loadHistory() {
  mode.value = 'history'
  history.value = await entryHistory(props.entry.id)
}

async function openRef(path: string) {
  openError.value = ''
  try {
    await revealItemInDir(path)
  } catch (e) {
    openError.value = `打开失败：${String(e)}`
  }
}

async function toTask() {
  try {
    await createTask({
      title: props.entry.title,
      projectId: props.entry.projectId,
    })
    message.value = '已转为任务'
    emit('changed')
  } catch (e) {
    message.value = String(e)
  }
}

async function remove() {
  if (!confirmDelete.value) {
    confirmDelete.value = true
    return
  }
  await deleteEntry(props.entry.id)
  emit('changed')
  emit('close')
}

function onSaved() {
  mode.value = 'view'
  emit('changed')
}
</script>

<template>
  <div class="inspector">
    <div class="head">
      <span v-if="meta">{{ meta.icon }} {{ meta.label }}</span>
      <span class="when">{{ entry.date }} {{ entry.time ?? '' }}</span>
      <button class="close" type="button" @click="emit('close')">✕</button>
    </div>

    <!-- 查看模式 -->
    <div v-if="mode === 'view'" class="body">
      <h3 class="title">{{ entry.title }}</h3>
      <div class="chips">
        <span v-if="project" class="chip proj">
          <span class="dot" :style="{ background: project.color ?? '#8a94a0' }" />
          {{ project.name }}
        </span>
        <span v-for="t in entry.tags" :key="t" class="chip">*{{ t }}</span>
      </div>

      <div v-for="[k, v] in fieldRows" :key="k" class="kv">
        <span class="k">{{ FIELD_LABELS[k] ?? k }}</span>
        <span class="v">{{ v }}</span>
      </div>

      <p v-if="entry.content" class="content">{{ entry.content }}</p>

      <div v-if="entry.refs.length" class="refs">
        <div v-for="(r, i) in entry.refs" :key="i" class="ref">
          <span class="ref-label">{{ r.label }}</span>
          <code class="path">{{ r.path }}</code>
          <button type="button" class="mini" @click="openRef(r.path)">📂 打开</button>
        </div>
      </div>
      <p v-if="openError" class="error">{{ openError }}</p>

      <div class="acts">
        <button class="btn" type="button" @click="toTask">转为任务</button>
        <button class="btn" type="button" @click="mode = 'edit'">编辑</button>
        <button class="btn" type="button" @click="loadHistory">历史</button>
        <button class="btn danger" type="button" @click="remove">
          {{ confirmDelete ? '确认删除？' : '删除' }}
        </button>
      </div>
      <p v-if="message" class="msg">{{ message }}</p>
    </div>

    <!-- 编辑模式 -->
    <div v-else-if="mode === 'edit'" class="body">
      <EntryForm :initial="entry" @saved="onSaved" @cancel="mode = 'view'" />
    </div>

    <!-- 历史模式 -->
    <div v-else class="body">
      <div v-if="history.length">
        <div v-for="(h, i) in history" :key="i" class="hist-row">
          <span class="when">{{ h.savedAt }}</span>
          <span class="hist-title">{{ h.snapshot.title ?? '（无标题快照）' }}</span>
        </div>
      </div>
      <p v-else class="dim">没有修改历史</p>
      <button class="btn" type="button" @click="mode = 'view'">返回</button>
    </div>
  </div>
</template>

<style scoped>
.inspector {
  position: fixed;
  top: 0;
  right: 0;
  bottom: 0;
  width: 360px;
  background: var(--card);
  border-left: 1px solid var(--line);
  box-shadow: -8px 0 24px rgba(0, 0, 0, 0.08);
  display: flex;
  flex-direction: column;
  z-index: 40;
}

.head {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--line);
  font-size: 13px;
}

.when {
  color: var(--muted-fg);
  font-size: 12px;
}

.close {
  margin-left: auto;
  border: none;
  background: none;
  cursor: pointer;
  color: var(--muted-fg);
}

.body {
  flex: 1;
  overflow-y: auto;
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.title {
  margin: 0;
  font-size: 15px;
  font-weight: 500;
}

.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
}

.chip {
  font-size: 11px;
  border-radius: 9px;
  padding: 1px 8px;
  background: var(--hover);
  color: var(--muted-fg);
}

.chip.proj {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
}

.kv {
  display: flex;
  gap: 10px;
  font-size: 13px;
  padding: 4px 0;
  border-bottom: 1px dashed var(--line);
}

.kv .k {
  width: 70px;
  flex: none;
  color: var(--muted-fg);
  font-size: 12px;
}

.kv .v {
  flex: 1;
  white-space: pre-wrap;
  word-break: break-word;
}

.content {
  margin: 4px 0 0;
  white-space: pre-wrap;
  font-size: 13px;
  color: var(--fg);
}

.refs {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 4px;
}

.ref {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}

.ref-label {
  color: var(--muted-fg);
  flex: none;
}

.path {
  font-size: 11px;
  background: var(--hover);
  border-radius: 4px;
  padding: 1px 6px;
  word-break: break-all;
}

.mini {
  border: 1px solid var(--line);
  background: var(--bg);
  border-radius: 6px;
  padding: 2px 8px;
  font-size: 12px;
  cursor: pointer;
  flex: none;
}

.acts {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
  margin-top: 8px;
}

.btn {
  border: 1px solid var(--line);
  background: var(--bg);
  border-radius: 8px;
  padding: 6px 12px;
  cursor: pointer;
  font-size: 13px;
}

.btn.danger {
  color: var(--danger);
}

.error,
.msg {
  font-size: 12px;
  margin: 0;
}

.error {
  color: var(--danger);
}

.msg {
  color: var(--ok);
}

.dim {
  color: var(--muted-fg);
  font-size: 13px;
}

.hist-row {
  display: flex;
  gap: 10px;
  padding: 6px 0;
  border-bottom: 1px dashed var(--line);
  font-size: 12px;
}

.hist-title {
  color: var(--muted-fg);
}
</style>
