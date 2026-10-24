<script setup lang="ts">
import { emit } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { computed, ref, watch } from 'vue'
import { createEntry, createTask, parseQuick, type ParsedDraft } from './api/commands'

const KIND_LABELS: Record<string, string> = {
  task: '✅ 任务',
  experiment: '🧪 实验',
  literature: '📖 文献',
  expense: '💰 报销',
  meeting: '📅 组会',
  note: '📝 笔记',
}

const win = getCurrentWindow()
const text = ref('')
const draft = ref<ParsedDraft | null>(null)
const error = ref('')
const saving = ref(false)
let timer: number | undefined

watch(text, (v) => {
  window.clearTimeout(timer)
  if (!v.trim()) {
    draft.value = null
    return
  }
  timer = window.setTimeout(async () => {
    try {
      draft.value = await parseQuick(v, 'task')
      error.value = ''
    } catch (e) {
      error.value = String(e)
    }
  }, 200)
})

const canSave = computed(() => !!draft.value && draft.value.title.trim().length > 0)

function hide() {
  text.value = ''
  draft.value = null
  void win.hide()
}

async function save() {
  const d = draft.value
  if (!d || !canSave.value || saving.value) return
  saving.value = true
  try {
    if (d.kind === 'task') {
      await createTask({
        title: d.title,
        projectId: d.projectMatched,
        priority: d.priority ?? 2,
        dueDate: d.date,
      })
    } else {
      await createEntry({
        kind: d.kind,
        title: d.title,
        date: d.date,
        time: d.time,
        projectId: d.projectMatched,
        tags: d.tags,
      })
    }
    await emit('scitrail://changed')
    text.value = ''
    draft.value = null
    void win.hide()
  } catch (e) {
    error.value = String(e)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="quick">
    <input
      v-model="text"
      placeholder="速录：实 完成#催化 1150℃×2h 明天 9:00"
      autofocus
      @keydown.enter.prevent="save"
      @keydown.esc.prevent="hide"
    />
    <div v-if="draft" class="preview">
      <span class="chip kind">{{ KIND_LABELS[draft.kind] ?? draft.kind }}</span>
      <span v-if="draft.title" class="title-preview">{{ draft.title }}</span>
      <span v-if="draft.projectName" class="chip" :class="{ ok: draft.projectMatched }">
        #{{ draft.projectName }}{{ draft.projectMatched ? '' : '（未匹配）' }}
      </span>
      <span v-if="draft.date" class="chip">📅 {{ draft.date }}</span>
      <span v-if="draft.time" class="chip">⏰ {{ draft.time }}</span>
      <span v-if="draft.priority === 0" class="chip hot">!高</span>
      <span v-else-if="draft.priority === 1" class="chip warm">!中</span>
      <span v-for="t in draft.tags" :key="t" class="chip">*{{ t }}</span>
    </div>
    <p v-if="error" class="error">{{ error }}</p>
    <p class="hint">Enter 保存 · Esc 取消 · 前缀 实/文/钱/会/记/任 · #课题 · !高中低 · 明天 9:00</p>
  </div>
</template>

<style scoped>
.quick {
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
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

.preview {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
  align-items: center;
  min-height: 20px;
}

.chip {
  font-size: var(--text-xs);
  border-radius: 9px;
  padding: 1px 8px;
  background: var(--hover);
  color: var(--muted-fg);
}

.chip.kind {
  background: color-mix(in srgb, var(--accent) 14%, transparent);
  color: var(--accent);
}

.chip.ok {
  background: color-mix(in srgb, var(--ok) 14%, transparent);
  color: var(--ok);
}

.chip.hot {
  background: color-mix(in srgb, var(--danger) 14%, transparent);
  color: var(--danger);
}

.chip.warm {
  background: color-mix(in srgb, var(--warn) 14%, transparent);
  color: var(--warn);
}

.title-preview {
  font-size: var(--text-sm);
  font-weight: 500;
}

.error {
  margin: 0;
  color: var(--danger);
  font-size: var(--text-xs);
}

.hint {
  margin: 0;
  color: var(--muted-fg);
  font-size: var(--text-xs);
}
</style>
