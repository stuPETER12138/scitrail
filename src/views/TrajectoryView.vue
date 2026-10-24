<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import DayStrip from '../components/DayStrip.vue'
import InspectorDrawer from '../components/InspectorDrawer.vue'
import type { StripTick } from '../components/DayStrip.vue'
import {
  createEntry,
  onChanged,
  parseQuick,
  type Entry,
  type ParsedDraft,
} from '../api/commands'
import { useProjectsStore } from '../stores/projects'
import { useTrajectoryStore, KIND_META } from '../stores/trajectory'
import { addDays, dueInfo, todayStr } from '../utils/date'

const route = useRoute()
const traj = useTrajectoryStore()
const projects = useProjectsStore()

// ---------- 账本滚动与跟随最新 ----------
const scroller = ref<HTMLElement>()
const follow = ref(true)
let unlisten: (() => void) | null = null

function scrollToBottom() {
  const el = scroller.value
  if (el) el.scrollTop = el.scrollHeight
}

function onScroll() {
  const el = scroller.value
  if (!el) return
  follow.value = el.scrollHeight - el.scrollTop - el.clientHeight < 60
}

watch(
  () => traj.days,
  () => {
    if (follow.value) void nextTick(scrollToBottom)
  },
  { deep: true },
)

onMounted(async () => {
  await focusLoad()
  void projects.load()
  void nextTick(scrollToBottom)
  // 速录小窗保存后若本页开着，自动刷新账本
  unlisten = await onChanged(() => void traj.load())
})

/// 回顾页热力图点击跳转：?date=YYYY-MM-DD → 加载覆盖该日的窗口并定位
async function focusLoad() {
  const q = route.query.date
  const focus = typeof q === 'string' && /^\d{4}-\d{2}-\d{2}$/.test(q) ? q : ''
  if (focus) {
    follow.value = false
    await traj.load(addDays(focus, -3))
    await nextTick()
    document.getElementById(`day-${focus}`)?.scrollIntoView({ block: 'start' })
  } else {
    follow.value = true
    await traj.load()
  }
}

watch(
  () => route.query.date,
  () => void focusLoad(),
)

onUnmounted(() => unlisten?.())

// ---------- 速录 composer（本页默认前缀 = 记/笔记） ----------
const text = ref('')
const draft = ref<ParsedDraft | null>(null)
let timer: number | undefined

watch(text, (v) => {
  window.clearTimeout(timer)
  if (!v.trim()) {
    draft.value = null
    return
  }
  timer = window.setTimeout(async () => {
    draft.value = await parseQuick(v, 'note')
  }, 200)
})

async function submitComposer() {
  const d = draft.value
  if (!d || !d.title.trim()) return
  const now = new Date()
  const nowHM = `${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`
  await createEntry({
    kind: d.kind === 'task' ? 'note' : d.kind, // 账本 composer 只建条目；任务速录走 Ctrl+Alt+K
    title: d.title,
    date: d.date ?? todayStr(),
    time: d.time ?? nowHM,
    projectId: d.projectMatched,
    tags: d.tags,
  })
  text.value = ''
  draft.value = null
  follow.value = true
  await traj.load()
}

// ---------- 当日时序条（跟随账本当前聚焦的日期，先支持今天） ----------
const today = todayStr()

const todayTicks = computed<StripTick[]>(() => {
  const day = traj.days.find((d) => d.date === today)
  if (!day) return []
  return day.items
    .filter((it) => it.at)
    .map((it) => ({
      at: it.at,
      kind: it.src === 'entry' ? it.entry!.kind : it.src === 'task_done' ? 'task' : 'ddl',
      title:
        it.src === 'entry'
          ? it.entry!.title
          : it.src === 'task_done'
            ? it.task!.title
            : it.ddl!.name,
    }))
})

// 时间过滤（拖拽框选）
const filterRange = ref<{ from: string; to: string } | null>(null)
const selectedAt = ref('')

function applyFilter(f: { from: string; to: string } | null) {
  filterRange.value = f
}

function inFilter(at: string): boolean {
  if (!filterRange.value) return true
  return !!at && at >= filterRange.value.from && at <= filterRange.value.to
}

function dayItems(date: string) {
  const day = traj.days.find((d) => d.date === date)
  if (!day) return []
  return day.items.filter((it) => inFilter(it.at))
}

async function loadOlder() {
  await traj.loadOlder()
}

// ---------- 检查器 ----------
const selected = ref<Entry | null>(null)

