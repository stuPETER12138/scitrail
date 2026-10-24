<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import {
  createPaper,
  deletePaper,
  listPapers,
  updatePaper,
  type Paper,
} from '../api/commands'
import { useProjectsStore } from '../stores/projects'
import { useTimeStore } from '../stores/time'
import { dueInfo, todayStr } from '../utils/date'

const STAGES = ['构思', '撰写中', '组内审', '投稿中', '审稿中', '返修', '录用', '见刊']
const STAGE_COLORS: Record<string, string> = {
  构思: '#8a94a0',
  撰写中: '#3b7dd8',
  组内审: '#0ea5a4',
  投稿中: '#8b5cf6',
  审稿中: '#d97706',
  返修: '#e0533d',
  录用: '#2f9e68',
  见刊: '#64748b',
}

const papers = ref<Paper[]>([])
const projects = useProjectsStore()
const time = useTimeStore()

async function reload() {
  papers.value = await listPapers()
  // 放榜 DDL 随阶段联动，顺手刷新时间线数据
  await time.load(true)
}

onMounted(() => {
  void reload()
  void projects.load()
})

const byStage = (stage: string) => papers.value.filter((p) => p.stage === stage)

/** 审稿中超预计放榜 → 该催了 */
function isUrgent(p: Paper): boolean {
  return (
    p.stage === '审稿中' &&
    !!p.expectReply &&
    p.expectReply.slice(0, 10) < todayStr()
  )
}

async function onDrop(stage: string, e: DragEvent) {
  const id = Number(e.dataTransfer?.getData('text/plain'))
  const p = papers.value.find((x) => x.id === id)
  if (!p || p.stage === stage) return
  await updatePaper(p.id, {
    title: p.title,
    venue: p.venue,
    stage,
    submitDate: p.submitDate,
    expectReply: p.expectReply,
    reviseDeadline: p.reviseDeadline,
    projectId: p.projectId,
    note: p.note,
  })
  await reload()
}

// ---- 编辑抽屉 ----
const editing = ref<Paper | null>(null)
const creating = ref(false)
const error = ref('')
const confirmDelete = ref(false)

const form = reactive({
  title: '',
  venue: '',
  stage: '构思',
  submitDate: '',
  expectReply: '',
  reviseDeadline: '',
  projectId: null as number | null,
  note: '',
})

watch(
  [editing, creating],
  () => {
    const p = editing.value
    form.title = p?.title ?? ''
    form.venue = p?.venue ?? ''
    form.stage = p?.stage ?? '构思'
    form.submitDate = p?.submitDate?.slice(0, 10) ?? ''
    form.expectReply = p?.expectReply?.slice(0, 10) ?? ''
    form.reviseDeadline = p?.reviseDeadline?.slice(0, 10) ?? ''
    form.projectId = p?.projectId ?? null
    form.note = p?.note ?? ''
    error.value = ''
    confirmDelete.value = false
  },
  { immediate: true },
)

function openCreate() {
  editing.value = null
  creating.value = true
}

function openEdit(p: Paper) {
  creating.value = false
  editing.value = p
}

function closeDrawer() {
  editing.value = null
  creating.value = false
}

const drawerOpen = computed(() => creating.value || !!editing.value)

async function save() {
  if (!form.title.trim()) {
    error.value = '标题不能为空'
    return
  }
  const input = {
    title: form.title.trim(),
    venue: form.venue.trim() || null,
    stage: form.stage,
    submitDate: form.submitDate || null,
    expectReply: form.expectReply || null,
    reviseDeadline: form.reviseDeadline || null,
    projectId: form.projectId,
    note: form.note.trim() || null,
  }
  try {
    if (editing.value) await updatePaper(editing.value.id, input)
    else await createPaper(input)
    closeDrawer()
    await reload()
  } catch (e) {
    error.value = String(e)
  }
}

async function remove() {
  if (!editing.value) return
  if (!confirmDelete.value) {
    confirmDelete.value = true
    return
  }
  await deletePaper(editing.value.id)
  closeDrawer()
  await reload()
}
</script>

