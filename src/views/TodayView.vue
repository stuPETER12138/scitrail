<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { listProjects, type Project } from '../api/commands'

const projects = ref<Project[]>([])
const loadError = ref('')

onMounted(async () => {
  try {
    projects.value = await listProjects()
  } catch (e) {
    loadError.value = String(e)
  }
})

const now = new Date()
const dateLabel = `${now.getFullYear()} 年 ${now.getMonth() + 1} 月 ${now.getDate()} 日`
</script>

<template>
  <div class="page">
    <h2>今日</h2>
    <p class="date">{{ dateLabel }}</p>

    <section>
      <h3>今日任务</h3>
      <p class="empty">暂无任务 —— P1 接入任务与速录语法后，Ctrl+Alt+K 随手记</p>
    </section>

    <section>
      <h3>倒计时</h3>
      <p class="empty">暂无临近 DDL —— P2 接入 DDL 与学位里程碑</p>
    </section>

    <section>
      <h3>课题</h3>
      <p v-if="loadError" class="empty error-text">数据库读取失败：{{ loadError }}</p>
      <ul v-else-if="projects.length" class="proj-list">
        <li v-for="p in projects" :key="p.id">
          <span class="dot" :style="{ background: p.color ?? '#8a94a0' }" />
          {{ p.name }}
          <span class="meta">{{ p.type }} · {{ p.status }}</span>
        </li>
      </ul>
      <p v-else class="empty">暂无课题 —— P1 接入课题创建</p>
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
  margin: 0 0 6px;
}

.empty {
  margin: 0;
  color: var(--muted-fg);
  font-size: 13px;
}

.error-text {
  color: var(--danger);
}

.proj-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.proj-list li {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 0;
  border-bottom: 1px solid var(--line);
}

.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.meta {
  margin-left: auto;
  color: var(--muted-fg);
  font-size: 12px;
}
</style>
