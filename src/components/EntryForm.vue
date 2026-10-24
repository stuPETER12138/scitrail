<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { createEntry, updateEntry, type Entry, type RefItem } from '../api/commands'
import { useProjectsStore } from '../stores/projects'
import { KIND_META } from '../stores/trajectory'
import { todayStr } from '../utils/date'

const props = defineProps<{ initial?: Entry | null }>()
const emit = defineEmits<{ saved: []; cancel: [] }>()

const projects = useProjectsStore()
const error = ref('')
const saving = ref(false)

interface FieldDef {
  key: string
  label: string
  area?: boolean
}

const FIELD_DEFS: Record<string, FieldDef[]> = {
  experiment: [
    { key: 'purpose', label: '目的' },
    { key: 'params', label: '参数' },
    { key: 'conditions', label: '条件' },
    { key: 'sample', label: '样品' },
    { key: 'device', label: '设备' },
    { key: 'conclusion', label: '结论', area: true },
    { key: 'followup', label: '后续', area: true },
  ],
  literature: [
    { key: 'authors', label: '作者' },
    { key: 'year', label: '年份' },
    { key: 'venue', label: '期刊/会议' },
    { key: 'doi', label: 'DOI/链接' },
    { key: 'note', label: '一句话笔记' },
  ],
  expense: [
    { key: 'amount', label: '金额' },
    { key: 'reason', label: '事由' },
  ],
  meeting: [
    { key: 'topic', label: '主题' },
    { key: 'advisorFeedback', label: '导师意见', area: true },
  ],
  note: [],
}

const EXPENSE_STATUS = ['垫付中', '已提交', '已到账']

function nowHHMM(): string {
  const n = new Date()
  return `${String(n.getHours()).padStart(2, '0')}:${String(n.getMinutes()).padStart(2, '0')}`
}

const form = reactive({
  kind: 'note',
  date: todayStr(),
  time: nowHHMM(),
  title: '',
  projectId: null as number | null,
  tagsText: '',
  content: '',
  expenseStatus: '垫付中',
  fields: {} as Record<string, string>,
  refs: [] as RefItem[],
})

watch(
  () => props.initial,
  (e) => {
    if (e) {
      form.kind = e.kind
      form.date = e.date
      form.time = e.time ?? ''
      form.title = e.title
      form.projectId = e.projectId
      form.tagsText = e.tags.join(' ')
      form.content = e.content
      const f = (e.fields ?? {}) as Record<string, unknown>
      form.fields = Object.fromEntries(
        Object.entries(f).map(([k, v]) => [k, typeof v === 'string' ? v : JSON.stringify(v)]),
      )
      form.expenseStatus = (f.status as string) ?? '垫付中'
      form.refs = e.refs.map((r) => ({ ...r }))
    }
  },
  { immediate: true },
)

const defs = computed(() => FIELD_DEFS[form.kind] ?? [])

