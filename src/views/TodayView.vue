<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { RouterLink } from 'vue-router'
import InlineTaskAdd from '../components/InlineTaskAdd.vue'
import TaskItem from '../components/TaskItem.vue'
import { todayEntryCount } from '../api/commands'
import { useProjectsStore } from '../stores/projects'
import { useTasksStore } from '../stores/tasks'
import { useTimeStore } from '../stores/time'
import { dueInfo, todayStr } from '../utils/date'

const tasks = useTasksStore()
const projects = useProjectsStore()
const time = useTimeStore()
const todayEntries = ref(0)

onMounted(() => {
  void projects.load()
  void tasks.loadToday()
  void tasks.loadStats()
  void tasks.loadActive()
  void time.load()
  void todayEntryCount().then((n) => (todayEntries.value = n))
})

const now = new Date()
const dateLabel = `${now.getFullYear()} 年 ${now.getMonth() + 1} 月 ${now.getDate()} 日 · ${['日', '一', '二', '三', '四', '五', '六'][now.getDay()]}`

const overdueCount = computed(
  () => tasks.today.filter((t) => t.dueDate && t.dueDate < todayStr()).length,
)

const activeDdls = computed(() => time.ddls.filter((d) => d.status === '生效'))
const countdownDdls = computed(() => {
  const overdue = activeDdls.value.filter((d) => dueInfo(d.dueDate).cls === 'overdue')
  const upcoming = activeDdls.value.filter((d) => {
    const c = dueInfo(d.dueDate).cls
    return c !== 'overdue' && c !== 'later'
  })
  const later = activeDdls.value.filter((d) => dueInfo(d.dueDate).cls === 'later')
  return [...overdue, ...upcoming, ...later].slice(0, 6)
})
const nextMilestone = computed(() =>
  [...time.milestones]
    .filter((m) => m.status !== '已完成' && m.targetDate)
    .sort((a, b) => (a.targetDate ?? '').localeCompare(b.targetDate ?? ''))[0],
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
      <template v-if="countdownDdls.length || nextMilestone">
        <div v-for="d in countdownDdls" :key="d.id" class="cd-row">
          <span>⏰ {{ d.name }}</span>
          <span class="badge" :class="dueInfo(d.dueDate).cls">{{ dueInfo(d.dueDate).label }}</span>
        </div>
        <div v-if="nextMilestone" class="cd-row ms">
          <span>🎓 里程碑「{{ nextMilestone.name }}」</span>
          <span
            v-if="nextMilestone.targetDate"
            class="badge"
            :class="dueInfo(nextMilestone.targetDate).cls"
          >
            {{ dueInfo(nextMilestone.targetDate).label }}
          </span>
        </div>
      </template>
      <p v-else class="empty">
        暂无临近 DDL —— 到 <RouterLink to="/ddl">DDL 页</RouterLink> 添加
      </p>
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
      <p class="empty traj-link">
        🧭 今日轨迹：{{ todayEntries }} 条 ——
        <RouterLink to="/trajectory">进入轨迹页</RouterLink>
      </p>
    </section>
  </div>
</template>

<style scoped>
.date {
  color: var(--muted-fg);
  font-size: var(--text-sm);
  margin: 0 0 8px;
}

section {
  margin-top: 18px;
}

h3 {
  font-size: var(--text-md);
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
  font-size: var(--text-xs);
}

.empty {
  margin: 0;
  color: var(--muted-fg);
  font-size: var(--text-md);
}

.empty a {
  color: var(--accent);
}

.cd-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 2px;
  border-bottom: 1px solid var(--line);
  font-size: var(--text-md);
}

.cd-row.ms span:first-child {
  color: var(--muted-fg);
}

.badge {
  font-size: var(--text-xs);
  border-radius: 9px;
  padding: 1px 8px;
  background: var(--hover);
  color: var(--muted-fg);
  flex: none;
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
  font-size: var(--text-xs);
}
</style>
