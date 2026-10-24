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
</script>

<template>
  <div class="page">
    <h2>课题</h2>
    <p v-if="loadError" class="placeholder">{{ loadError }}</p>
    <ul v-else-if="projects.length" class="list">
      <li v-for="p in projects" :key="p.id">
        <span class="dot" :style="{ background: p.color ?? '#8a94a0' }" />
        <div class="body">
          <div class="name">{{ p.name }}</div>
          <div class="meta">{{ p.type }} · {{ p.status }}{{ p.role ? ` · ${p.role}` : '' }}</div>
        </div>
      </li>
    </ul>
    <p v-else class="placeholder">暂无课题 —— 课题创建与管理将在 P1 接入</p>
  </div>
</template>

<style scoped>
.list {
  list-style: none;
  margin: 16px 0 0;
  padding: 0;
}

.list li {
  display: flex;
  gap: 10px;
  align-items: center;
  padding: 10px 4px;
  border-bottom: 1px solid var(--line);
}

.dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex: none;
}

.name {
  font-weight: 500;
}

.meta {
  color: var(--muted-fg);
  font-size: 12px;
}
</style>