async function save() {
  if (!form.title.trim()) {
    error.value = '标题不能为空'
    return
  }
  if (saving.value) return
  saving.value = true
  try {
    const fields: Record<string, string> = {}
    for (const d of defs.value) {
      const v = (form.fields[d.key] ?? '').trim()
      if (v) fields[d.key] = v
    }
    if (form.kind === 'expense') fields.status = form.expenseStatus
    const refs = form.refs.filter((r) => r.label.trim() && r.path.trim())
    const input = {
      kind: form.kind,
      title: form.title.trim(),
      date: form.date || todayStr(),
      time: form.time.trim() || null,
      content: form.content.trim() || null,
      projectId: form.projectId,
      tags: form.tagsText
        .split(/[\s,，]+/)
        .map((t) => t.replace(/^\*/, ''))
        .filter(Boolean),
      fields,
      refs,
    }
    if (props.initial) await updateEntry(props.initial.id, input)
    else await createEntry(input)
    emit('saved')
  } catch (e) {
    error.value = String(e)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <form class="entry-form" @submit.prevent="save">
    <div class="row head">
      <select v-model="form.kind" class="kind">
        <option v-for="(m, k) in KIND_META" :key="k" v-show="k !== 'task' && k !== 'ddl'" :value="k">
          {{ m.icon }} {{ m.label }}
        </option>
      </select>
      <input v-model="form.date" type="date" class="dt" />
      <input v-model="form.time" type="time" class="dt" />
      <input v-model="form.title" placeholder="标题（必填）" class="title" autofocus />
    </div>

    <div class="row">
      <select v-model="form.projectId">
        <option :value="null">不关联课题</option>
        <option v-for="p in projects.projects" :key="p.id" :value="p.id">{{ p.name }}</option>
      </select>
      <input v-model="form.tagsText" placeholder="标签（空格分隔）" />
      <template v-if="form.kind === 'expense'">
        <select v-model="form.expenseStatus">
          <option v-for="s in EXPENSE_STATUS" :key="s">{{ s }}</option>
        </select>
      </template>
    </div>

    <div v-if="defs.length" class="fields">
      <label v-for="d in defs" :key="d.key" class="f">
        <span>{{ d.label }}</span>
        <textarea v-if="d.area" v-model="form.fields[d.key]" rows="2" />
        <input v-else v-model="form.fields[d.key]" />
      </label>
    </div>

    <label class="f">
      <span>正文（Markdown）</span>
      <textarea v-model="form.content" rows="3" placeholder="详细记录，可空" />
    </label>

    <div class="refs">
      <div class="refs-h">
        <span>数据/文件路径</span>
        <button type="button" class="mini" @click="form.refs.push({ label: '', path: '' })">
          ＋ 添加
        </button>
      </div>
      <div v-for="(r, i) in form.refs" :key="i" class="ref-row">
        <input v-model="r.label" placeholder="名称（如 XPS 原始数据）" />
        <input v-model="r.path" placeholder="路径 \\server\data 或 D:\data" class="path" />
        <button type="button" class="mini del" @click="form.refs.splice(i, 1)">删</button>
      </div>
    </div>

    <p v-if="error" class="error">{{ error }}</p>
    <div class="actions">
      <button class="btn primary" type="submit" :disabled="saving">
        {{ initial ? '保存修改' : '保存' }}
      </button>
      <button class="btn" type="button" @click="emit('cancel')">取消</button>
    </div>
  </form>
</template>

<style scoped>
.entry-form {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.row {
  display: flex;
  gap: 8px;
}

.row.head .title {
  flex: 1;
}

.row select,
.row input,
.refs input,
.f input,
.f textarea {
  padding: 7px 9px;
  border: 1px solid var(--line);
  border-radius: 7px;
  background: var(--bg);
  color: var(--fg);
  outline: none;
  font-size: 13px;
}

.row.head .kind {
  width: 110px;
}

.dt {
  width: 130px;
}

:focus-visible {
  border-color: var(--accent);
}

.fields {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px 10px;
}

.f {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
  color: var(--muted-fg);
}

.refs-h {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
  color: var(--muted-fg);
}

.ref-row {
  display: flex;
  gap: 6px;
  margin-top: 6px;
}

.ref-row .path {
  flex: 1;
  font-family: ui-monospace, Consolas, monospace;
  font-size: 12px;
}

.mini {
  border: 1px solid var(--line);
  background: var(--bg);
  border-radius: 6px;
  padding: 2px 8px;
  font-size: 12px;
  cursor: pointer;
  color: var(--muted-fg);
}

.mini.del:hover {
  color: var(--danger);
}

.error {
  color: var(--danger);
  font-size: 12px;
  margin: 0;
}

.actions {
  display: flex;
  gap: 8px;
}

.btn {
  border: 1px solid var(--line);
  background: var(--bg);
  border-radius: 8px;
  padding: 7px 16px;
  cursor: pointer;
}

.btn.primary {
  background: var(--accent);
  border-color: var(--accent);
  color: var(--accent-fg);
}
</style>