<template>
  <div class="page wide">
    <div class="head">
      <h2>论文流水线</h2>
      <span class="hint">拖动卡片流转阶段 · 投稿中/审稿中/返修 阶段会按「预计放榜」自动生成 DDL 提醒</span>
      <button class="btn primary" type="button" @click="openCreate">＋ 新建论文</button>
    </div>

    <div class="board">
      <div
        v-for="st in STAGES"
        :key="st"
        class="col"
        @dragover.prevent
        @drop.prevent="onDrop(st, $event)"
      >
        <div class="col-h" :style="{ color: STAGE_COLORS[st] }">{{ st }} · {{ byStage(st).length }}</div>
        <div
          v-for="p in byStage(st)"
          :key="p.id"
          class="card"
          draggable="true"
          @dragstart="(e) => e.dataTransfer?.setData('text/plain', String(p.id))"
          @click="openEdit(p)"
        >
          <div class="c-title">{{ p.title }}</div>
          <div v-if="p.venue" class="c-venue">{{ p.venue }}</div>
          <div class="c-badges">
            <span v-if="isUrgent(p)" class="badge urgent">⚠ 该催了</span>
            <span
              v-else-if="p.expectReply"
              class="badge"
              :class="dueInfo(p.expectReply).cls"
            >
              放榜 {{ dueInfo(p.expectReply).label }}
            </span>
            <span v-if="p.reviseDeadline" class="badge warn">返修 {{ p.reviseDeadline.slice(5) }}</span>
            <span v-if="p.submitDate" class="badge dim">投 {{ p.submitDate.slice(5) }}</span>
          </div>
        </div>
        <p v-if="!byStage(st).length" class="col-empty">—</p>
      </div>
    </div>

    <!-- 编辑/新建抽屉 -->
    <div v-if="drawerOpen" class="mask" @click.self="closeDrawer">
      <div class="drawer">
        <div class="d-head">
          <span>{{ editing ? '编辑论文' : '新建论文' }}</span>
          <button class="close" type="button" @click="closeDrawer">✕</button>
        </div>
        <div class="d-body">
          <label class="f span2">
            <span>标题（暂定名亦可）</span>
            <input v-model="form.title" placeholder="如：MoS₂ 边界缺陷调控（暂定）" />
          </label>
          <label class="f span2">
            <span>目标期刊 / 会议</span>
            <input v-model="form.venue" placeholder="如：ACS Applied Materials & Interfaces" />
          </label>
          <label class="f">
            <span>阶段</span>
            <select v-model="form.stage">
              <option v-for="st in STAGES" :key="st">{{ st }}</option>
            </select>
          </label>
          <label class="f">
            <span>关联课题</span>
            <select v-model="form.projectId">
              <option :value="null">不关联</option>
              <option v-for="pr in projects.projects" :key="pr.id" :value="pr.id">
                {{ pr.name }}
              </option>
            </select>
          </label>
          <label class="f">
            <span>投稿日期</span>
            <input v-model="form.submitDate" type="date" />
          </label>
          <label class="f">
            <span>预计放榜</span>
            <input v-model="form.expectReply" type="date" />
          </label>
          <label class="f span2">
            <span>返修截止</span>
            <input v-model="form.reviseDeadline" type="date" />
          </label>
          <label class="f span2">
            <span>备注（版本 / 审稿人意见摘要）</span>
            <textarea v-model="form.note" rows="3" />
          </label>
          <p v-if="error" class="error">{{ error }}</p>
        </div>
        <div class="d-foot">
          <button class="btn danger" type="button" @click="remove">
            {{ confirmDelete ? '确认删除？' : '删除' }}
          </button>
          <button class="btn primary" type="button" @click="save">保存</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.page.wide {
  max-width: none;
  padding: 18px 20px;
}

.head {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.head h2 {
  font-size: var(--text-lg);
  font-weight: 500;
  margin: 0;
}

.hint {
  color: var(--muted-fg);
  font-size: var(--text-sm);
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

.btn.danger {
  color: var(--danger);
}

.board {
  display: flex;
  gap: 10px;
  margin-top: 14px;
  overflow-x: auto;
  padding-bottom: 10px;
}

.col {
  min-width: 190px;
  flex: 1 0 190px;
  background: color-mix(in srgb, var(--card) 55%, transparent);
  border: 1px solid var(--line);
  border-radius: var(--radius);
  padding: 10px;
  min-height: 260px;
}

.col-h {
  font-size: var(--text-sm);
  font-weight: 500;
  padding: 2px 4px 8px;
}

.card {
  background: var(--card);
  border: 1px solid var(--line);
  border-radius: 8px;
  padding: 9px 10px;
  margin-bottom: 8px;
  cursor: grab;
}

.card:hover {
  border-color: var(--accent);
}

.c-title {
  font-size: var(--text-md);
  font-weight: 500;
  margin-bottom: 3px;
}

.c-venue {
  font-size: var(--text-xs);
  color: var(--muted-fg);
  margin-bottom: 6px;
}

.c-badges {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.badge {
  font-size: var(--text-2xs);
  border-radius: 8px;
  padding: 1px 7px;
  background: var(--hover);
  color: var(--muted-fg);
}

.badge.urgent {
  background: var(--danger);
  color: #fff;
  font-weight: 500;
}

.badge.overdue {
  background: color-mix(in srgb, var(--danger) 14%, transparent);
  color: var(--danger);
}

.badge.soon,
.badge.warn {
  background: color-mix(in srgb, var(--warn) 14%, transparent);
  color: var(--warn);
}

.badge.today {
  background: color-mix(in srgb, var(--danger) 12%, transparent);
  color: var(--danger);
}

.badge.dim {
  opacity: 0.8;
}

.col-empty {
  color: var(--muted-fg);
  text-align: center;
  font-size: var(--text-xs);
  opacity: 0.5;
}

/* 抽屉 */
.mask {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
  display: flex;
  justify-content: flex-end;
  z-index: 50;
}

.drawer {
  width: 380px;
  max-width: 100%;
  height: 100%;
  background: var(--card);
  border-left: 1px solid var(--line);
  display: flex;
  flex-direction: column;
}

.d-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 13px 16px;
  border-bottom: 1px solid var(--line);
  font-weight: 500;
}

.close {
  border: none;
  background: none;
  color: var(--muted-fg);
  cursor: pointer;
}

.d-body {
  flex: 1;
  overflow-y: auto;
  padding: 14px 16px;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  align-content: start;
}

.span2 {
  grid-column: span 2;
}

.f {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: var(--text-sm);
  color: var(--muted-fg);
}

.f input,
.f select,
.f textarea {
  padding: 7px 9px;
  border: 1px solid var(--line);
  border-radius: 7px;
  background: var(--bg);
  color: var(--fg);
  outline: none;
  font-size: var(--text-md);
}

.f input:focus,
.f select:focus,
.f textarea:focus {
  border-color: var(--accent);
}

.error {
  grid-column: span 2;
  color: var(--danger);
  font-size: var(--text-sm);
  margin: 0;
}

.d-foot {
  display: flex;
  justify-content: space-between;
  padding: 12px 16px;
  border-top: 1px solid var(--line);
}
</style>
