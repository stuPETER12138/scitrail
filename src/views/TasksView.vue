<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import InlineTaskAdd from '../components/InlineTaskAdd.vue'
import TaskEditDrawer from '../components/TaskEditDrawer.vue'
import TaskItem from '../components/TaskItem.vue'
import type { Task } from '../api/commands'
import { useProjectsStore } from '../stores/projects'
import { useTasksStore } from '../stores/tasks'

const tasks = useTasksStore()
const projects = useProjectsStore()

type Tab = 'today' | 'inbox' | 'board' | 'done'
const TABS: { key: Tab; label: string }[] = [
  { key: 'today', label: '今日' },
  { key: 'inbox', label: '收件箱' },
  { key: 'board', label: '看板' },
  { key: 'done', label: '已完成' },
]
const tab = ref<Tab>('today')
const editing = ref<Task | null>(null)

async function loadTab() {
  if (tab.value === 'today') await tasks.loadToday()
  else if (tab.value === 'inbox') await tasks.loadInbox()
  else if (tab.value === 'done') await tasks.loadDone()
  else {
    await Promise.all([tasks.loadActive(), tasks.loadDone()])
  }
}

onMounted(() => {
  void projects.load()
  void loadTab()
})

watch(tab, () => void loadTab())

function afterChange() {
  void loadTab()
}

// ---- 看板拖拽 ----
const BOARD_COLUMNS = [
  { status: '待办', cls: 'c-todo' },
  { status: '进行中', cls: 'c-doing' },
  { status: '完成', cls: 'c-done' },
  { status: '放弃', cls: 'c-drop' },
]

function columnTasks(status: string): Task[] {
  if (status === '完成' || status === '放弃') {
    return tasks.done.filter((t) => t.status === status)
  }
  return tasks.active.filter((t) => t.status === status)
}

async function onDrop(status: string, e: DragEvent) {
  const id = Number(e.dataTransfer?.getData('text/plain'))
  if (!id) return
  const all = [...tasks.active, ...tasks.done]
  const t = all.find((x) => x.id === id)
  if (!t || t.status === status) return
  await tasks.update(t.id, {
    title: t.title,
    note: t.note,
    projectId: t.projectId,
    priority: t.priority,
    dueDate: t.dueDate,
    repeatRule: t.repeatRule,
    status,
  })
}
</script>

<template>
  <div class="page">
    <h2>任务</h2>

    <div class="tabs">
      <button
        v-for="t in TABS"
        :key="t.key"
        class="tab"
        :class="{ on: tab === t.key }"
        type="button"
        @click="tab = t.key"
      >
        {{ t.label }}
      </button>
    </div>

    <!-- 列表视图 -->
    <div v-if="tab !== 'board'" class="list">
      <InlineTaskAdd @added="afterChange" />
      <div v-if="tab === 'today' && tasks.today.length">
        <TaskItem
          v-for="t in tasks.today"
          :key="t.id"
          :task="t"
          @complete="tasks.complete"
          @edit="editing = $event"
        />
      </div>
      <div v-else-if="tab === 'inbox' && tasks.inbox.length">
        <TaskItem
          v-for="t in tasks.inbox"
          :key="t.id"
          :task="t"
          @complete="tasks.complete"
          @edit="editing = $event"
        />
      </div>
      <div v-else-if="tab === 'done' && tasks.done.length">
        <TaskItem
          v-for="t in tasks.done"
          :key="t.id"
          :task="t"
          @complete="tasks.complete"
          @edit="editing = $event"
        />
      </div>
      <p v-else class="empty">这里还没有任务</p>
    </div>

    <!-- 看板视图 -->
    <div v-else class="board">
      <div
        v-for="col in BOARD_COLUMNS"
        :key="col.status"
        class="col"
        :class="col.cls"
        @dragover.prevent
        @drop.prevent="onDrop(col.status, $event)"
      >
        <div class="col-h">{{ col.status }} · {{ columnTasks(col.status).length }}</div>
        <div
          v-for="t in columnTasks(col.status)"
          :key="t.id"
          class="card"
          draggable="true"
          @dragstart="(e) => e.dataTransfer?.setData('text/plain', String(t.id))"
          @click="editing = t"
        >
          <div class="ct">{{ t.title }}</div>
          <div class="cm">
            <span v-if="t.priority === 0" class="pri">!高</span>
            <span v-if="t.dueDate" class="due">{{ t.dueDate.slice(5) }}</span>
            <span v-if="t.repeatRule" class="rp">🔁</span>
          </div>
        </div>
        <p v-if="!columnTasks(col.status).length" class="col-empty">拖卡片到这里</p>
      </div>
    </div>

    <TaskEditDrawer v-if="editing" :task="editing" @close="editing = null" @saved="editing = null; afterChange()" />
  </div>
</template>

<style scoped>
.tabs {
  display: flex;
  gap: 6px;
  margin: 10px 0 14px;
}

.tab {
  border: 1px solid var(--line);
  background: var(--card);
  border-radius: 8px;
  padding: 5px 14px;
  cursor: pointer;
  color: var(--muted-fg);
  font-size: var(--text-md);
}

.tab.on {
  background: var(--hover);
  color: var(--fg);
  font-weight: 500;
}

.empty {
  color: var(--muted-fg);
  font-size: var(--text-md);
  padding: 20px 0;
}

/* 看板 */
.board {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
}

@media (max-width: 900px) {
  .board {
    grid-template-columns: repeat(2, 1fr);
  }
}

.col {
  background: color-mix(in srgb, var(--card) 60%, transparent);
  border: 1px solid var(--line);
  border-radius: var(--radius);
  padding: 10px;
  min-height: 220px;
}

.col-h {
  font-size: var(--text-sm);
  font-weight: 500;
  padding: 2px 4px 8px;
  color: var(--muted-fg);
}

.col.c-todo .col-h { color: #4a7fd6; }
.col.c-doing .col-h { color: #2f9e68; }
.col.c-done .col-h { color: var(--muted-fg); }
.col.c-drop .col-h { color: #8a94a0; }

.card {
  background: var(--card);
  border: 1px solid var(--line);
  border-radius: 8px;
  padding: 8px 10px;
  margin-bottom: 8px;
  cursor: grab;
}

.card:hover {
  border-color: var(--accent);
}

.ct {
  font-size: var(--text-md);
  margin-bottom: 5px;
}

.cm {
  display: flex;
  gap: 6px;
  align-items: center;
  font-size: var(--text-xs);
  color: var(--muted-fg);
}

.pri {
  background: var(--danger);
  color: #fff;
  border-radius: 4px;
  padding: 0 5px;
  font-size: var(--text-2xs);
}

.col-empty {
  color: var(--muted-fg);
  font-size: var(--text-xs);
  text-align: center;
  margin: 30px 0;
  opacity: 0.7;
}
</style>