function openEntry(e: Entry) {
  selected.value = e
}

function onInspectorChanged() {
  void traj.load()
}

// 日期标签
function dayLabel(date: string): string {
  if (date === today) return '今天'
  const yst = new Date(Date.now() - 86400000)
  const m = String(yst.getMonth() + 1).padStart(2, '0')
  const d = String(yst.getDate()).padStart(2, '0')
  if (date === `${yst.getFullYear()}-${m}-${d}`) return '昨天'
  const [, mm, dd] = date.split('-')
  const wd = ['日', '一', '二', '三', '四', '五', '六'][
    new Date(`${date}T00:00:00`).getDay()
  ]
  return `${Number(mm)} 月 ${Number(dd)} 日 · 周${wd}`
}
</script>

<template>
  <div class="traj-page">
    <!-- 头部：标题 + composer + 时序条 -->
    <div class="head-area">
      <div class="page-head">
        <h2>轨迹</h2>
        <span class="follow-tag" :class="{ on: follow }">
          {{ follow ? '● 跟随最新' : '已暂停跟随' }}
        </span>
        <button v-if="!follow" class="mini-btn" type="button" @click="follow = true; scrollToBottom()">
          回到最新 ↓
        </button>
      </div>

      <div class="composer">
        <span class="pen">✏️</span>
        <input
          v-model="text"
          placeholder="记一笔…（实/文/钱/会/记 前缀，#课题 *标签 明天 9:00）"
          @keydown.enter.prevent="submitComposer"
        />
        <div v-if="draft" class="chips">
          <span v-if="draft.kind !== 'note'" class="chip kind">
            {{ KIND_META[draft.kind]?.icon }} {{ KIND_META[draft.kind]?.label }}
          </span>
          <span v-if="draft.title" class="chip title-chip">{{ draft.title }}</span>
          <span v-if="draft.projectName" class="chip" :class="{ ok: draft.projectMatched }">
            #{{ draft.projectName }}
          </span>
          <span v-if="draft.date" class="chip">📅 {{ draft.date }}</span>
          <span v-if="draft.time" class="chip">⏰ {{ draft.time }}</span>
          <span v-for="t in draft.tags" :key="t" class="chip">*{{ t }}</span>
        </div>
      </div>

      <DayStrip
        :ticks="todayTicks"
        :active="!!filterRange"
        @seek="(at) => (selectedAt = at)"
        @filter="applyFilter"
      />
      <div v-if="filterRange" class="filter-bar">
        已过滤 {{ filterRange.from }}–{{ filterRange.to }}
        <button class="mini-btn" type="button" @click="filterRange = null">✕ 清除</button>
      </div>
    </div>

    <!-- 账本 -->
    <div ref="scroller" class="ledger" @scroll="onScroll">
      <button
        class="load-older"
        type="button"
        :disabled="traj.loading || traj.noMore"
        @click="loadOlder"
      >
        {{ traj.noMore ? '已到最早' : traj.loading ? '加载中…' : '↑ 加载更早 7 天' }}
      </button>

      <section v-for="day in traj.days" :id="`day-${day.date}`" :key="day.date" class="day">
        <div class="day-h">
          <span class="dlabel">{{ dayLabel(day.date) }}</span>
          <span class="ddate">{{ day.date }}</span>
          <span class="dcount">{{ dayItems(day.date).length }} 条</span>
        </div>

        <template v-for="(it, i) in dayItems(day.date)" :key="`${it.src}-${i}`">
          <!-- DDL 提醒（投影） -->
          <div v-if="it.src === 'ddl' && it.ddl" class="row ddl-row">
            <span class="ico">⏰</span>
            <span class="t ddl-text">
              DDL 逼近：{{ it.ddl.name }} · {{ dueInfo(it.ddl.dueDate).label }}
            </span>
          </div>
          <!-- 任务完成（投影） -->
          <div v-else-if="it.src === 'task_done' && it.task" class="row">
            <span class="ico" style="color: #2f9e68">✓</span>
            <span class="time">{{ it.at }}</span>
            <span class="t">任务完成：{{ it.task.title }}</span>
            <span v-if="it.task.priority === 0" class="pri">!高</span>
          </div>
          <!-- 条目 -->
          <div
            v-else-if="it.entry"
            class="row entry-row"
            :class="{ selected: selected?.id === it.entry.id }"
            @click="openEntry(it.entry!)"
          >
            <span class="ico">{{ KIND_META[it.entry.kind]?.icon ?? '📝' }}</span>
            <span class="time">{{ it.at || '--:--' }}</span>
            <span class="t">{{ it.entry.title }}</span>
            <span v-if="projects.byId(it.entry.projectId)" class="proj">
              <span class="dot" :style="{ background: projects.byId(it.entry.projectId)!.color ?? '#8a94a0' }" />
              {{ projects.byId(it.entry.projectId)!.name }}
            </span>
            <span v-for="tg in it.entry.tags.slice(0, 2)" :key="tg" class="tag">*{{ tg }}</span>
          </div>
        </template>
      </section>

      <p v-if="!traj.days.length" class="empty-ledger">
        还没有记录 —— 用上方输入框或 Ctrl+Alt+K 记第一笔
      </p>
    </div>

    <InspectorDrawer
      v-if="selected"
      :entry="selected"
      @close="selected = null"
      @changed="onInspectorChanged"
    />
  </div>
