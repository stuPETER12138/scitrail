<script setup lang="ts">
import { computed, onMounted } from 'vue'
import InlineTaskAdd from '../components/InlineTaskAdd.vue'
import TaskItem from '../components/TaskItem.vue'
import { useProjectsStore } from '../stores/projects'
import { useTasksStore } from '../stores/tasks'
import { todayStr } from '../utils/date'

const tasks = useTasksStore()
const projects = useProjectsStore()

onMounted(() => {
  void projects.load()
  void tasks.loadToday()
  void tasks.loadStats()
  void tasks.loadActive()
})

const now = new Date()
const dateLabel = `${now.getFullYear()} 年 ${now.getMonth() + 1} 月 ${now.getDate()} 日 · ${['日', '一', '二', '三', '四', '五', '六'][now.getDay()]}`

const overdueCount = computed(
  () => tasks.today.filter((t) => t.dueDate && t.dueDate < todayStr()).length,
)

const activeProjects = computed(() => projects.projects.filter((p) => p.status === '进行中' || p.status === '暂停'))

function projectActiveCount(projectId: number) {
  return tasks.active.filter((t) => t.projectId === projectId).length
}
</script>

<template>
  <div class="page">
    <h2>今日</h2>
    <p class="date">{{ dateLabel }}</p>

    <section>
      <h3>今日任务 <span v-if="overdueCount" class="warn-badge">逾期 {{ overdueCount }}</span></h3>
      <InlineTaskAdd @added="tasks.loadToday()" />
      <div v-if="tasks.today.length">
        <TaskItem
          v-for="t in tasks.today"
          :key="t.id"
          :task="t"
          @complete="tasks.complete"
          @edit="() => {}"
        />
      </div>
      <p v-else class="empty">今天没有到期任务 —— 用上方输入框或 Ctrl+Alt+K 速录一条</p>
    </section>

    <section>
      <h3>倒计时</h3>
      <p class="empty">DDL 与学位里程碑将在 P2 接入</p>
    </section>

    <section>
      <h3>课题</h3>
      <div v-if="activeProjects.length" class="cards">
        <div v-for="p in activeProjects" :key="p.id" class="pcard">
          <div class="nm">
            <span class="dot" :style="{ background: p.color ?? '#8a94a0' }" />
            {{ p.name }}
          </div>
          <div class="meta">{{ p.type }} · {{ p.status }} · {{ projectActiveCount(p.id) }} 个进行中任务</div>
        </div>
      </div>
      <p v-else class="empty">暂无课题 —— 到「课题」页新建一个</p>
    </section>

    <section v-if="tasks.stats">
      <h3>本周小结</h3>
      <p class="empty">
        新建 {{ tasks.stats.tasksCreated }} · 完成 {{ tasks.stats.tasksCompleted }} · 记录
        {{ tasks.stats.entriesCreated }} 条
      </p>
    </section>
  </div>
</template>

<style scoped>
.date {
  color: var(--muted-fg);
  font-size: 12px;
  margin: 0 0 8px;
}

section {
  margin-top: 18px;
}

h3 {
  font-size: 13px;
  font-weight: 500;
  color: var(--muted-fg);
  margin: 0 0 8px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.warn-badge {
  background: color-mix(in srgb, var(--danger) 14%, transparent);
  color: var(--danger);
  border-radius: 9px;
  padding: 0 8px;
  font-size: 11px;
}

.empty {
  margin: 0;
  color: var(--muted-fg);
  font-size: 13px;
}

.cards {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 10px;
}

.pcard {
  border: 1px solid var(--line);
  border-radius: var(--radius);
  padding: 10px 12px;
  background: var(--card);
}

.nm {
  display: flex;
  align-items: center;
  gap: 7px;
  font-weight: 500;
}

.dot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  flex: none;
}

.meta {
  margin-top: 6px;
  color: var(--muted-fg);
  font-size: 11px;
}
</style>
