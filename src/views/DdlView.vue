<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import DdlForm from '../components/DdlForm.vue'
import type { Ddl } from '../api/commands'
import { MILESTONE_STATUSES, useTimeStore } from '../stores/time'
import { useProjectsStore } from '../stores/projects'
import { dueInfo } from '../utils/date'

const time = useTimeStore()
const projects = useProjectsStore()

const showCreate = ref(false)
const editing = ref<Ddl | null>(null)
const showClosed = ref(false)

onMounted(() => {
  void time.load(true)
  void projects.load()
})

const active = computed(() => time.ddls.filter((d) => d.status === '生效'))
const closed = computed(() => time.ddls.filter((d) => d.status !== '生效'))

const GROUPS = [
  { key: 'overdue', label: '已逾期', test: (n: number) => n < 0 },
  { key: 'week', label: '7 天内', test: (n: number) => n >= 0 && n <= 7 },
  { key: 'month', label: '30 天内', test: (n: number) => n > 7 && n <= 30 },
  { key: 'later', label: '更远', test: (n: number) => n > 30 },
]

const groups = computed(() =>
  GROUPS.map((g) => ({
    ...g,
    items: active.value.filter((d) => g.test(daysOf(d))),
  })),
)

function daysOf(d: Ddl): number {
  const [y, m, day] = d.dueDate.slice(0, 10).split('-').map(Number)
  const dueMs = Date.UTC(y, m - 1, day)
  const now = new Date()
  const todayMs = Date.UTC(now.getFullYear(), now.getMonth(), now.getDate())
  return Math.round((dueMs - todayMs) / 86400000)
}

function projectChip(d: Ddl) {
  return projects.byId(d.projectId)
}

async function settle(d: Ddl) {
  await time.update(d.id, {
    name: d.name,
    category: d.category,
    dueDate: d.dueDate,
    projectId: d.projectId,
    repeatYearly: d.repeatYearly,
    note: d.note,
    status: '已办结',
  })
}

async function removeDdl(d: Ddl) {
  if (window.confirm(`删除「${d.name}」？`)) await time.remove(d.id)
}

// 里程碑
const msForm = reactive({ name: '', targetDate: '' })

async function addMilestone() {
  if (!msForm.name.trim() || !msForm.targetDate) return
  await time.createM({ name: msForm.name.trim(), targetDate: msForm.targetDate })
  msForm.name = ''
  msForm.targetDate = ''
}

async function changeMs(id: number, name: string, targetDate: string, status: string) {
  await time.updateM(id, { name, targetDate: targetDate || null, status })
}

async function removeMs(id: number, name: string) {
  if (window.confirm(`删除里程碑「${name}」？`)) await time.removeM(id)
}
</script>

<template>
  <div class="page">
    <h2>DDL 时间轴</h2>

    <button v-if="!showCreate && !editing" class="btn" type="button" @click="showCreate = true">
      ＋ 新建 DDL
    </button>
    <DdlForm v-if="showCreate" :ddl="null" @saved="showCreate = false" @cancel="showCreate = false" />
    <DdlForm v-if="editing" :ddl="editing" @saved="editing = null" @cancel="editing = null" />

    <section v-for="g in groups" :key="g.key">
      <template v-if="g.items.length">
        <h3 :class="g.key">{{ g.label }} · {{ g.items.length }}</h3>
        <div v-for="d in g.items" :key="d.id" class="ddl-row">
          <span class="badge" :class="dueInfo(d.dueDate).cls">{{ dueInfo(d.dueDate).label }}</span>
          <span class="name">{{ d.name }}</span>
          <span v-if="d.dueDate.includes(' ')" class="time">{{ d.dueDate.split(' ')[1] }}</span>
          <span v-if="d.category" class="cat">{{ d.category }}</span>
          <span v-if="projectChip(d)" class="proj">
            <span class="dot" :style="{ background: projectChip(d)!.color ?? '#8a94a0' }" />
            {{ projectChip(d)!.name }}
          </span>
          <span class="acts">
            <button type="button" @click="settle(d)">办结</button>
            <button type="button" title="复制到明年并办结当前" @click="time.rollover(d.id)">明年</button>
            <button type="button" @click="editing = d">编辑</button>
            <button type="button" class="del" @click="removeDdl(d)">删除</button>
          </span>
        </div>
      </template>
    </section>
    <p v-if="!active.length" class="placeholder">暂无生效 DDL —— 点击上方「新建 DDL」</p>

    <button v-if="closed.length" class="btn ghost" type="button" @click="showClosed = !showClosed">
      {{ showClosed ? '收起' : '展开' }}已办结 / 已取消（{{ closed.length }}）
    </button>
    <section v-if="showClosed">
      <div v-for="d in closed" :key="d.id" class="ddl-row dim">
        <span class="badge">{{ d.status }}</span>
        <span class="name">{{ d.name }}</span>
        <span class="time">{{ d.dueDate.slice(0, 10) }}</span>
      </div>
    </section>

    <hr />

    <h2 class="ms-title">学位里程碑</h2>
    <div class="ms-add">
      <input v-model="msForm.name" placeholder="里程碑名称（如：开题）" @keydown.enter.prevent="addMilestone" />
      <input v-model="msForm.targetDate" type="date" />
      <button class="btn" type="button" @click="addMilestone">添加</button>
    </div>
    <div v-if="time.milestones.length">
      <div v-for="m in time.milestones" :key="m.id" class="ms-row" :class="{ done: m.status === '已完成' }">
        <span class="name">{{ m.name }}</span>
        <input
          class="ms-date"
          type="date"
          :value="m.targetDate ?? ''"
          @change="changeMs(m.id, m.name, ($event.target as HTMLInputElement).value, m.status)"
        />
        <span v-if="m.targetDate && m.status !== '已完成'" class="badge" :class="dueInfo(m.targetDate).cls">
          {{ dueInfo(m.targetDate).label }}
        </span>
        <select
          :value="m.status"
          @change="changeMs(m.id, m.name, m.targetDate ?? '', ($event.target as HTMLSelectElement).value)"
        >
          <option v-for="s in MILESTONE_STATUSES" :key="s">{{ s }}</option>
        </select>
        <button type="button" class="del" @click="removeMs(m.id, m.name)">删除</button>
      </div>
    </div>
    <p v-else class="placeholder">尚未设置里程碑 —— 可按《需求报告》2.5 模板录入（开题/中期/预答辩/送审/答辩）</p>
  </div>