</template>

<style scoped>
.traj-page {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.head-area {
  padding: 16px 24px 0;
  border-bottom: 1px solid var(--line);
  background: var(--bg);
}

.page-head {
  display: flex;
  align-items: center;
  gap: 12px;
}

.page-head h2 {
  font-size: var(--text-lg);
  font-weight: 500;
  margin: 0;
}

.follow-tag {
  font-size: var(--text-xs);
  color: var(--muted-fg);
}

.follow-tag.on {
  color: var(--ok);
}

.mini-btn {
  border: 1px solid var(--line);
  background: var(--card);
  border-radius: 6px;
  padding: 1px 8px;
  font-size: var(--text-xs);
  cursor: pointer;
  color: var(--muted-fg);
}

.composer {
  position: relative;
  margin-top: 10px;
}

.composer .pen {
  position: absolute;
  left: 10px;
  top: 8px;
  font-size: var(--text-md);
}

.composer input {
  width: 100%;
  padding: 8px 12px 8px 30px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: var(--card);
  outline: none;
}

.composer input:focus {
  border-color: var(--accent);
}

.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
  margin-top: 6px;
  min-height: 18px;
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

.title-chip {
  font-weight: 500;
  color: var(--fg);
}

.filter-bar {
  margin: 8px 0 0;
  font-size: var(--text-sm);
  color: var(--accent);
  display: flex;
  align-items: center;
  gap: 8px;
  padding-bottom: 8px;
}

/* 账本 */
.ledger {
  flex: 1;
  overflow-y: auto;
  padding: 12px 24px 24px;
}

.load-older {
  display: block;
  width: 100%;
  text-align: center;
  color: var(--muted-fg);
  border: 1px dashed var(--line);
  border-radius: 8px;
  padding: 6px;
  margin-bottom: 12px;
  background: none;
  font-size: var(--text-sm);
  cursor: pointer;
}

.load-older:disabled {
  cursor: default;
  opacity: 0.6;
}

.day {
  margin-bottom: 8px;
}

.day-h {
  display: flex;
  align-items: baseline;
  gap: 10px;
  padding: 8px 2px 4px;
  position: sticky;
  top: 0;
  background: var(--bg);
  border-bottom: 1px solid var(--line);
  z-index: 5;
}

.dlabel {
  font-weight: 500;
  font-size: var(--text-md);
}

.ddate,
.dcount {
  color: var(--muted-fg);
  font-size: var(--text-xs);
}

.row {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 7px 8px;
  border-bottom: 1px solid var(--line);
  font-size: var(--text-md);
}

.row .ico {
  width: 20px;
  text-align: center;
  flex: none;
}

.row .time {
  color: var(--muted-fg);
  width: 44px;
  flex: none;
  font-variant-numeric: tabular-nums;
}

.row .t {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.entry-row {
  cursor: pointer;
  border-radius: 8px;
}

.entry-row:hover {
  background: var(--hover);
}

.entry-row.selected {
  background: color-mix(in srgb, var(--accent) 8%, transparent);
}

.ddl-row .ddl-text {
  color: var(--danger);
}

.pri {
  font-size: var(--text-2xs);
  background: var(--danger);
  color: #fff;
  border-radius: 4px;
  padding: 0 5px;
  flex: none;
}

.proj {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: var(--text-xs);
  color: var(--muted-fg);
  flex: none;
}

.dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
}

.tag {
  font-size: var(--text-xs);
  color: var(--muted-fg);
  flex: none;
}

.empty-ledger {
  color: var(--muted-fg);
  text-align: center;
  padding: 40px 0;
}
</style>