</template>

<style scoped>
.btn {
  border: 1px solid var(--line);
  background: var(--card);
  border-radius: 8px;
  padding: 7px 14px;
  cursor: pointer;
}

.btn.primary,
.btn.ghost {
  background: var(--bg);
  color: var(--muted-fg);
}

section {
  margin-top: 16px;
}

h3 {
  font-size: var(--text-md);
  font-weight: 500;
  margin: 0 0 4px;
  color: var(--muted-fg);
}

h3.overdue {
  color: var(--danger);
}

h3.week {
  color: var(--warn);
}

.ddl-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 4px;
  border-bottom: 1px solid var(--line);
}

.ddl-row.dim {
  opacity: 0.6;
}

.badge {
  font-size: var(--text-xs);
  border-radius: 9px;
  padding: 1px 8px;
  background: var(--hover);
  color: var(--muted-fg);
  flex: none;
  min-width: 56px;
  text-align: center;
}

.badge.overdue {
  background: color-mix(in srgb, var(--danger) 14%, transparent);
  color: var(--danger);
  font-weight: 500;
}

.badge.today {
  background: color-mix(in srgb, var(--danger) 12%, transparent);
  color: var(--danger);
}

.badge.soon {
  background: color-mix(in srgb, var(--warn) 14%, transparent);
  color: var(--warn);
}

.name {
  font-weight: 500;
}

.time,
.cat,
.proj {
  font-size: var(--text-sm);
  color: var(--muted-fg);
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.proj .dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
}

.acts {
  margin-left: auto;
  display: flex;
  gap: 4px;
}

.acts button,
.del {
  border: none;
  background: none;
  color: var(--muted-fg);
  cursor: pointer;
  font-size: var(--text-sm);
  padding: 2px 6px;
  border-radius: 5px;
}

.acts button:hover {
  background: var(--hover);
  color: var(--fg);
}

.acts .del:hover,
.del:hover {
  color: var(--danger);
}

hr {
  border: none;
  border-top: 1px solid var(--line);
  margin: 26px 0 14px;
}

.ms-title {
  font-size: var(--text-lg);
  font-weight: 500;
}

.ms-add {
  display: flex;
  gap: 8px;
  margin: 12px 0 8px;
  flex-wrap: wrap;
}

.ms-add input {
  padding: 7px 10px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: var(--bg);
  color: var(--fg);
  outline: none;
}

.ms-add input:first-child {
  flex: 1;
  min-width: 200px;
}

.ms-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 4px;
  border-bottom: 1px solid var(--line);
}

.ms-row.done .name {
  text-decoration: line-through;
  color: var(--muted-fg);
}

.ms-row .name {
  font-weight: 500;
  flex: 1;
}

.ms-date {
  border: 1px solid var(--line);
  border-radius: 7px;
  padding: 4px 6px;
  background: var(--bg);
  color: var(--fg);
  font-size: var(--text-sm);
}

.ms-row select {
  border: 1px solid var(--line);
  border-radius: 7px;
  padding: 4px 6px;
  background: var(--bg);
  color: var(--fg);
  font-size: var(--text-sm);
}
</style>
